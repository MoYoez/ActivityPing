use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn default_poll_interval_ms() -> u64 {
    5_000
}

pub fn default_heartbeat_interval_ms() -> u64 {
    60_000
}

pub fn default_report_foreground_app() -> bool {
    true
}

pub fn default_report_window_title() -> bool {
    true
}

pub fn default_report_media() -> bool {
    true
}

pub fn default_report_stopped_media() -> bool {
    false
}

pub fn default_report_play_source() -> bool {
    true
}

pub fn default_discord_application_id() -> String {
    String::new()
}

pub fn default_discord_details_format() -> String {
    "{activity}".into()
}

pub fn default_discord_state_format() -> String {
    "{context}".into()
}

pub fn default_discord_activity_type() -> DiscordActivityType {
    DiscordActivityType::default()
}

pub fn default_discord_status_display() -> DiscordStatusDisplay {
    DiscordStatusDisplay::Name
}

pub fn default_discord_app_name_mode() -> DiscordAppNameMode {
    DiscordAppNameMode::Default
}

pub fn default_discord_music_app_name_mode() -> DiscordAppNameMode {
    DiscordAppNameMode::Source
}

pub fn default_discord_custom_app_name() -> String {
    String::new()
}

pub fn default_discord_smart_enable_music_countdown() -> bool {
    true
}

pub fn default_discord_smart_show_app_name() -> bool {
    false
}

pub fn default_discord_use_app_artwork() -> bool {
    false
}

pub fn default_discord_use_music_artwork() -> bool {
    false
}

pub fn default_discord_artwork_worker_upload_url() -> String {
    String::new()
}

pub fn default_discord_artwork_worker_token() -> String {
    String::new()
}

pub fn default_discord_use_custom_addons_override() -> bool {
    false
}

pub fn default_capture_reported_apps_enabled() -> bool {
    true
}

pub fn default_capture_history_record_limit() -> u32 {
    3
}

pub fn default_capture_history_title_limit() -> u32 {
    3
}

