mod capture;
mod message_rules;
mod templates;
mod types;

pub use capture::{
    should_capture_foreground_app_icon_for_reporting,
    should_capture_foreground_snapshot_for_reporting, should_capture_media_for_reporting,
    should_capture_process_name_for_reporting, should_capture_window_title_for_reporting,
};
pub use types::{
    ResolvedActivity, ResolvedDiscordAddons, ResolvedDiscordParty, ResolvedDiscordSecrets,
};

use crate::{
    models::{
        AppFilterMode, AppMessageRuleGroup, AppMessageTitleRule, ClientConfig, DiscordCustomAsset,
        DiscordCustomPreset, DiscordReportMode,
    },
    platform::{ForegroundSnapshot, MediaInfo},
};
use message_rules::match_message_rule;
use templates::{render_discord_template, DiscordTemplateValues};

const DISCORD_CUSTOM_LINE_CUSTOM_VALUE: &str = "__custom__";

pub fn normalize_client_config(config: &mut ClientConfig) {
    config.poll_interval_ms = config.poll_interval_ms.max(1_000);
    config.heartbeat_interval_ms = config.heartbeat_interval_ms.max(0);
    config.capture_history_record_limit = config.capture_history_record_limit.clamp(1, 50);
    config.capture_history_title_limit = config.capture_history_title_limit.clamp(1, 50);
    if config.capture_history_title_limit == 5 && config.capture_history_record_limit != 5 {
        config.capture_history_title_limit = config.capture_history_record_limit;
    }
    config.runtime_autostart_enabled = config.runtime_autostart_enabled
        || config.legacy_reporter_enabled
        || config.legacy_discord_enabled;
    config.legacy_reporter_enabled = false;
    config.legacy_discord_enabled = false;
    config.discord_use_app_artwork =
        config.discord_use_app_artwork || config.legacy_discord_use_media_artwork;
    config.discord_use_music_artwork =
        config.discord_use_music_artwork || config.legacy_discord_use_media_artwork;
    config.legacy_discord_use_media_artwork = false;
    config.discord_application_id = config.discord_application_id.trim().to_string();
    config.discord_artwork_worker_upload_url =
        config.discord_artwork_worker_upload_url.trim().to_string();
    config.discord_artwork_worker_token = config.discord_artwork_worker_token.trim().to_string();
    config.discord_custom_artwork_text = config.discord_custom_artwork_text.trim().to_string();
    config.discord_custom_artwork_asset_id =
        config.discord_custom_artwork_asset_id.trim().to_string();
    config.discord_custom_app_icon_text =
        config.discord_custom_app_icon_text.trim().to_string();
    config.discord_custom_app_icon_asset_id =
        config.discord_custom_app_icon_asset_id.trim().to_string();
    config.discord_custom_assets = normalize_discord_custom_assets(&config.discord_custom_assets);
    config.app_blacklist = normalize_string_list(&config.app_blacklist, false);
    config.app_whitelist = normalize_string_list(&config.app_whitelist, false);
    config.app_name_only_list = normalize_string_list(&config.app_name_only_list, false);
    config.media_play_source_blocklist =
        normalize_string_list(&config.media_play_source_blocklist, true);
    config.app_message_rules = normalize_rule_groups(&config.app_message_rules);
    if let Some(value) = config.legacy_discord_status_display.clone() {
        config.discord_smart_status_display = value.clone();
        config.discord_app_status_display = value.clone();
        config.discord_custom_mode_status_display = value.clone();
        config.discord_music_status_display = value;
    }
    if let Some(value) = config.legacy_discord_app_name_mode.clone() {
        config.discord_smart_app_name_mode = value.clone();
        config.discord_app_app_name_mode = value.clone();
        config.discord_custom_mode_app_name_mode = value.clone();
        config.discord_music_app_name_mode = value;
    }
    if let Some(value) = config.legacy_discord_custom_app_name.as_ref() {
        let trimmed = value.trim().to_string();
        config.discord_smart_custom_app_name = trimmed.clone();
        config.discord_app_custom_app_name = trimmed.clone();
        config.discord_custom_mode_custom_app_name = trimmed.clone();
        config.discord_music_custom_app_name = trimmed;
    }
    config.legacy_discord_status_display = None;
    config.legacy_discord_app_name_mode = None;
    config.legacy_discord_custom_app_name = None;
    config.discord_smart_custom_app_name = config.discord_smart_custom_app_name.trim().to_string();
    config.discord_music_custom_app_name = config.discord_music_custom_app_name.trim().to_string();
    config.discord_app_custom_app_name = config.discord_app_custom_app_name.trim().to_string();
    config.discord_custom_mode_custom_app_name = config
        .discord_custom_mode_custom_app_name
        .trim()
        .to_string();
    config.discord_custom_buttons = normalize_discord_buttons(&config.discord_custom_buttons);
    config.discord_custom_party_id = config.discord_custom_party_id.trim().to_string();
    config.discord_custom_party_size_current =
        normalize_party_size(config.discord_custom_party_size_current);
    config.discord_custom_party_size_max =
        normalize_party_size(config.discord_custom_party_size_max);
    config.discord_custom_join_secret = config.discord_custom_join_secret.trim().to_string();
    config.discord_custom_spectate_secret =
        config.discord_custom_spectate_secret.trim().to_string();
    config.discord_custom_match_secret = config.discord_custom_match_secret.trim().to_string();
    config.discord_custom_presets = normalize_discord_custom_presets(&config.discord_custom_presets);
    config.discord_details_format = normalize_discord_line_format(&config.discord_details_format);
    config.discord_state_format = normalize_discord_line_format(&config.discord_state_format);
}

