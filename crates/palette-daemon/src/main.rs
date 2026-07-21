use anyhow::{Context, Result};
use clap::Parser;
use palette_core::{
    AppPaths, RuntimeConfig, Store, default_live_plugin_database, import_live_plugin_database,
};
use palette_protocol::{
    MAX_FRAME_BYTES, PROTOCOL_VERSION, PeerRole, PeerTarget, ProtocolError, RequestKind,
    ResponseData, ServiceStatus, WireMessage,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock, mpsc},
};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "ableton-palette-daemon")]
struct Args {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long, help = "Create runtime configuration and exit")]
    init: bool,
    #[arg(long, help = "Override the loopback port and persist it in config")]
    port: Option<u16>,
}

#[derive(Clone)]
struct PeerHandle {
    id: Uuid,
    sender: mpsc::Sender<WireMessage>,
}

#[derive(Clone)]
struct PendingRequest {
    client: mpsc::Sender<WireMessage>,
    original: RequestKind,
}

struct State {
    config: RuntimeConfig,
    paths: AppPaths,
    store: Mutex<Store>,
    peers: RwLock<HashMap<PeerRole, PeerHandle>>,
    pending: Mutex<HashMap<String, PendingRequest>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();
    let paths = args
        .data_dir
        .map(AppPaths::from_data_dir)
        .map(Ok)
        .unwrap_or_else(AppPaths::discover)?;
    paths.ensure()?;
    let mut config = load_or_create_config(&paths)?;
    if let Some(port) = args.port {
        config.port = port;
        config.save(&paths.config_file)?;
    }
    if args.init {
        println!("{}", paths.config_file.display());
        return Ok(());
    }

    let store = Store::open(&paths.database_file)?;
    let state = Arc::new(State {
        config: config.clone(),
        paths,
        store: Mutex::new(store),
        peers: RwLock::new(HashMap::new()),
        pending: Mutex::new(HashMap::new()),
    });

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("binding daemon to {address}"))?;
    info!(%address, "Ableton Command Palette daemon listening");

    loop {
        let (stream, remote) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, state).await {
                warn!(%remote, error = ?error, "connection ended with error");
            }
        });
    }
}

fn load_or_create_config(paths: &AppPaths) -> Result<RuntimeConfig> {
    if paths.config_file.exists() {
        return RuntimeConfig::load(&paths.config_file);
    }
    let config = RuntimeConfig::new(Uuid::new_v4().to_string());
    config.save(&paths.config_file)?;
    Ok(config)
}

async fn handle_connection(stream: TcpStream, state: Arc<State>) -> Result<()> {
    stream.set_nodelay(true)?;
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();
    let hello_line = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
        .await
        .context("hello timeout")??
        .context("peer disconnected before hello")?;
    ensure_frame_size(&hello_line)?;
    let hello: WireMessage = serde_json::from_str(&hello_line).context("invalid hello frame")?;
    let (role, peer_name) = match hello {
        WireMessage::Hello {
            protocol_version,
            role,
            token,
            peer_name,
            ..
        } => {
            anyhow::ensure!(
                protocol_version == PROTOCOL_VERSION,
                "unsupported protocol version {protocol_version}"
            );
            anyhow::ensure!(
                constant_time_eq(&token, &state.config.token),
                "authentication failed"
            );
            (role, peer_name)
        }
        _ => anyhow::bail!("first frame must be hello"),
    };

    write_frame(
        &mut write_half,
        &WireMessage::HelloAck {
            protocol_version: PROTOCOL_VERSION,
            daemon_version: DAEMON_VERSION.into(),
        },
    )
    .await?;

    let (sender, mut receiver) = mpsc::channel::<WireMessage>(64);
    let peer_id = Uuid::new_v4();
    if role != PeerRole::Client {
        state.peers.write().await.insert(
            role.clone(),
            PeerHandle {
                id: peer_id,
                sender: sender.clone(),
            },
        );
        info!(?role, %peer_name, "integration peer connected");
    }

    let writer = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            write_frame(&mut write_half, &message).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    while let Some(line) = lines.next_line().await? {
        ensure_frame_size(&line)?;
        let message: WireMessage = serde_json::from_str(&line).context("invalid protocol frame")?;
        match message {
            WireMessage::Request {
                request_id,
                request,
            } if role == PeerRole::Client => {
                route_client_request(request_id, request, sender.clone(), state.clone()).await;
            }
            WireMessage::Response {
                request_id,
                result,
                error,
            } if role != PeerRole::Client => {
                route_peer_response(request_id, result, error, state.clone()).await;
            }
            WireMessage::Event { event, data } if role != PeerRole::Client => {
                info!(?role, %event, data = %data, "integration event");
            }
            _ => warn!(?role, "ignored unexpected protocol message"),
        }
    }

    drop(sender);
    if role != PeerRole::Client {
        let mut peers = state.peers.write().await;
        if peers.get(&role).is_some_and(|peer| peer.id == peer_id) {
            peers.remove(&role);
            info!(?role, %peer_name, "integration peer disconnected");
        }
    }
    writer.abort();
    Ok(())
}

