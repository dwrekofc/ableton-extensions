use anyhow::{Context, Result};
use directories::BaseDirs;
use palette_protocol::DEFAULT_DAEMON_PORT;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub config_file: PathBuf,
    pub database_file: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        if let Ok(value) = std::env::var("ABLETON_PALETTE_DATA_DIR") {
            return Ok(Self::from_data_dir(PathBuf::from(value)));
        }
        let base = BaseDirs::new().context("unable to determine the user data directory")?;
        Ok(Self::from_data_dir(
            base.data_dir().join("Ableton Command Palette"),
        ))
    }

    pub fn from_data_dir(data_dir: PathBuf) -> Self {
        Self {
            config_file: data_dir.join("config.json"),
            database_file: data_dir.join("palette.sqlite3"),
            log_dir: data_dir.join("logs"),
            data_dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("creating {}", self.data_dir.display()))?;
        fs::create_dir_all(&self.log_dir)
            .with_context(|| format!("creating {}", self.log_dir.display()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub host: String,
    pub port: u16,
    pub token: String,
    pub protocol_version: u32,
}

impl RuntimeConfig {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: DEFAULT_DAEMON_PORT,
            token: token.into(),
            protocol_version: palette_protocol::PROTOCOL_VERSION,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}
