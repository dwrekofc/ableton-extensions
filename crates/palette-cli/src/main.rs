use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use palette_core::{AppPaths, RuntimeConfig};
use palette_protocol::{
    InsertionPosition, MAX_FRAME_BYTES, PROTOCOL_VERSION, PeerRole, ProtocolError, RequestKind,
    WireMessage, WorkflowDefinition,
};
use std::path::PathBuf;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "ableton-palette",
    about = "Headless test client for Ableton Command Palette"
)]
struct Args {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long, global = true, help = "Print the full protocol response as JSON")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Ping,
    Status,
    Context,
    Scan {
        #[arg(long, default_value_t = 20_000)]
        max_items: usize,
        #[arg(long, default_value_t = 12)]
        max_depth: usize,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    ImportLiveIndex {
        #[arg(long)]
        plugin_database: Option<PathBuf>,
    },
    Load {
        item_id: String,
        #[arg(long, value_enum, default_value_t = PositionArg::After)]
        position: PositionArg,
    },
    InsertNative {
        name: String,
        #[arg(long, value_enum, default_value_t = PositionArg::After)]
        position: PositionArg,
    },
    SdkInsertNative {
        name: String,
        #[arg(long, value_enum, default_value_t = PositionArg::After)]
        position: PositionArg,
    },
    Favorite {
        item_id: String,
        #[arg(long, default_value_t = true)]
        enabled: bool,
    },
    Pin {
        item_id: String,
        #[arg(long, default_value_t = true)]
        enabled: bool,
    },
    Alias {
        item_id: String,
        alias: String,
    },
    RemoveAlias {
        item_id: String,
        alias: String,
    },
    Tag {
        item_id: String,
        tag: String,
    },
    RemoveTag {
        item_id: String,
        tag: String,
    },
    CreateCollection {
        name: String,
    },
    AddToCollection {
        name: String,
        item_id: String,
    },
    RemoveFromCollection {
        name: String,
        item_id: String,
    },
    DeleteCollection {
        name: String,
    },
    ListCollections,
    SaveWorkflow {
        file: PathBuf,
    },
    RunWorkflow {
        workflow_id: String,
    },
    DeleteWorkflow {
        workflow_id: String,
    },
    ListWorkflows,
    SetHotkey {
        accelerator: String,
        target_id: String,
    },
    ClearHotkey {
        accelerator: String,
    },
    ListHotkeys,
    Diagnostics,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PositionArg {
    Beginning,
    Before,
    After,
    End,
}

impl From<PositionArg> for InsertionPosition {
    fn from(value: PositionArg) -> Self {
        match value {
            PositionArg::Beginning => Self::Beginning,
            PositionArg::Before => Self::BeforeSelected,
            PositionArg::After => Self::AfterSelected,
            PositionArg::End => Self::End,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let paths = args
        .data_dir
        .map(AppPaths::from_data_dir)
        .map(Ok)
        .unwrap_or_else(AppPaths::discover)?;
    let config = RuntimeConfig::load(&paths.config_file).with_context(|| {
        format!(
            "runtime configuration not found; run `ableton-palette-daemon --init` first ({})",
            paths.config_file.display()
        )
    })?;
    let request = command_to_request(args.command)?;
    let response = send_request(&config, request).await?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        print_human(response)?;
    }
    Ok(())
}

fn command_to_request(command: Command) -> Result<RequestKind> {
    Ok(match command {
        Command::Ping => RequestKind::Ping,
        Command::Status => RequestKind::Status,
        Command::Context => RequestKind::GetContext,
        Command::Scan {
            max_items,
            max_depth,
        } => RequestKind::ScanCatalog {
            max_items: max_items.clamp(1, 250_000),
            max_depth: max_depth.clamp(1, 64),
        },
        Command::Search { query, limit } => RequestKind::Search { query, limit },
        Command::ImportLiveIndex { plugin_database } => RequestKind::ImportLiveDatabase {
            plugin_database: plugin_database.map(|path| path.display().to_string()),
        },
        Command::Load { item_id, position } => RequestKind::LoadItem {
            item_id,
            browser_path: Vec::new(),
            position: position.into(),
        },
        Command::InsertNative { name, position } => RequestKind::InsertNative {
            name,
            position: position.into(),
        },
        Command::SdkInsertNative { name, position } => RequestKind::SdkInsertNative {
            name,
            position: position.into(),
        },
        Command::Favorite { item_id, enabled } => RequestKind::SetFavorite {
            item_id,
            favorite: enabled,
        },
        Command::Pin { item_id, enabled } => RequestKind::SetPinned {
            item_id,
            pinned: enabled,
        },
        Command::Alias { item_id, alias } => RequestKind::AddAlias { item_id, alias },
        Command::RemoveAlias { item_id, alias } => RequestKind::RemoveAlias { item_id, alias },
        Command::Tag { item_id, tag } => RequestKind::AddTag { item_id, tag },
        Command::RemoveTag { item_id, tag } => RequestKind::RemoveTag { item_id, tag },
        Command::CreateCollection { name } => RequestKind::CreateCollection { name },
        Command::AddToCollection { name, item_id } => {
            RequestKind::AddToCollection { name, item_id }
        }
        Command::RemoveFromCollection { name, item_id } => {
            RequestKind::RemoveFromCollection { name, item_id }
        }
        Command::DeleteCollection { name } => RequestKind::DeleteCollection { name },
        Command::ListCollections => RequestKind::ListCollections,
        Command::SaveWorkflow { file } => {
            let source = std::fs::read_to_string(&file)
                .with_context(|| format!("reading workflow {}", file.display()))?;
            let workflow: WorkflowDefinition = serde_json::from_str(&source)
                .with_context(|| format!("parsing workflow {} as JSON", file.display()))?;
            RequestKind::SaveWorkflow { workflow }
        }
        Command::RunWorkflow { workflow_id } => RequestKind::RunWorkflow { workflow_id },
        Command::DeleteWorkflow { workflow_id } => RequestKind::DeleteWorkflow { workflow_id },
        Command::ListWorkflows => RequestKind::ListWorkflows,
        Command::SetHotkey {
            accelerator,
            target_id,
        } => RequestKind::SetHotkey {
            accelerator,
            target_id,
        },
        Command::ClearHotkey { accelerator } => RequestKind::ClearHotkey { accelerator },
        Command::ListHotkeys => RequestKind::ListHotkeys,
        Command::Diagnostics => RequestKind::Diagnostics,
    })
}

async fn send_request(config: &RuntimeConfig, request: RequestKind) -> Result<WireMessage> {
    let address = format!("{}:{}", config.host, config.port);
    let stream = TcpStream::connect(&address)
        .await
        .with_context(|| format!("connecting to daemon at {address}"))?;
    stream.set_nodelay(true)?;
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    write_frame(
        &mut write_half,
        &WireMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: PeerRole::Client,
            token: config.token.clone(),
            peer_name: "ableton-palette-cli".into(),
            peer_version: env!("CARGO_PKG_VERSION").into(),
            capabilities: vec!["headless_test".into()],
        },
    )
    .await?;
    let ack = read_frame(&mut lines).await?;
    anyhow::ensure!(
        matches!(ack, WireMessage::HelloAck { .. }),
        "daemon rejected hello"
    );

    let request_id = Uuid::new_v4().to_string();
    write_frame(
        &mut write_half,
        &WireMessage::Request {
            request_id: request_id.clone(),
            request,
        },
    )
    .await?;
    let response = read_frame(&mut lines).await?;
    match &response {
        WireMessage::Response {
            request_id: response_id,
            error: Some(ProtocolError { message, .. }),
            ..
        } if response_id == &request_id => anyhow::bail!(message.clone()),
        WireMessage::Response {
            request_id: response_id,
            ..
        } if response_id == &request_id => Ok(response),
        _ => anyhow::bail!("unexpected daemon response"),
    }
}

async fn write_frame<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    message: &WireMessage,
) -> Result<()> {
    let mut bytes = serde_json::to_vec(message)?;
    anyhow::ensure!(
        bytes.len() <= MAX_FRAME_BYTES,
        "outgoing frame exceeds size limit"
    );
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    Ok(())
}

async fn read_frame<R: tokio::io::AsyncBufRead + Unpin>(
    lines: &mut tokio::io::Lines<R>,
) -> Result<WireMessage> {
    let line = lines.next_line().await?.context("daemon disconnected")?;
    anyhow::ensure!(
        line.len() <= MAX_FRAME_BYTES,
        "incoming frame exceeds size limit"
    );
    Ok(serde_json::from_str(&line)?)
}

fn print_human(response: WireMessage) -> Result<()> {
    match response {
        WireMessage::Response {
            result: Some(result),
            ..
        } => println!("{}", serde_json::to_string_pretty(&result)?),
        other => println!("{}", serde_json::to_string_pretty(&other)?),
    }
    Ok(())
}
