use crate::{
    models::{
        ClientConfig, DiscordAssetTextMode, DiscordReportMode, DiscordSmartArtworkPreference,
    },
    platform::display_name_for_app_id,
};

pub(super) fn custom_hover_text(mode: &DiscordAssetTextMode, value: &str) -> Option<String> {
    match mode {
        DiscordAssetTextMode::Custom => Some(value.trim().to_string()),
        DiscordAssetTextMode::Auto => None,
    }
}

pub(super) fn smart_mode_prefers_app_artwork(config: &ClientConfig) -> bool {
    config.discord_report_mode == DiscordReportMode::Mixed
        && config.discord_smart_artwork_preference == DiscordSmartArtworkPreference::App
}

pub(super) fn fallback_app_name(source: &str) -> String {
    if let Some(display_name) = display_name_for_app_id(source) {
        return display_name;
    }

    let trimmed = source.trim();
    if trimmed.is_empty() {
        return "Playback app".to_string();
    }

    let tail = trimmed.rsplit(['\\', '/', '!']).next().unwrap_or(trimmed);
    let tail = tail
        .strip_suffix(".exe")
        .or_else(|| tail.strip_suffix(".app"))
        .or_else(|| tail.strip_suffix(".desktop"))
        .unwrap_or(tail);
    let tail = tail.split('_').next().unwrap_or(tail);

    if tail.contains('.') && !tail.contains(' ') {
        let bundle_tail = tail.rsplit('.').next().unwrap_or(tail);
        return title_case_words(bundle_tail);
    }

    title_case_words(tail)
}

fn title_case_words(value: &str) -> String {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
