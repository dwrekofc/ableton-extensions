use anyhow::{Context, Result};
use directories::BaseDirs;
use palette_protocol::{CatalogItem, ItemKind, ItemSource, LiveDatabaseImportSummary, TrackKind};
use rusqlite::{Connection, OpenFlags};
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn default_live_plugin_database() -> Result<PathBuf> {
    let base = BaseDirs::new().context("unable to determine the user data directory")?;
    Ok(base
        .data_dir()
        .join("Ableton")
        .join("Live Database")
        .join("Live-plugins-1.db"))
}

pub fn import_live_plugin_database(
    path: &Path,
) -> Result<(Vec<CatalogItem>, LiveDatabaseImportSummary)> {
    anyhow::ensure!(
        path.is_file(),
        "Ableton plug-in database not found: {}",
        path.display()
    );
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("opening Ableton plug-in database {}", path.display()))?;
    let schema_version = connection
        .query_row("SELECT version FROM version LIMIT 1", [], |row| row.get(0))
        .context("reading Ableton plug-in database version")?;
    let discovered: i64 =
        connection.query_row("SELECT COUNT(*) FROM plugins", [], |row| row.get(0))?;
    let mut statement = connection.prepare(
        "SELECT p.dev_identifier, p.name, p.vendor, p.version, p.sdk_version,
                p.subcategories, m.path
         FROM plugins p
         LEFT JOIN plugin_modules m ON m.module_id=p.module_id
         WHERE p.enabled=1 AND p.scanstate=1
         ORDER BY p.name COLLATE NOCASE, p.dev_identifier",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(PluginRow {
            device_identifier: row.get(0)?,
            name: row.get(1)?,
            vendor: row.get(2)?,
            version: row.get(3)?,
            sdk_version: row.get(4)?,
            subcategories: row.get(5)?,
            module_path: row.get(6)?,
        })
    })?;

    let mut items = Vec::new();
    let mut instruments = 0;
    let mut audio_effects = 0;
    let mut vendors = HashSet::new();
    for row in rows {
        let row = row?;
        let is_instrument = row.device_identifier.contains(":instr:");
        if is_instrument {
            instruments += 1;
        } else {
            audio_effects += 1;
        }
        if !row.vendor.is_empty() {
            vendors.insert(row.vendor.to_lowercase());
        }
        let plugin_format = row
            .device_identifier
            .strip_prefix("device:")
            .and_then(|value| value.split(':').next())
            .unwrap_or("plugin")
            .to_uppercase();
        let mut tags = vec![plugin_format.clone()];
        tags.extend(
            row.subcategories
                .split('|')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        );
        let mut categories = vec![
            "Plug-Ins".into(),
            if is_instrument {
                "Instruments".into()
            } else {
                "Audio Effects".into()
            },
        ];
        if !row.vendor.is_empty() {
            categories.push(row.vendor.clone());
            tags.push(row.vendor.clone());
        }
        items.push(CatalogItem {
            id: format!("live-plugin:{}", row.device_identifier),
            kind: ItemKind::Plugin,
            source: ItemSource::LiveDatabase,
            name: row.name,
            aliases: Vec::new(),
            categories,
            tags,
            browser_path: Vec::new(),
            compatible_tracks: if is_instrument {
                vec![TrackKind::Midi]
            } else {
                vec![TrackKind::Audio, TrackKind::Midi, TrackKind::Return]
            },
            favorite: false,
            pinned: false,
            usage_count: 0,
            last_used_at: None,
            metadata: json!({
                "ableton_index_schema": schema_version,
                "device_identifier": row.device_identifier,
                "vendor": row.vendor,
                "version": row.version,
                "sdk_version": row.sdk_version,
                "subcategories": row.subcategories,
                "module_path": row.module_path,
                "plugin_format": plugin_format,
                "browser_resolved": false,
            }),
        });
    }
    let summary = LiveDatabaseImportSummary {
        plugin_database: path.display().to_string(),
        schema_version,
        discovered: discovered.max(0) as usize,
        imported: items.len(),
        instruments,
        audio_effects,
        vendors: vendors.len(),
    };
    Ok((items, summary))
}

struct PluginRow {
    device_identifier: String,
    name: String,
    vendor: String,
    version: String,
    sdk_version: String,
    subcategories: String,
    module_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn imports_only_enabled_successfully_scanned_plugins() {
        let path = std::env::temp_dir().join(format!("palette-live-index-{}.db", Uuid::new_v4()));
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE version(version INT, platform INT);
                     INSERT INTO version VALUES(1, 2);
                     CREATE TABLE plugin_modules(module_id INTEGER PRIMARY KEY, path TEXT);
                     CREATE TABLE plugins(
                       plugin_id INTEGER PRIMARY KEY, module_id INTEGER, dev_identifier TEXT,
                       name TEXT, vendor TEXT, version TEXT, sdk_version TEXT, flags INTEGER,
                       scanstate INTEGER, subcategories TEXT, enabled INTEGER
                     );
                     INSERT INTO plugin_modules VALUES(1, '/plugins/Synth.vst3');
                     INSERT INTO plugins VALUES(
                       1, 1, 'device:vst3:instr:abc', 'Synth', 'Vendor', '1.2', 'VST 3.7',
                       1, 1, 'Instrument|Synth', 1
                     );
                     INSERT INTO plugins VALUES(
                       2, 1, 'device:vst3:audiofx:disabled', 'Disabled', 'Vendor', '1', 'VST',
                       1, 1, 'Fx', 0
                     );",
                )
                .unwrap();
        }
        let (items, summary) = import_live_plugin_database(&path).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(summary.instruments, 1);
        assert_eq!(summary.discovered, 2);
        assert_eq!(items[0].source, ItemSource::LiveDatabase);
        assert_eq!(items[0].compatible_tracks, vec![TrackKind::Midi]);
        assert_eq!(items[0].metadata["vendor"], "Vendor");
        std::fs::remove_file(path).unwrap();
    }
}