pub fn normalize_string_list(values: &[String], lowercase: bool) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_lowercase();
        if !seen.insert(key.clone()) {
            continue;
        }
        result.push(if lowercase { key } else { trimmed.to_string() });
    }

    result
}

pub fn resolve_activity(
    config: &ClientConfig,
    snapshot: &ForegroundSnapshot,
    media: &MediaInfo,
) -> Option<ResolvedActivity> {
    let process_name = snapshot.process_name.trim().to_string();
    if !passes_app_filter(config, &process_name) {
        return None;
    }

    let process_title_raw = non_empty(snapshot.process_title.as_str());
    let process_title_masked = if in_process_list(&config.app_name_only_list, &process_name) {
        None
    } else {
        process_title_raw.clone()
    };

    let raw_play_source = if config.report_play_source {
        non_empty(media.source_app_id.as_str())
    } else {
        None
    };
    let media_hidden = raw_play_source
        .as_ref()
        .map(|value| is_media_source_blocked(config, value))
        .unwrap_or(false);
    let media_reportable = media.is_reportable(config.report_stopped_media);
    let play_source = if config.report_play_source && media_reportable && !media_hidden {
        raw_play_source.clone()
    } else {
        None
    };
    let media_summary = if config.report_media && media_reportable && !media_hidden {
        non_empty(media.summary().as_str())
    } else {
        None
    };

    let matched_rule = match_message_rule(
        &process_name,
        process_title_raw.as_deref(),
        process_title_masked.as_deref(),
        &config.app_message_rules,
    );
    let status_text = matched_rule
        .as_ref()
        .and_then(|rule_match| rule_match.status_text.clone());

    let display_title = if status_text.is_some() {
        None
    } else {
        process_title_masked.clone()
    };

    let Some((discord_details, discord_state)) = build_discord_text(
        config,
        &process_name,
        display_title.as_deref(),
        media,
        media_hidden,
        status_text.as_deref(),
    ) else {
        return None;
    };

    let mut summary_parts = vec![discord_details.clone()];
    if let Some(state) = discord_state.as_ref() {
        summary_parts.push(state.clone());
    }
    let summary = summary_parts.join(" · ");

    let mut signature_parts = summary_parts;
    if config.discord_report_mode == DiscordReportMode::Mixed {
        if config.report_foreground_app && !process_name.is_empty() {
            if !signature_parts.iter().any(|value| value == &process_name) {
                signature_parts.push(process_name.clone());
            }
        }
        if let Some(media_summary) = media_summary.as_ref() {
            if !signature_parts.iter().any(|value| value == media_summary) {
                signature_parts.push(media_summary.clone());
            }
        }
    }
    let signature = signature_parts.join(" · ");

    Some(ResolvedActivity {
        process_name,
        process_title: display_title,
        media_summary,
        play_source,
        status_text,
        discord_addons: matched_rule
            .map(|rule_match| rule_match.addons)
            .unwrap_or_default(),
        discord_details,
        discord_state,
        summary,
        signature,
    })
}

