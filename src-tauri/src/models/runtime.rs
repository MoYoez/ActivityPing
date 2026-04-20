use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::config::DiscordRichPresenceButtonConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedTextEntry {
    pub text: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReporterActivity {
    #[serde(default)]
    pub process_name: String,
    #[serde(default)]
    pub process_title: Option<String>,
    #[serde(default)]
    pub raw_process_title: Option<String>,
    #[serde(default)]
    pub media_title: Option<String>,
    #[serde(default)]
    pub media_artist: Option<String>,
    #[serde(default)]
    pub media_album: Option<String>,
    #[serde(default)]
    pub media_summary: Option<String>,
    #[serde(default)]
    pub media_duration_ms: Option<u64>,
    #[serde(default)]
    pub media_position_ms: Option<u64>,
    #[serde(default)]
    pub play_source: Option<String>,
    #[serde(default)]
    pub status_text: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterLogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub title: String,
    pub detail: String,
    #[serde(default)]
    pub title_key: Option<String>,
    #[serde(default)]
    pub title_params: Option<Value>,
    #[serde(default)]
    pub detail_key: Option<String>,
    #[serde(default)]
    pub detail_params: Option<Value>,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeReporterSnapshot {
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub logs: Vec<ReporterLogEntry>,
    #[serde(default)]
    pub current_activity: Option<ReporterActivity>,
    #[serde(default)]
    pub last_heartbeat_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordPresenceSnapshot {
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub current_summary: Option<String>,
    #[serde(default)]
    pub debug_payload: Option<DiscordDebugPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordDebugPayload {
    #[serde(default)]
    pub activity_name: Option<String>,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub report_mode_applied: String,
    #[serde(default)]
    pub activity_type: String,
    #[serde(default)]
    pub status_display_type: Option<String>,
    #[serde(default)]
    pub started_at_millis: Option<i64>,
    #[serde(default)]
    pub ended_at_millis: Option<i64>,
    #[serde(default)]
    pub media_duration_ms: Option<u64>,
    #[serde(default)]
    pub media_position_ms: Option<u64>,
    #[serde(default)]
    pub app_icon_url: Option<String>,
    #[serde(default)]
    pub app_icon_text: Option<String>,
    #[serde(default)]
    pub app_icon_error: Option<String>,
    #[serde(default)]
    pub artwork_url: Option<String>,
    #[serde(default)]
    pub artwork_hover_text: Option<String>,
    #[serde(default)]
    pub artwork_content_type: Option<String>,
    #[serde(default)]
    pub artwork_upload_error: Option<String>,
    #[serde(default)]
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    #[serde(default)]
    pub party: Option<DiscordDebugParty>,
    #[serde(default)]
    pub secrets: Option<DiscordDebugSecrets>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordDebugParty {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub size: Option<[i32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordDebugSecrets {
    #[serde(default)]
    pub join: Option<String>,
    #[serde(default)]
    pub spectate: Option<String>,
    #[serde(default)]
    pub match_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformProbeResult {
    pub success: bool,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub guidance: Vec<String>,
    #[serde(default)]
    pub summary_key: Option<String>,
    #[serde(default)]
    pub summary_params: Option<Value>,
    #[serde(default)]
    pub detail_key: Option<String>,
    #[serde(default)]
    pub detail_params: Option<Value>,
    #[serde(default)]
    pub guidance_entries: Vec<LocalizedTextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSelfTestResult {
    pub platform: String,
    pub foreground: PlatformProbeResult,
    pub window_title: PlatformProbeResult,
    pub media: PlatformProbeResult,
}
