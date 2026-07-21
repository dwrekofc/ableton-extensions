use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_DAEMON_PORT: u16 = 49_371;
pub const MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerRole {
    Client,
    Bridge,
    Extension,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerTarget {
    Daemon,
    Bridge,
    Extension,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsertionPosition {
    Beginning,
    BeforeSelected,
    AfterSelected,
    End,
}

impl fmt::Display for InsertionPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Beginning => "beginning",
            Self::BeforeSelected => "before_selected",
            Self::AfterSelected => "after_selected",
            Self::End => "end",
        };
        f.write_str(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackKind {
    Audio,
    Midi,
    Return,
    Main,
    Group,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    NativeDevice,
    Plugin,
    Preset,
    Rack,
    MaxDevice,
    Sample,
    Loop,
    Command,
    Workflow,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemSource {
    LiveBrowser,
    LiveDatabase,
    OfficialExtension,
    User,
    BuiltIn,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub index: usize,
    pub name: String,
    pub class_name: Option<String>,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveContext {
    pub live_version: Option<String>,
    pub set_name: Option<String>,
    pub track_name: String,
    pub track_kind: TrackKind,
    pub track_index: Option<usize>,
    pub selected_device_index: Option<usize>,
    pub selected_device_name: Option<String>,
    pub frozen: bool,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInstance {
    pub track_name: String,
    pub track_kind: TrackKind,
    pub track_index: Option<usize>,
    pub device_name: String,
    pub class_name: Option<String>,
    pub class_display_name: Option<String>,
    pub active: Option<bool>,
    #[serde(default)]
    pub device_indices: Vec<usize>,
    #[serde(default)]
    pub chain_names: Vec<String>,
    #[serde(default)]
    pub device_path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInventory {
    pub live_version: Option<String>,
    pub set_name: Option<String>,
    pub query: String,
    pub instances: Vec<DeviceInstance>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: String,
    pub kind: ItemKind,
    pub source: ItemSource,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub browser_path: Vec<String>,
    #[serde(default)]
    pub compatible_tracks: Vec<TrackKind>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub usage_count: u64,
    pub last_used_at: Option<i64>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowAction {
    LoadItem {
        item_id: String,
        #[serde(default)]
        browser_path: Vec<String>,
        position: InsertionPosition,
    },
    InsertNative {
        name: String,
        position: InsertionPosition,
    },
    SetParameter {
        device_name: String,
        parameter_name: String,
        value: f64,
    },
    RenameTrack {
        name: String,
    },
    SetTrackArm {
        armed: bool,
    },
    SetTrackMute {
        muted: bool,
    },
    SetTrackSolo {
        soloed: bool,
    },
    CreateAudioTrack,
    CreateMidiTrack,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub actions: Vec<WorkflowAction>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    pub index: usize,
    pub action_type: String,
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostics {
    pub component: String,
    pub version: String,
    pub live_version: Option<String>,
    pub capabilities: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub details: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub protocol_version: u32,
    pub bridge_connected: bool,
    pub extension_connected: bool,
    pub catalog_count: usize,
    pub data_directory: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionSummary {
    pub name: String,
    pub item_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub accelerator: String,
    pub target_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveDatabaseImportSummary {
    pub plugin_database: String,
    pub schema_version: i64,
    pub discovered: usize,
    pub imported: usize,
    pub instruments: usize,
    pub audio_effects: usize,
    pub vendors: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogScanSummary {
    pub scanned: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum RequestKind {
    Ping,
    Status,
    GetContext,
    InspectDevices {
        query: String,
    },
    ScanCatalog {
        max_items: usize,
        max_depth: usize,
    },
    Search {
        query: String,
        limit: usize,
    },
    ImportLiveDatabase {
        plugin_database: Option<String>,
    },
    LoadItem {
        item_id: String,
        #[serde(default)]
        browser_path: Vec<String>,
        position: InsertionPosition,
    },
    ResolveAndLoadItem {
        item_id: String,
        name: String,
        kind: ItemKind,
        position: InsertionPosition,
    },
    InsertNative {
        name: String,
        position: InsertionPosition,
    },
    SdkInsertNative {
        name: String,
        position: InsertionPosition,
    },
    SetFavorite {
        item_id: String,
        favorite: bool,
    },
    SetPinned {
        item_id: String,
        pinned: bool,
    },
    AddAlias {
        item_id: String,
        alias: String,
    },
    RemoveAlias {
        item_id: String,
        alias: String,
    },
    AddTag {
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
        workflow: WorkflowDefinition,
    },
    DeleteWorkflow {
        workflow_id: String,
    },
    ListWorkflows,
    RunWorkflow {
        workflow_id: String,
    },
    ExecuteWorkflow {
        workflow: WorkflowDefinition,
    },
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

impl RequestKind {
    pub fn target(&self) -> PeerTarget {
        match self {
            Self::GetContext
            | Self::InspectDevices { .. }
            | Self::ScanCatalog { .. }
            | Self::LoadItem { .. }
            | Self::ResolveAndLoadItem { .. }
            | Self::InsertNative { .. }
            | Self::ExecuteWorkflow { .. }
            | Self::Diagnostics => PeerTarget::Bridge,
            Self::SdkInsertNative { .. } => PeerTarget::Extension,
            _ => PeerTarget::Daemon,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum ResponseData {
    Ack,
    Pong,
    Status(ServiceStatus),
    Context(LiveContext),
    DeviceInventory(DeviceInventory),
    Catalog(Vec<CatalogItem>),
    CatalogScan(CatalogScanSummary),
    SearchResults(Vec<CatalogItem>),
    LiveDatabaseImport(LiveDatabaseImportSummary),
    Workflow(WorkflowDefinition),
    Workflows(Vec<WorkflowDefinition>),
    Collections(Vec<CollectionSummary>),
    Hotkeys(Vec<HotkeyBinding>),
    ActionResults(Vec<ActionResult>),
    Diagnostics(Diagnostics),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub retryable: bool,
}

impl ProtocolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireMessage {
    Hello {
        protocol_version: u32,
        role: PeerRole,
        token: String,
        peer_name: String,
        peer_version: String,
        #[serde(default)]
        capabilities: Vec<String>,
    },
    HelloAck {
        protocol_version: u32,
        daemon_version: String,
    },
    Request {
        request_id: String,
        request: RequestKind,
    },
    Response {
        request_id: String,
        result: Option<ResponseData>,
        error: Option<ProtocolError>,
    },
    Event {
        event: String,
        #[serde(default)]
        data: Value,
    },
}

impl WireMessage {
    pub fn success(request_id: impl Into<String>, result: ResponseData) -> Self {
        Self::Response {
            request_id: request_id.into(),
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(request_id: impl Into<String>, error: ProtocolError) -> Self {
        Self::Response {
            request_id: request_id.into(),
            result: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_round_trip_is_tagged_and_stable() {
        let message = WireMessage::Request {
            request_id: "r1".into(),
            request: RequestKind::LoadItem {
                item_id: "browser:reverb".into(),
                browser_path: vec!["Audio Effects".into(), "Reverb".into()],
                position: InsertionPosition::AfterSelected,
            },
        };
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"type\":\"request\""));
        assert!(json.contains("\"method\":\"load_item\""));
        assert_eq!(serde_json::from_str::<WireMessage>(&json).unwrap(), message);
    }

    #[test]
    fn routing_targets_are_explicit() {
        assert_eq!(RequestKind::GetContext.target(), PeerTarget::Bridge);
        assert_eq!(RequestKind::Status.target(), PeerTarget::Daemon);
        assert_eq!(
            RequestKind::SdkInsertNative {
                name: "Reverb".into(),
                position: InsertionPosition::End,
            }
            .target(),
            PeerTarget::Extension
        );
    }
}