fn normalize_rule_groups(rules: &[AppMessageRuleGroup]) -> Vec<AppMessageRuleGroup> {
    let mut normalized = Vec::new();

    for rule in rules {
        let process_match = rule.process_match.trim().to_string();
        if process_match.is_empty() {
            continue;
        }

        let default_text = rule.default_text.trim().to_string();
        let mut title_rules = Vec::new();
        let buttons = normalize_discord_buttons(&rule.buttons);
        let party_id = rule.party_id.trim().to_string();
        let party_size_current = normalize_party_size(rule.party_size_current);
        let party_size_max = normalize_party_size(rule.party_size_max);
        let join_secret = rule.join_secret.trim().to_string();
        let spectate_secret = rule.spectate_secret.trim().to_string();
        let match_secret = rule.match_secret.trim().to_string();

        for title_rule in &rule.title_rules {
            let pattern = title_rule.pattern.trim().to_string();
            let text = title_rule.text.trim().to_string();
            let buttons = normalize_discord_buttons(&title_rule.buttons);
            if pattern.is_empty() || text.is_empty() {
                continue;
            }
            title_rules.push(AppMessageTitleRule {
                mode: title_rule.mode.clone(),
                pattern,
                text,
                buttons,
            });
        }

        if default_text.is_empty()
            && title_rules.is_empty()
            && buttons.is_empty()
            && party_id.is_empty()
            && party_size_current.is_none()
            && party_size_max.is_none()
            && join_secret.is_empty()
            && spectate_secret.is_empty()
            && match_secret.is_empty()
        {
            continue;
        }

        normalized.push(AppMessageRuleGroup {
            process_match,
            default_text,
            title_rules,
            buttons,
            party_id,
            party_size_current,
            party_size_max,
            join_secret,
            spectate_secret,
            match_secret,
        });
    }

    normalized
}

