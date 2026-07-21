use anyhow::Result;
use palette_core::AppPaths;
use palette_protocol::ItemKind;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentGroup {
    Devices,
    Plugins,
    Presets,
    Racks,
    MaxForLive,
    Samples,
    Loops,
    Commands,
    Workflows,
    Other,
}

impl ContentGroup {
    pub const SETTINGS: [Self; 9] = [
        Self::Devices,
        Self::Plugins,
        Self::Presets,
        Self::Racks,
        Self::MaxForLive,
        Self::Samples,
        Self::Loops,
        Self::Commands,
        Self::Workflows,
    ];

    pub fn for_kind(kind: &ItemKind) -> Self {
        match kind {
            ItemKind::NativeDevice => Self::Devices,
            ItemKind::Plugin => Self::Plugins,
            ItemKind::Preset => Self::Presets,
            ItemKind::Rack => Self::Racks,
            ItemKind::MaxDevice => Self::MaxForLive,
            ItemKind::Sample => Self::Samples,
            ItemKind::Loop => Self::Loops,
            ItemKind::Command => Self::Commands,
            ItemKind::Workflow => Self::Workflows,
            ItemKind::Unknown => Self::Other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Devices => "Devices",
            Self::Plugins => "Plug-ins",
            Self::Presets => "Presets",
            Self::Racks => "Racks",
            Self::MaxForLive => "Max for Live",
            Self::Samples => "Samples",
            Self::Loops => "Loops and clips",
            Self::Commands => "Commands",
            Self::Workflows => "Workflows",
            Self::Other => "Other",
        }
    }

    pub fn order(self) -> usize {
        match self {
            Self::Devices => 0,
            Self::Plugins => 1,
            Self::Presets => 2,
            Self::Racks => 3,
            Self::MaxForLive => 4,
            Self::Samples => 5,
            Self::Loops => 6,
            Self::Commands => 7,
            Self::Workflows => 8,
            Self::Other => 9,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ContentVisibility {
    pub devices: bool,
    pub plugins: bool,
    pub presets: bool,
    pub racks: bool,
    pub max_for_live: bool,
    pub samples: bool,
    pub loops: bool,
    pub commands: bool,
    pub workflows: bool,
}

impl Default for ContentVisibility {
    fn default() -> Self {
        Self {
            devices: true,
            plugins: true,
            presets: true,
            racks: true,
            max_for_live: true,
            samples: true,
            loops: true,
            commands: true,
            workflows: true,
        }
    }
}

impl ContentVisibility {
    pub fn is_visible(&self, group: ContentGroup) -> bool {
        match group {
            ContentGroup::Devices => self.devices,
            ContentGroup::Plugins => self.plugins,
            ContentGroup::Presets => self.presets,
            ContentGroup::Racks => self.racks,
            ContentGroup::MaxForLive => self.max_for_live,
            ContentGroup::Samples => self.samples,
            ContentGroup::Loops => self.loops,
            ContentGroup::Commands => self.commands,
            ContentGroup::Workflows => self.workflows,
            ContentGroup::Other => true,
        }
    }

    pub fn toggle(&mut self, group: ContentGroup) {
        match group {
            ContentGroup::Devices => self.devices = !self.devices,
            ContentGroup::Plugins => self.plugins = !self.plugins,
            ContentGroup::Presets => self.presets = !self.presets,
            ContentGroup::Racks => self.racks = !self.racks,
            ContentGroup::MaxForLive => self.max_for_live = !self.max_for_live,
            ContentGroup::Samples => self.samples = !self.samples,
            ContentGroup::Loops => self.loops = !self.loops,
            ContentGroup::Commands => self.commands = !self.commands,
            ContentGroup::Workflows => self.workflows = !self.workflows,
            ContentGroup::Other => {}
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteSettings {
    pub visibility: ContentVisibility,
}

impl PaletteSettings {
    pub fn load() -> Self {
        settings_path()
            .ok()
            .and_then(|path| fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = settings_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(temp, path)?;
        Ok(())
    }
}

fn settings_path() -> Result<PathBuf> {
    Ok(AppPaths::discover()?.data_dir.join("ui-settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_map_to_settings_and_default_visible() {
        let settings = PaletteSettings::default();
        assert_eq!(ContentGroup::for_kind(&ItemKind::Loop), ContentGroup::Loops);
        assert!(settings.visibility.is_visible(ContentGroup::Plugins));
    }

    #[test]
    fn visibility_toggle_is_reversible() {
        let mut visibility = ContentVisibility::default();
        visibility.toggle(ContentGroup::Samples);
        assert!(!visibility.is_visible(ContentGroup::Samples));
        visibility.toggle(ContentGroup::Samples);
        assert!(visibility.is_visible(ContentGroup::Samples));
    }
}