async fn route_client_request(
    request_id: String,
    mut request: RequestKind,
    client: mpsc::Sender<WireMessage>,
    state: Arc<State>,
) {
    if let RequestKind::LoadItem {
        item_id,
        browser_path,
        ..
    } = &mut request
    {
        if browser_path.is_empty() {
            match state.store.lock().await.get_item(item_id) {
                Ok(Some(item)) => {
                    if item.source == palette_protocol::ItemSource::LiveDatabase
                        && item.browser_path.is_empty()
                    {
                        let _ = client
                            .send(WireMessage::failure(
                                request_id,
                                ProtocolError::new(
                                    "browser_path_unresolved",
                                    "plug-in was discovered in Ableton's index but still needs Browser-path resolution before loading",
                                ),
                            ))
                            .await;
                        return;
                    }
                    *browser_path = item.browser_path;
                }
                Ok(None) => {
                    let _ = client
                        .send(WireMessage::failure(
                            request_id,
                            ProtocolError::new(
                                "item_not_found",
                                "catalog item not found; scan the Live Browser first",
                            ),
                        ))
                        .await;
                    return;
                }
                Err(error) => {
                    let _ = client
                        .send(WireMessage::failure(
                            request_id,
                            ProtocolError::new("daemon_error", error.to_string()),
                        ))
                        .await;
                    return;
                }
            }
        }
    }
    match request.target() {
        PeerTarget::Daemon => {
            let response = handle_daemon_request(&request_id, request, state.clone()).await;
            let _ = client.send(response).await;
        }
        PeerTarget::Bridge | PeerTarget::Extension => {
            forward_request(request_id, request, client, state).await;
        }
    }
}

async fn forward_request(
    request_id: String,
    request: RequestKind,
    client: mpsc::Sender<WireMessage>,
    state: Arc<State>,
) {
    let role = match request.target() {
        PeerTarget::Bridge => PeerRole::Bridge,
        PeerTarget::Extension => PeerRole::Extension,
        PeerTarget::Daemon => unreachable!(),
    };
    let peer = state.peers.read().await.get(&role).cloned();
    let Some(peer) = peer else {
        let _ = client
            .send(WireMessage::failure(
                request_id,
                ProtocolError {
                    code: "peer_unavailable".into(),
                    message: format!("{role:?} is not connected"),
                    retryable: true,
                },
            ))
            .await;
        return;
    };

    state.pending.lock().await.insert(
        request_id.clone(),
        PendingRequest {
            client: client.clone(),
            original: request.clone(),
        },
    );
    if peer
        .sender
        .send(WireMessage::Request {
            request_id: request_id.clone(),
            request,
        })
        .await
        .is_err()
    {
        state.pending.lock().await.remove(&request_id);
        let _ = client
            .send(WireMessage::failure(
                request_id,
                ProtocolError::new("peer_disconnected", "integration peer disconnected"),
            ))
            .await;
        return;
    }

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(120)).await;
        if let Some(pending) = state.pending.lock().await.remove(&request_id) {
            let _ = pending
                .client
                .send(WireMessage::failure(
                    request_id,
                    ProtocolError {
                        code: "request_timeout".into(),
                        message: "Ableton integration did not respond within 120 seconds".into(),
                        retryable: true,
                    },
                ))
                .await;
        }
    });
}

async fn route_peer_response(
    request_id: String,
    result: Option<ResponseData>,
    error: Option<ProtocolError>,
    state: Arc<State>,
) {
    let Some(pending) = state.pending.lock().await.remove(&request_id) else {
        warn!(%request_id, "received response for unknown request");
        return;
    };
    if error.is_none() {
        if let Some(ResponseData::Catalog(items)) = &result {
            if let Err(error) = state.store.lock().await.upsert_catalog(items) {
                error!(%error, "failed to persist scanned catalog");
            }
        }
        if let RequestKind::LoadItem { item_id, .. } = &pending.original {
            if let Err(error) = state.store.lock().await.record_usage(item_id) {
                warn!(%error, %item_id, "failed to record usage");
            }
        }
    }
    let _ = pending
        .client
        .send(WireMessage::Response {
            request_id,
            result,
            error,
        })
        .await;
}