fn normalize_discord_buttons(
    buttons: &[crate::models::DiscordRichPresenceButtonConfig],
) -> Vec<crate::models::DiscordRichPresenceButtonConfig> {
    buttons
        .iter()
        .map(|button| crate::models::DiscordRichPresenceButtonConfig {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .collect()
}

fn normalize_discord_custom_assets(assets: &[DiscordCustomAsset]) -> Vec<DiscordCustomAsset> {
    let mut normalized = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for asset in assets {
        let id = asset.id.trim().to_string();
        let name = asset.name.trim().to_string();
        let file_name = asset.file_name.trim().to_string();
        let stored_path = asset.stored_path.trim().to_string();
        let content_type = asset.content_type.trim().to_ascii_lowercase();
        let created_at = asset.created_at.trim().to_string();
        if id.is_empty()
            || name.is_empty()
            || file_name.is_empty()
            || stored_path.is_empty()
            || content_type.is_empty()
            || created_at.is_empty()
            || !seen.insert(id.clone())
        {
            continue;
        }

        normalized.push(DiscordCustomAsset {
            id,
            name,
            file_name,
            stored_path,
            content_type,
            byte_size: asset.byte_size,
            created_at,
        });
    }

    normalized
}

fn normalize_discord_custom_presets(presets: &[DiscordCustomPreset]) -> Vec<DiscordCustomPreset> {
    presets
        .iter()
        .map(|preset| DiscordCustomPreset {
            name: preset.name.trim().to_string(),
            activity_type: preset.activity_type.clone(),
            status_display: preset.status_display.clone(),
            app_name_mode: preset.app_name_mode.clone(),
            custom_app_name: preset.custom_app_name.trim().to_string(),
            details_format: normalize_discord_line_format(&preset.details_format),
            state_format: normalize_discord_line_format(&preset.state_format),
            custom_artwork_source: preset.custom_artwork_source.clone(),
            custom_artwork_text_mode: preset.custom_artwork_text_mode.clone(),
            custom_artwork_text: preset.custom_artwork_text.trim().to_string(),
            custom_artwork_asset_id: preset.custom_artwork_asset_id.trim().to_string(),
            custom_app_icon_source: preset.custom_app_icon_source.clone(),
            custom_app_icon_text_mode: preset.custom_app_icon_text_mode.clone(),
            custom_app_icon_text: preset.custom_app_icon_text.trim().to_string(),
            custom_app_icon_asset_id: preset.custom_app_icon_asset_id.trim().to_string(),
            buttons: normalize_discord_buttons(&preset.buttons),
            party_id: preset.party_id.trim().to_string(),
            party_size_current: normalize_party_size(preset.party_size_current),
            party_size_max: normalize_party_size(preset.party_size_max),
            join_secret: preset.join_secret.trim().to_string(),
            spectate_secret: preset.spectate_secret.trim().to_string(),
            match_secret: preset.match_secret.trim().to_string(),
        })
        .collect()
}

fn normalize_party_size(value: Option<u32>) -> Option<u32> {
    value.filter(|size| *size > 0)
}

fn passes_app_filter(config: &ClientConfig, process_name: &str) -> bool {
    let key = normalize_process_name(process_name);
    match config.app_filter_mode {
        AppFilterMode::Whitelist => {
            if config.app_whitelist.is_empty() {
                return false;
            }
            config
                .app_whitelist
                .iter()
                .any(|candidate| normalize_process_name(candidate) == key)
        }
        AppFilterMode::Blacklist => !config
            .app_blacklist
            .iter()
            .any(|candidate| normalize_process_name(candidate) == key),
    }
}

fn in_process_list(values: &[String], process_name: &str) -> bool {
    let key = normalize_process_name(process_name);
    values
        .iter()
        .any(|candidate| normalize_process_name(candidate) == key)
}

fn is_media_source_blocked(config: &ClientConfig, play_source: &str) -> bool {
    let key = play_source.trim().to_lowercase();
    if key.is_empty() {
        return false;
    }
    config
        .media_play_source_blocklist
        .iter()
        .any(|candidate| candidate.trim().eq_ignore_ascii_case(&key))
}

fn normalize_discord_line_format(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == DISCORD_CUSTOM_LINE_CUSTOM_VALUE {
        String::new()
    } else {
        trimmed.to_string()
    }
}

fn build_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    let (base_details, base_state) = match config.discord_report_mode {
        DiscordReportMode::Music => build_music_discord_text(config, media, media_hidden),
        DiscordReportMode::App => {
            build_app_discord_text(config, process_name, process_title, status_text)
        }
        DiscordReportMode::Mixed => build_smart_discord_text(
            config,
            process_name,
            process_title,
            media,
            media_hidden,
            status_text,
        ),
        DiscordReportMode::Custom => build_custom_base_discord_text(
            config,
            process_name,
            process_title,
            media,
            media_hidden,
            status_text,
        ),
    }?;
    if config.discord_report_mode != DiscordReportMode::Custom {
        return Some((base_details, base_state));
    }
    let media_visible = media.is_reportable(config.report_stopped_media) && !media_hidden;
    let visible_source =
        if media_visible && config.report_play_source && !media.source_app_id.trim().is_empty() {
            Some(media.source_app_id.as_str())
        } else {
            None
        };
    let values = DiscordTemplateValues::new(
        &base_details,
        base_state.as_deref(),
        process_name,
        process_title,
        status_text,
        media,
        visible_source,
        media_visible,
    );
    let state = render_discord_template(&config.discord_state_format, &values)
        .filter(|value| !value.is_empty());
    let details = match render_discord_template(&config.discord_details_format, &values) {
        Some(value) if !value.is_empty() => value,
        Some(_) | None if config.discord_details_format.trim().is_empty() => String::new(),
        _ => base_details,
    };

    Some((details, state))
}

fn build_smart_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    if build_smart_media_text(config, media, media_hidden).is_some()
        && !has_reported_app_name(config, process_name, status_text)
    {
        return build_music_discord_text(config, media, media_hidden);
    }

    if let Some(title_text) =
        build_smart_primary_title_text(config, process_name, process_title, status_text)
    {
        let media_text = build_smart_media_text(config, media, media_hidden);
        let app_text =
            build_smart_rule_hit_app_text(config, process_name, status_text, media_text.is_none());
        return Some(build_smart_text_layout(title_text, media_text, app_text));
    }

    if build_smart_media_text(config, media, media_hidden).is_some() {
        return build_music_discord_text(config, media, media_hidden);
    }

    build_app_discord_text(config, process_name, process_title, status_text)
}