pub fn default_app_message_rules_show_process_name() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppTitleRuleMode {
    #[default]
    Plain,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppFilterMode {
    #[default]
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordReportMode {
    Music,
    App,
    #[default]
    Mixed,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordActivityType {
    #[default]
    Playing,
    Listening,
    Watching,
    Competing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiscordStatusDisplay {
    Name,
    State,
    Details,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiscordAppNameMode {
    Default,
    Song,
    Artist,
    Album,
    #[serde(rename = "source", alias = "media_source")]
    Source,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppMessageTitleRule {
    #[serde(default)]
    pub mode: AppTitleRuleMode,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppMessageRuleGroup {
    #[serde(default)]
    pub process_match: String,
    #[serde(default)]
    pub default_text: String,
    #[serde(default)]
    pub title_rules: Vec<AppMessageTitleRule>,
    #[serde(default)]
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    #[serde(default)]
    pub party_id: String,
    #[serde(default)]
    pub party_size_current: Option<u32>,
    #[serde(default)]
    pub party_size_max: Option<u32>,
    #[serde(default)]
    pub join_secret: String,
    #[serde(default)]
    pub spectate_secret: String,
    #[serde(default)]
    pub match_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscordRichPresenceButtonConfig {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordCustomPreset {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_discord_activity_type")]
    pub activity_type: DiscordActivityType,
    #[serde(default = "default_discord_status_display")]
    pub status_display: DiscordStatusDisplay,
    #[serde(default = "default_discord_app_name_mode")]
    pub app_name_mode: DiscordAppNameMode,
    #[serde(default = "default_discord_custom_app_name")]
    pub custom_app_name: String,
    #[serde(default)]
    pub details_format: String,
    #[serde(default)]
    pub state_format: String,
    #[serde(default)]
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    #[serde(default)]
    pub party_id: String,
    #[serde(default)]
    pub party_size_current: Option<u32>,
    #[serde(default)]
    pub party_size_max: Option<u32>,
    #[serde(default)]
    pub join_secret: String,
    #[serde(default)]
    pub spectate_secret: String,
    #[serde(default)]
    pub match_secret: String,
}

impl Default for DiscordCustomPreset {
    fn default() -> Self {
        Self {
            name: String::new(),
            activity_type: default_discord_activity_type(),
            status_display: default_discord_status_display(),
            app_name_mode: default_discord_app_name_mode(),
            custom_app_name: default_discord_custom_app_name(),
            details_format: String::new(),
            state_format: String::new(),
            buttons: Vec::new(),
            party_id: String::new(),
            party_size_current: None,
            party_size_max: None,
            join_secret: String::new(),
            spectate_secret: String::new(),
            match_secret: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    pub realtime_reporter: bool,
    pub tray: bool,
    pub platform_self_test: bool,
    pub discord_presence: bool,
    pub autostart: bool,
}

#[cfg(desktop)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: true,
        tray: true,
        platform_self_test: true,
        discord_presence: true,
        autostart: true,
    }
}

#[cfg(mobile)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
        discord_presence: false,
        autostart: false,
    }
}

#[cfg(not(any(desktop, mobile)))]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
        discord_presence: false,
        autostart: false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_heartbeat_interval_ms")]
    pub heartbeat_interval_ms: u64,
    #[serde(default)]
    pub runtime_autostart_enabled: bool,
    #[serde(default, rename = "reporterEnabled", skip_serializing)]
    pub legacy_reporter_enabled: bool,
    #[serde(default = "default_report_foreground_app")]
    pub report_foreground_app: bool,
    #[serde(default = "default_report_window_title")]
    pub report_window_title: bool,
    #[serde(default = "default_report_media")]
    pub report_media: bool,
    #[serde(default = "default_report_stopped_media")]
    pub report_stopped_media: bool,
    #[serde(default = "default_report_play_source")]
    pub report_play_source: bool,
    #[serde(default, rename = "discordEnabled", skip_serializing)]
    pub legacy_discord_enabled: bool,
    #[serde(default = "default_discord_application_id")]
    pub discord_application_id: String,
    #[serde(default)]
    pub discord_report_mode: DiscordReportMode,
    #[serde(default = "default_discord_activity_type")]
    pub discord_activity_type: DiscordActivityType,
    #[serde(default, alias = "discordStatusDisplay", skip_serializing)]
    pub legacy_discord_status_display: Option<DiscordStatusDisplay>,
    #[serde(default, alias = "discordAppNameMode", skip_serializing)]
    pub legacy_discord_app_name_mode: Option<DiscordAppNameMode>,
    #[serde(default, alias = "discordCustomAppName", skip_serializing)]
    pub legacy_discord_custom_app_name: Option<String>,
    #[serde(default = "default_discord_status_display")]
    pub discord_smart_status_display: DiscordStatusDisplay,
    #[serde(default = "default_discord_app_name_mode")]
    pub discord_smart_app_name_mode: DiscordAppNameMode,
    #[serde(default = "default_discord_custom_app_name")]
    pub discord_smart_custom_app_name: String,
    #[serde(default = "default_discord_status_display")]
    pub discord_music_status_display: DiscordStatusDisplay,
    #[serde(default = "default_discord_music_app_name_mode")]
    pub discord_music_app_name_mode: DiscordAppNameMode,
    #[serde(default = "default_discord_custom_app_name")]
    pub discord_music_custom_app_name: String,
    #[serde(default = "default_discord_status_display")]
    pub discord_app_status_display: DiscordStatusDisplay,
    #[serde(default = "default_discord_app_name_mode")]
    pub discord_app_app_name_mode: DiscordAppNameMode,
    #[serde(default = "default_discord_custom_app_name")]
    pub discord_app_custom_app_name: String,
    #[serde(default = "default_discord_status_display")]
    pub discord_custom_mode_status_display: DiscordStatusDisplay,
    #[serde(default = "default_discord_app_name_mode")]
    pub discord_custom_mode_app_name_mode: DiscordAppNameMode,
    #[serde(default = "default_discord_custom_app_name")]
    pub discord_custom_mode_custom_app_name: String,
    #[serde(default = "default_discord_smart_enable_music_countdown")]
    pub discord_smart_enable_music_countdown: bool,
    #[serde(default = "default_discord_smart_show_app_name")]
    pub discord_smart_show_app_name: bool,
    #[serde(default, rename = "discordUseMediaArtwork", skip_serializing)]
    pub legacy_discord_use_media_artwork: bool,
    #[serde(default = "default_discord_use_app_artwork")]
    pub discord_use_app_artwork: bool,
    #[serde(default = "default_discord_use_music_artwork")]
    pub discord_use_music_artwork: bool,
    #[serde(default = "default_discord_artwork_worker_upload_url")]
    pub discord_artwork_worker_upload_url: String,
    #[serde(default = "default_discord_artwork_worker_token")]
    pub discord_artwork_worker_token: String,
    #[serde(default = "default_discord_details_format")]
    pub discord_details_format: String,
    #[serde(default = "default_discord_state_format")]
    pub discord_state_format: String,
    #[serde(default)]
    pub discord_custom_buttons: Vec<DiscordRichPresenceButtonConfig>,
    #[serde(default)]
    pub discord_custom_party_id: String,
    #[serde(default)]
    pub discord_custom_party_size_current: Option<u32>,
    #[serde(default)]
    pub discord_custom_party_size_max: Option<u32>,
    #[serde(default)]
    pub discord_custom_join_secret: String,
    #[serde(default)]
    pub discord_custom_spectate_secret: String,
    #[serde(default)]
    pub discord_custom_match_secret: String,
    #[serde(default = "default_discord_use_custom_addons_override")]
    pub discord_use_custom_addons_override: bool,
    #[serde(default, alias = "discordCustomRules")]
    pub discord_custom_presets: Vec<DiscordCustomPreset>,
    #[serde(default)]
    pub launch_on_startup: bool,
    #[serde(default = "default_capture_reported_apps_enabled")]
    pub capture_reported_apps_enabled: bool,
    #[serde(default = "default_capture_history_record_limit")]
    pub capture_history_record_limit: u32,
    #[serde(default = "default_capture_history_title_limit")]
    pub capture_history_title_limit: u32,
    #[serde(default)]
    pub app_message_rules: Vec<AppMessageRuleGroup>,
    #[serde(default = "default_app_message_rules_show_process_name")]
    pub app_message_rules_show_process_name: bool,
    #[serde(default)]
    pub app_filter_mode: AppFilterMode,
    #[serde(default)]
    pub app_blacklist: Vec<String>,
    #[serde(default)]
    pub app_whitelist: Vec<String>,
    #[serde(default)]
    pub app_name_only_list: Vec<String>,
    #[serde(default)]
    pub media_play_source_blocklist: Vec<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: default_poll_interval_ms(),
            heartbeat_interval_ms: default_heartbeat_interval_ms(),
            runtime_autostart_enabled: false,
            legacy_reporter_enabled: false,
            report_foreground_app: default_report_foreground_app(),
            report_window_title: default_report_window_title(),
            report_media: default_report_media(),
            report_stopped_media: default_report_stopped_media(),
            report_play_source: default_report_play_source(),
            legacy_discord_enabled: false,
            discord_application_id: default_discord_application_id(),
            discord_report_mode: DiscordReportMode::default(),
            discord_activity_type: default_discord_activity_type(),
            legacy_discord_status_display: None,
            legacy_discord_app_name_mode: None,
            legacy_discord_custom_app_name: None,
            discord_smart_status_display: default_discord_status_display(),
            discord_smart_app_name_mode: default_discord_app_name_mode(),
            discord_smart_custom_app_name: default_discord_custom_app_name(),
            discord_music_status_display: default_discord_status_display(),
            discord_music_app_name_mode: default_discord_music_app_name_mode(),
            discord_music_custom_app_name: default_discord_custom_app_name(),
            discord_app_status_display: default_discord_status_display(),
            discord_app_app_name_mode: default_discord_app_name_mode(),
            discord_app_custom_app_name: default_discord_custom_app_name(),
            discord_custom_mode_status_display: default_discord_status_display(),
            discord_custom_mode_app_name_mode: default_discord_app_name_mode(),
            discord_custom_mode_custom_app_name: default_discord_custom_app_name(),
            discord_smart_enable_music_countdown: default_discord_smart_enable_music_countdown(),
            discord_smart_show_app_name: default_discord_smart_show_app_name(),
            legacy_discord_use_media_artwork: false,
            discord_use_app_artwork: default_discord_use_app_artwork(),
            discord_use_music_artwork: default_discord_use_music_artwork(),
            discord_artwork_worker_upload_url: default_discord_artwork_worker_upload_url(),
            discord_artwork_worker_token: default_discord_artwork_worker_token(),
            discord_details_format: default_discord_details_format(),
            discord_state_format: default_discord_state_format(),
            discord_custom_buttons: Vec::new(),
            discord_custom_party_id: String::new(),
            discord_custom_party_size_current: None,
            discord_custom_party_size_max: None,
            discord_custom_join_secret: String::new(),
            discord_custom_spectate_secret: String::new(),
            discord_custom_match_secret: String::new(),
            discord_use_custom_addons_override: default_discord_use_custom_addons_override(),
            discord_custom_presets: Vec::new(),
            launch_on_startup: false,
            capture_reported_apps_enabled: default_capture_reported_apps_enabled(),
            capture_history_record_limit: default_capture_history_record_limit(),
            capture_history_title_limit: default_capture_history_title_limit(),
            app_message_rules: Vec::new(),
            app_message_rules_show_process_name: default_app_message_rules_show_process_name(),
            app_filter_mode: AppFilterMode::default(),
            app_blacklist: Vec::new(),
            app_whitelist: Vec::new(),
            app_name_only_list: Vec::new(),
            media_play_source_blocklist: Vec::new(),
        }
    }
}

impl Default for DiscordStatusDisplay {
    fn default() -> Self {
        Self::Name
    }
}

impl Default for DiscordAppNameMode {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppStatePayload {
    #[serde(default)]
    pub config: ClientConfig,
    #[serde(default, deserialize_with = "deserialize_app_history")]
    pub app_history: Vec<AppHistoryEntry>,
    #[serde(default, deserialize_with = "deserialize_play_source_history")]
    pub play_source_history: Vec<PlaySourceHistoryEntry>,
    #[serde(default)]
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppHistoryEntry {
    #[serde(default)]
    pub process_name: String,
    #[serde(default)]
    pub process_title: Option<String>,
    #[serde(default)]
    pub process_titles: Vec<String>,
    #[serde(default)]
    pub status_text: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlaySourceHistoryEntry {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub media_title: Option<String>,
    #[serde(default)]
    pub media_artist: Option<String>,
    #[serde(default)]
    pub media_album: Option<String>,
    #[serde(default)]
    pub media_summary: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AppHistoryEntryCompat {
    Entry(AppHistoryEntry),
    Name(String),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PlaySourceHistoryEntryCompat {
    Entry(PlaySourceHistoryEntry),
    Source(String),
}

fn deserialize_app_history<'de, D>(deserializer: D) -> Result<Vec<AppHistoryEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Vec::<AppHistoryEntryCompat>::deserialize(deserializer)?;
    Ok(values
        .into_iter()
        .map(|value| match value {
            AppHistoryEntryCompat::Entry(entry) => entry,
            AppHistoryEntryCompat::Name(process_name) => AppHistoryEntry {
                process_name,
                ..AppHistoryEntry::default()
            },
        })
        .collect())
}

fn deserialize_play_source_history<'de, D>(
    deserializer: D,
) -> Result<Vec<PlaySourceHistoryEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Vec::<PlaySourceHistoryEntryCompat>::deserialize(deserializer)?;
    Ok(values
        .into_iter()
        .map(|value| match value {
            PlaySourceHistoryEntryCompat::Entry(entry) => entry,
            PlaySourceHistoryEntryCompat::Source(source) => PlaySourceHistoryEntry {
                source,
                ..PlaySourceHistoryEntry::default()
            },
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub status: u16,
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResult<T> {
    pub success: bool,
    pub status: u16,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResult<T> {
    pub fn success(status: u16, data: T) -> Self {
        Self {
            success: true,
            status,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure_localized<S>(
        status: u16,
        code: Option<S>,
        message: impl Into<String>,
        params: Option<Value>,
        details: Option<Value>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            success: false,
            status,
            data: None,
            error: Some(ApiError {
                status,
                message: message.into(),
                code: code.map(Into::into),
                params,
                details,
            }),
        }
    }
}

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
