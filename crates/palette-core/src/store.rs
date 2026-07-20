use crate::{rank_items, validate_workflow};
use anyhow::{Context, Result};
use palette_protocol::{
    CatalogItem, CollectionSummary, HotkeyBinding, ItemKind, ItemSource, WorkflowDefinition,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store {
    connection: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("opening database {}", path.display()))?;
        let store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self> {
        let store = Self {
            connection: Connection::open_in_memory()?,
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        self.connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS catalog_items (
               id TEXT PRIMARY KEY,
               item_json TEXT NOT NULL,
               updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS personalization (
               item_id TEXT PRIMARY KEY,
               favorite INTEGER NOT NULL DEFAULT 0,
               pinned INTEGER NOT NULL DEFAULT 0,
               usage_count INTEGER NOT NULL DEFAULT 0,
               last_used_at INTEGER
             );
             CREATE TABLE IF NOT EXISTS aliases (
               item_id TEXT NOT NULL,
               alias TEXT NOT NULL COLLATE NOCASE,
               PRIMARY KEY(item_id, alias)
             );
             CREATE TABLE IF NOT EXISTS custom_tags (
               item_id TEXT NOT NULL,
               tag TEXT NOT NULL COLLATE NOCASE,
               PRIMARY KEY(item_id, tag)
             );
             CREATE TABLE IF NOT EXISTS collections (
               name TEXT NOT NULL COLLATE NOCASE,
               item_id TEXT NOT NULL,
               sort_order INTEGER NOT NULL DEFAULT 0,
               PRIMARY KEY(name, item_id)
             );
             CREATE TABLE IF NOT EXISTS workflows (
               id TEXT PRIMARY KEY,
               workflow_json TEXT NOT NULL,
               updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS hotkeys (
               accelerator TEXT PRIMARY KEY COLLATE NOCASE,
               target_id TEXT NOT NULL
             );",
        )?;
        Ok(())
    }

    pub fn upsert_catalog(&mut self, items: &[CatalogItem]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        let now = now_unix();
        {
            let mut statement = transaction.prepare(
                "INSERT INTO catalog_items(id, item_json, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(id) DO UPDATE SET item_json=excluded.item_json, updated_at=excluded.updated_at",
            )?;
            for item in items {
                statement.execute(params![item.id, serde_json::to_string(item)?, now])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn catalog_count(&self) -> Result<usize> {
        let count: i64 =
            self.connection
                .query_row("SELECT COUNT(*) FROM catalog_items", [], |row| row.get(0))?;
        Ok(count.max(0) as usize)
    }

    pub fn get_item(&self, item_id: &str) -> Result<Option<CatalogItem>> {
        let json: Option<String> = self
            .connection
            .query_row(
                "SELECT item_json FROM catalog_items WHERE id=?1",
                [item_id],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|json| self.hydrate_item(serde_json::from_str(&json)?))
            .transpose()
    }

    pub fn list_catalog(&self) -> Result<Vec<CatalogItem>> {
        let mut statement = self
            .connection
            .prepare("SELECT item_json FROM catalog_items")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| {
            let item: CatalogItem = serde_json::from_str(&row?)?;
            self.hydrate_item(item)
        })
        .collect()
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<CatalogItem>> {
        Ok(rank_items(query, self.list_catalog()?, limit.clamp(1, 200)))
    }

    fn hydrate_item(&self, mut item: CatalogItem) -> Result<CatalogItem> {
        let preference = self
            .connection
            .query_row(
                "SELECT favorite, pinned, usage_count, last_used_at FROM personalization WHERE item_id=?1",
                [&item.id],
                |row| {
                    Ok((
                        row.get::<_, bool>(0)?,
                        row.get::<_, bool>(1)?,
                        row.get::<_, i64>(2)?.max(0) as u64,
                        row.get::<_, Option<i64>>(3)?,
                    ))
                },
            )
            .optional()?;
        if let Some((favorite, pinned, usage_count, last_used_at)) = preference {
            item.favorite = favorite;
            item.pinned = pinned;
            item.usage_count = usage_count;
            item.last_used_at = last_used_at;
        }
        let mut statement = self
            .connection
            .prepare("SELECT alias FROM aliases WHERE item_id=?1 ORDER BY alias")?;
        let aliases = statement
            .query_map([&item.id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for alias in aliases {
            if !item
                .aliases
                .iter()
                .any(|known| known.eq_ignore_ascii_case(&alias))
            {
                item.aliases.push(alias);
            }
        }
        let mut statement = self
            .connection
            .prepare("SELECT tag FROM custom_tags WHERE item_id=?1 ORDER BY tag")?;
        let tags = statement
            .query_map([&item.id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for tag in tags {
            if !item
                .tags
                .iter()
                .any(|known| known.eq_ignore_ascii_case(&tag))
            {
                item.tags.push(tag);
            }
        }
        Ok(item)
    }

    pub fn set_favorite(&self, item_id: &str, favorite: bool) -> Result<()> {
        self.ensure_preference(item_id)?;
        self.connection.execute(
            "UPDATE personalization SET favorite=?2 WHERE item_id=?1",
            params![item_id, favorite],
        )?;
        Ok(())
    }

    pub fn set_pinned(&self, item_id: &str, pinned: bool) -> Result<()> {
        self.ensure_preference(item_id)?;
        self.connection.execute(
            "UPDATE personalization SET pinned=?2 WHERE item_id=?1",
            params![item_id, pinned],
        )?;
        Ok(())
    }

    pub fn record_usage(&self, item_id: &str) -> Result<()> {
        self.ensure_preference(item_id)?;
        self.connection.execute(
            "UPDATE personalization SET usage_count=usage_count+1, last_used_at=?2 WHERE item_id=?1",
            params![item_id, now_unix()],
        )?;
        Ok(())
    }

    fn ensure_preference(&self, item_id: &str) -> Result<()> {
        self.connection.execute(
            "INSERT OR IGNORE INTO personalization(item_id) VALUES (?1)",
            [item_id],
        )?;
        Ok(())
    }

    pub fn add_alias(&self, item_id: &str, alias: &str) -> Result<()> {
        let alias = alias.trim();
        anyhow::ensure!(!alias.is_empty(), "alias cannot be empty");
        self.connection.execute(
            "INSERT OR IGNORE INTO aliases(item_id, alias) VALUES (?1, ?2)",
            params![item_id, alias],
        )?;
        Ok(())
    }

    pub fn remove_alias(&self, item_id: &str, alias: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM aliases WHERE item_id=?1 AND alias=?2 COLLATE NOCASE",
            params![item_id, alias.trim()],
        )?;
        Ok(())
    }

    pub fn add_tag(&self, item_id: &str, tag: &str) -> Result<()> {
        let tag = tag.trim();
        anyhow::ensure!(!tag.is_empty(), "tag cannot be empty");
        self.connection.execute(
            "INSERT OR IGNORE INTO custom_tags(item_id, tag) VALUES (?1, ?2)",
            params![item_id, tag],
        )?;
        Ok(())
    }

    pub fn remove_tag(&self, item_id: &str, tag: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM custom_tags WHERE item_id=?1 AND tag=?2 COLLATE NOCASE",
            params![item_id, tag.trim()],
        )?;
        Ok(())
    }

    pub fn create_collection(&self, name: &str) -> Result<()> {
        let name = name.trim();
        anyhow::ensure!(!name.is_empty(), "collection name cannot be empty");
        self.connection.execute(
            "INSERT OR IGNORE INTO collections(name, item_id, sort_order) VALUES (?1, '', -1)",
            [name],
        )?;
        Ok(())
    }

    pub fn add_to_collection(&self, name: &str, item_id: &str) -> Result<()> {
        anyhow::ensure!(!name.trim().is_empty(), "collection name cannot be empty");
        anyhow::ensure!(!item_id.trim().is_empty(), "item id cannot be empty");
        self.connection.execute(
            "DELETE FROM collections WHERE name=?1 COLLATE NOCASE AND item_id=''",
            [name],
        )?;
        let order: i64 = self.connection.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM collections WHERE name=?1 COLLATE NOCASE",
            [name],
            |row| row.get(0),
        )?;
        self.connection.execute(
            "INSERT OR IGNORE INTO collections(name, item_id, sort_order) VALUES (?1, ?2, ?3)",
            params![name, item_id, order],
        )?;
        Ok(())
    }

    pub fn remove_from_collection(&self, name: &str, item_id: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM collections WHERE name=?1 COLLATE NOCASE AND item_id=?2",
            params![name, item_id],
        )?;
        if !self.collection_exists(name)? {
            self.create_collection(name)?;
        }
        Ok(())
    }

    pub fn delete_collection(&self, name: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM collections WHERE name=?1 COLLATE NOCASE",
            [name],
        )?;
        Ok(())
    }

    fn collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM collections WHERE name=?1 COLLATE NOCASE)",
            [name],
            |row| row.get(0),
        )?)
    }

    pub fn list_collections(&self) -> Result<Vec<CollectionSummary>> {
        let mut names = self
            .connection
            .prepare("SELECT DISTINCT name FROM collections ORDER BY name COLLATE NOCASE")?;
        let names = names
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        names
            .into_iter()
            .map(|name| {
                let mut items = self.connection.prepare(
                    "SELECT item_id FROM collections
                     WHERE name=?1 COLLATE NOCASE AND item_id<>'' ORDER BY sort_order",
                )?;
                let item_ids = items
                    .query_map([&name], |row| row.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CollectionSummary { name, item_ids })
            })
            .collect()
    }

    pub fn save_workflow(&self, workflow: &WorkflowDefinition) -> Result<()> {
        validate_workflow(workflow)?;
        self.connection.execute(
            "INSERT INTO workflows(id, workflow_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET workflow_json=excluded.workflow_json, updated_at=excluded.updated_at",
            params![workflow.id, serde_json::to_string(workflow)?, now_unix()],
        )?;
        let catalog_item = CatalogItem {
            id: format!("workflow:{}", workflow.id),
            kind: ItemKind::Workflow,
            source: ItemSource::User,
            name: workflow.name.clone(),
            aliases: Vec::new(),
            categories: vec!["Workflows".into()],
            tags: Vec::new(),
            browser_path: Vec::new(),
            compatible_tracks: Vec::new(),
            favorite: false,
            pinned: false,
            usage_count: 0,
            last_used_at: None,
        };
        self.connection.execute(
            "INSERT INTO catalog_items(id, item_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET item_json=excluded.item_json, updated_at=excluded.updated_at",
            params![catalog_item.id, serde_json::to_string(&catalog_item)?, now_unix()],
        )?;
        Ok(())
    }

    pub fn get_workflow(&self, workflow_id: &str) -> Result<Option<WorkflowDefinition>> {
        let json: Option<String> = self
            .connection
            .query_row(
                "SELECT workflow_json FROM workflows WHERE id=?1",
                [workflow_id],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|json| Ok(serde_json::from_str(&json)?))
            .transpose()
    }

    pub fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>> {
        let mut statement = self
            .connection
            .prepare("SELECT workflow_json FROM workflows ORDER BY id")?;
        statement
            .query_map([], |row| row.get::<_, String>(0))?
            .map(|row| Ok(serde_json::from_str(&row?)?))
            .collect()
    }

    pub fn delete_workflow(&self, workflow_id: &str) -> Result<()> {
        self.connection
            .execute("DELETE FROM workflows WHERE id=?1", [workflow_id])?;
        self.connection.execute(
            "DELETE FROM catalog_items WHERE id=?1",
            [format!("workflow:{workflow_id}")],
        )?;
        Ok(())
    }

    pub fn set_hotkey(&self, accelerator: &str, target_id: &str) -> Result<()> {
        let accelerator = accelerator.trim();
        let target_id = target_id.trim();
        anyhow::ensure!(!accelerator.is_empty(), "accelerator cannot be empty");
        anyhow::ensure!(!target_id.is_empty(), "target id cannot be empty");
        self.connection.execute(
            "INSERT INTO hotkeys(accelerator, target_id) VALUES (?1, ?2)
             ON CONFLICT(accelerator) DO UPDATE SET target_id=excluded.target_id",
            params![accelerator, target_id],
        )?;
        Ok(())
    }

    pub fn clear_hotkey(&self, accelerator: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM hotkeys WHERE accelerator=?1 COLLATE NOCASE",
            [accelerator.trim()],
        )?;
        Ok(())
    }

    pub fn list_hotkeys(&self) -> Result<Vec<HotkeyBinding>> {
        let mut statement = self.connection.prepare(
            "SELECT accelerator, target_id FROM hotkeys ORDER BY accelerator COLLATE NOCASE",
        )?;
        statement
            .query_map([], |row| {
                Ok(HotkeyBinding {
                    accelerator: row.get(0)?,
                    target_id: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette_protocol::{InsertionPosition, ItemKind, ItemSource, WorkflowAction};

    fn item() -> CatalogItem {
        CatalogItem {
            id: "browser:auto-filter".into(),
            kind: ItemKind::NativeDevice,
            source: ItemSource::LiveBrowser,
            name: "Auto Filter".into(),
            aliases: vec![],
            categories: vec!["Audio Effects".into()],
            tags: vec!["filter".into()],
            browser_path: vec!["Audio Effects".into(), "Auto Filter".into()],
            compatible_tracks: vec![],
            favorite: false,
            pinned: false,
            usage_count: 0,
            last_used_at: None,
        }
    }

    #[test]
    fn catalog_and_personalization_round_trip() {
        let mut store = Store::in_memory().unwrap();
        store.upsert_catalog(&[item()]).unwrap();
        store.add_alias("browser:auto-filter", "af").unwrap();
        store.set_favorite("browser:auto-filter", true).unwrap();
        store.record_usage("browser:auto-filter").unwrap();
        let result = store.search("af", 10).unwrap().remove(0);
        assert!(result.favorite);
        assert_eq!(result.usage_count, 1);
        assert!(result.aliases.contains(&"af".into()));
    }

    #[test]
    fn workflows_are_validated_and_persisted() {
        let store = Store::in_memory().unwrap();
        let workflow = WorkflowDefinition {
            id: "vocal".into(),
            name: "Vocal Chain".into(),
            description: String::new(),
            actions: vec![WorkflowAction::InsertNative {
                name: "EQ Eight".into(),
                position: InsertionPosition::End,
            }],
        };
        store.save_workflow(&workflow).unwrap();
        assert_eq!(store.get_workflow("vocal").unwrap(), Some(workflow));
        assert_eq!(store.list_workflows().unwrap().len(), 1);
        assert_eq!(
            store.search("Vocal Chain", 10).unwrap()[0].kind,
            ItemKind::Workflow
        );
    }

    #[test]
    fn tags_collections_and_hotkeys_round_trip() {
        let mut store = Store::in_memory().unwrap();
        store.upsert_catalog(&[item()]).unwrap();
        store.add_tag("browser:auto-filter", "go-to").unwrap();
        assert!(
            store.search("go-to", 10).unwrap()[0]
                .tags
                .contains(&"go-to".into())
        );

        store.create_collection("Mixing").unwrap();
        store
            .add_to_collection("Mixing", "browser:auto-filter")
            .unwrap();
        assert_eq!(
            store.list_collections().unwrap()[0].item_ids,
            vec!["browser:auto-filter"]
        );

        store
            .set_hotkey("cmd+shift+f", "browser:auto-filter")
            .unwrap();
        assert_eq!(
            store.list_hotkeys().unwrap()[0].target_id,
            "browser:auto-filter"
        );
    }
}