async fn handle_daemon_request(
    request_id: &str,
    request: RequestKind,
    state: Arc<State>,
) -> WireMessage {
    if let RequestKind::RunWorkflow { workflow_id } = &request {
        let mut workflow = match state.store.lock().await.get_workflow(workflow_id) {
            Ok(Some(workflow)) => workflow,
            Ok(None) => {
                return WireMessage::failure(
                    request_id,
                    ProtocolError::new(
                        "workflow_not_found",
                        format!("workflow not found: {workflow_id}"),
                    ),
                );
            }
            Err(error) => {
                return WireMessage::failure(
                    request_id,
                    ProtocolError::new("daemon_error", error.to_string()),
                );
            }
        };
        for action in &mut workflow.actions {
            if let palette_protocol::WorkflowAction::LoadItem {
                item_id,
                browser_path,
                ..
            } = action
            {
                if browser_path.is_empty() {
                    if let Ok(Some(item)) = state.store.lock().await.get_item(item_id) {
                        *browser_path = item.browser_path;
                    }
                }
            }
        }
        let bridge_request = RequestKind::ExecuteWorkflow { workflow };
        let (tx, mut rx) = mpsc::channel(1);
        forward_request(request_id.to_string(), bridge_request, tx, state.clone()).await;
        return rx.recv().await.unwrap_or_else(|| {
            WireMessage::failure(
                request_id,
                ProtocolError::new("internal_error", "workflow routing failed"),
            )
        });
    }

    let outcome: Result<ResponseData> = async {
        match request {
            RequestKind::Ping => Ok(ResponseData::Pong),
            RequestKind::Status => {
                let peers = state.peers.read().await;
                let catalog_count = state.store.lock().await.catalog_count()?;
                Ok(ResponseData::Status(ServiceStatus {
                    protocol_version: PROTOCOL_VERSION,
                    bridge_connected: peers.contains_key(&PeerRole::Bridge),
                    extension_connected: peers.contains_key(&PeerRole::Extension),
                    catalog_count,
                    data_directory: state.paths.data_dir.display().to_string(),
                }))
            }
            RequestKind::Search { query, limit } => Ok(ResponseData::SearchResults(
                state.store.lock().await.search(&query, limit)?,
            )),
            RequestKind::ImportLiveDatabase { plugin_database } => {
                let path = plugin_database
                    .map(PathBuf::from)
                    .map(Ok)
                    .unwrap_or_else(default_live_plugin_database)?;
                let (items, summary) = import_live_plugin_database(&path)?;
                state.store.lock().await.upsert_catalog(&items)?;
                Ok(ResponseData::LiveDatabaseImport(summary))
            }
            RequestKind::SetFavorite { item_id, favorite } => {
                state.store.lock().await.set_favorite(&item_id, favorite)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::SetPinned { item_id, pinned } => {
                state.store.lock().await.set_pinned(&item_id, pinned)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::AddAlias { item_id, alias } => {
                state.store.lock().await.add_alias(&item_id, &alias)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::RemoveAlias { item_id, alias } => {
                state.store.lock().await.remove_alias(&item_id, &alias)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::AddTag { item_id, tag } => {
                state.store.lock().await.add_tag(&item_id, &tag)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::RemoveTag { item_id, tag } => {
                state.store.lock().await.remove_tag(&item_id, &tag)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::CreateCollection { name } => {
                state.store.lock().await.create_collection(&name)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::AddToCollection { name, item_id } => {
                state
                    .store
                    .lock()
                    .await
                    .add_to_collection(&name, &item_id)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::RemoveFromCollection { name, item_id } => {
                state
                    .store
                    .lock()
                    .await
                    .remove_from_collection(&name, &item_id)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::DeleteCollection { name } => {
                state.store.lock().await.delete_collection(&name)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::ListCollections => Ok(ResponseData::Collections(
                state.store.lock().await.list_collections()?,
            )),
            RequestKind::SaveWorkflow { workflow } => {
                state.store.lock().await.save_workflow(&workflow)?;
                Ok(ResponseData::Workflow(workflow))
            }
            RequestKind::DeleteWorkflow { workflow_id } => {
                state.store.lock().await.delete_workflow(&workflow_id)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::ListWorkflows => Ok(ResponseData::Workflows(
                state.store.lock().await.list_workflows()?,
            )),
            RequestKind::SetHotkey {
                accelerator,
                target_id,
            } => {
                state
                    .store
                    .lock()
                    .await
                    .set_hotkey(&accelerator, &target_id)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::ClearHotkey { accelerator } => {
                state.store.lock().await.clear_hotkey(&accelerator)?;
                Ok(ResponseData::Ack)
            }
            RequestKind::ListHotkeys => Ok(ResponseData::Hotkeys(
                state.store.lock().await.list_hotkeys()?,
            )),
            _ => anyhow::bail!("request is not handled by daemon"),
        }
    }
    .await;

    match outcome {
        Ok(result) => WireMessage::success(request_id, result),
        Err(error) => WireMessage::failure(
            request_id,
            ProtocolError::new("daemon_error", error.to_string()),
        ),
    }
}

async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, message: &WireMessage) -> Result<()> {
    let mut bytes = serde_json::to_vec(message)?;
    anyhow::ensure!(
        bytes.len() <= MAX_FRAME_BYTES,
        "outgoing frame exceeds size limit"
    );
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    Ok(())
}

fn ensure_frame_size(frame: &str) -> Result<()> {
    anyhow::ensure!(
        frame.len() <= MAX_FRAME_BYTES,
        "incoming frame exceeds size limit"
    );
    Ok(())
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_comparison_rejects_length_and_content_changes() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
    }
}
