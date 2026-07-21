use anyhow::{Context, Result, anyhow, bail};
use palette_core::{AppPaths, RuntimeConfig};
use palette_protocol::{
    MAX_FRAME_BYTES, PROTOCOL_VERSION, PeerRole, RequestKind, ResponseData, WireMessage,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct DaemonClient {
    config: RuntimeConfig,
}

impl DaemonClient {
    pub fn discover() -> Result<Self> {
        let paths = AppPaths::discover()?;
        let config = RuntimeConfig::load(&paths.config_file).with_context(|| {
            format!(
                "Command Palette is not installed yet ({})",
                paths.config_file.display()
            )
        })?;
        Ok(Self { config })
    }

    pub fn request(&self, request: RequestKind) -> Result<ResponseData> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let address = address
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("could not resolve the local palette service"))?;
        let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .context("Ableton Command Palette service is not running")?;
        stream.set_read_timeout(Some(Duration::from_secs(125)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        write_frame(
            &mut stream,
            &WireMessage::Hello {
                protocol_version: PROTOCOL_VERSION,
                role: PeerRole::Client,
                token: self.config.token.clone(),
                peer_name: "gpui-command-bar".into(),
                peer_version: env!("CARGO_PKG_VERSION").into(),
                capabilities: vec!["command_bar".into()],
            },
        )?;

        let mut reader = BufReader::new(stream.try_clone()?);
        match read_frame(&mut reader)? {
            WireMessage::HelloAck { .. } => {}
            WireMessage::Response {
                error: Some(error), ..
            } => bail!(error.message),
            message => bail!("unexpected service handshake: {message:?}"),
        }

        let request_id = Uuid::new_v4().to_string();
        write_frame(
            &mut stream,
            &WireMessage::Request {
                request_id: request_id.clone(),
                request,
            },
        )?;

        loop {
            match read_frame(&mut reader)? {
                WireMessage::Response {
                    request_id: response_id,
                    result,
                    error,
                } if response_id == request_id => {
                    if let Some(error) = error {
                        bail!(error.message);
                    }
                    return result.ok_or_else(|| anyhow!("service returned an empty response"));
                }
                WireMessage::Event { .. } => continue,
                _ => continue,
            }
        }
    }
}

fn write_frame(stream: &mut TcpStream, message: &WireMessage) -> Result<()> {
    let mut bytes = serde_json::to_vec(message)?;
    if bytes.len() > MAX_FRAME_BYTES {
        bail!("request exceeds the protocol frame limit");
    }
    bytes.push(b'\n');
    stream.write_all(&bytes)?;
    stream.flush()?;
    Ok(())
}

fn read_frame(reader: &mut BufReader<TcpStream>) -> Result<WireMessage> {
    let mut frame = String::new();
    let count = reader.read_line(&mut frame)?;
    if count == 0 {
        bail!("palette service closed the connection");
    }
    if frame.len() > MAX_FRAME_BYTES {
        bail!("response exceeds the protocol frame limit");
    }
    Ok(serde_json::from_str(&frame)?)
}