fn build_custom_base_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    if let Some(app_text) = build_app_discord_text(config, process_name, process_title, status_text)
    {
        if status_text.is_some() {
            return Some(app_text);
        }
    }

    build_mixed_music_discord_text(config, process_name, process_title, media, media_hidden)
        .or_else(|| build_app_discord_text(config, process_name, process_title, status_text))
}

fn build_smart_primary_title_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    status_text: Option<&str>,
) -> Option<String> {
    if let Some(status_text) = status_text.and_then(non_empty) {
        return Some(status_text);
    }

    if let Some(process_title) = process_title.and_then(non_empty) {
        return Some(process_title);
    }

    if config.report_foreground_app {
        return non_empty(process_name);
    }

    None
}

fn build_smart_rule_hit_app_text(
    config: &ClientConfig,
    process_name: &str,
    status_text: Option<&str>,
    no_media_visible: bool,
) -> Option<String> {
    if !config.discord_smart_show_app_name {
        if !no_media_visible
            || !config.app_message_rules_show_process_name
            || status_text.and_then(non_empty).is_none()
        {
            return None;
        }
    }

    non_empty(process_name)
}

fn build_smart_media_text(
    config: &ClientConfig,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<String> {
    if !config.report_media || media_hidden || !media.is_reportable(config.report_stopped_media) {
        return None;
    }

    non_empty(media.summary().as_str())
}

fn has_reported_app_name(
    config: &ClientConfig,
    process_name: &str,
    status_text: Option<&str>,
) -> bool {
    if non_empty(process_name).is_none() {
        return false;
    }

    config.report_foreground_app
        || config.discord_smart_show_app_name
        || status_text.and_then(non_empty).is_some()
}

fn build_mixed_music_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<(String, Option<String>)> {
    if !config.report_media || media_hidden || !media.is_reportable(config.report_stopped_media) {
        return None;
    }

    let details = first_non_empty(&[media.title.as_str(), media.summary().as_str()])?;
    let state = join_non_empty(&[
        media.artist.as_str(),
        if config.report_window_title {
            process_title.unwrap_or("")
        } else {
            ""
        },
        if config.report_foreground_app {
            process_name
        } else {
            ""
        },
    ]);
    Some((details, state))
}

fn build_music_discord_text(
    config: &ClientConfig,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<(String, Option<String>)> {
    if !config.report_media || media_hidden || !media.is_reportable(config.report_stopped_media) {
        return None;
    }

    let details = first_non_empty(&[media.title.as_str(), media.summary().as_str()])?;
    let state = non_empty(media.artist.as_str());
    Some((details, state))
}

fn build_app_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    if let Some(status_text) = status_text {
        let details = status_text.trim().to_string();
        if details.is_empty() {
            return None;
        }
        let state = if config.app_message_rules_show_process_name {
            non_empty(process_name)
        } else {
            None
        };
        return Some((details, state));
    }

    if let Some(process_title) = process_title {
        let details = process_title.trim().to_string();
        if details.is_empty() {
            return None;
        }
        let state = if config.report_foreground_app && !process_name.trim().is_empty() {
            Some(process_name.trim().to_string())
        } else {
            None
        };
        return Some((details, state));
    }

    if config.report_foreground_app && !process_name.trim().is_empty() {
        return Some((process_name.trim().to_string(), None));
    }

    None
}

fn normalize_process_name(value: &str) -> String {
    value.trim().to_lowercase()
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn first_non_empty(values: &[&str]) -> Option<String> {
    values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

fn join_non_empty(values: &[&str]) -> Option<String> {
    let parts = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn build_smart_text_layout(
    title_text: String,
    media_text: Option<String>,
    app_text: Option<String>,
) -> (String, Option<String>) {
    match (media_text, app_text) {
        (Some(media_text), Some(app_text)) => {
            (format!("{title_text} | {app_text}"), Some(media_text))
        }
        (Some(media_text), None) => (title_text, Some(media_text)),
        (None, Some(app_text)) => (title_text, Some(app_text)),
        (None, None) => (title_text, None),
    }
}

#[cfg(test)]
mod tests;
