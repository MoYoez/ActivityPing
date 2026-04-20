use serde::{Deserialize, Serialize};

use super::config::ClientConfig;

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
