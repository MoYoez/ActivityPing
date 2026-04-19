use std::collections::HashSet;

use crate::{
    models::{
        AppFilterMode, AppMessageRuleGroup, AppMessageTitleRule, AppTitleRuleMode, ClientConfig,
        DiscordReportMode, DiscordRichPresenceButtonConfig,
    },
    platform::{ForegroundSnapshot, MediaInfo},
};

const DISCORD_CUSTOM_LINE_CUSTOM_VALUE: &str = "__custom__";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedActivity {
    pub process_name: String,
    pub process_title: Option<String>,
    pub media_summary: Option<String>,
    pub play_source: Option<String>,
    pub status_text: Option<String>,
    pub discord_addons: ResolvedDiscordAddons,
    pub discord_details: String,
    pub discord_state: Option<String>,
    pub summary: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ResolvedDiscordAddons {
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    pub party: Option<ResolvedDiscordParty>,
    pub secrets: Option<ResolvedDiscordSecrets>,
}

impl ResolvedDiscordAddons {
    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty() && self.party.is_none() && self.secrets.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDiscordParty {
    pub id: Option<String>,
    pub size: Option<(u32, u32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDiscordSecrets {
    pub join: Option<String>,
    pub spectate: Option<String>,
    pub match_secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MessageRuleMatch {
    status_text: Option<String>,
    addons: ResolvedDiscordAddons,
}

pub fn normalize_client_config(config: &mut ClientConfig) {
    config.poll_interval_ms = config.poll_interval_ms.max(1_000);
    config.heartbeat_interval_ms = config.heartbeat_interval_ms.max(0);
    config.capture_history_record_limit = config.capture_history_record_limit.clamp(1, 50);
    config.capture_history_title_limit = config.capture_history_title_limit.clamp(1, 50);
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
    config.discord_details_format = normalize_discord_line_format(&config.discord_details_format);
    config.discord_state_format = normalize_discord_line_format(&config.discord_state_format);
}

fn requires_process_name_for_filters(config: &ClientConfig) -> bool {
    matches!(config.app_filter_mode, AppFilterMode::Whitelist)
        || !config.app_blacklist.is_empty()
        || !config.app_whitelist.is_empty()
}

pub fn should_capture_process_name_for_reporting(config: &ClientConfig) -> bool {
    let filter_capture = requires_process_name_for_filters(config);

    match config.discord_report_mode {
        DiscordReportMode::Music => filter_capture,
        _ => {
            config.report_foreground_app
                || config.discord_smart_show_app_name
                || config.app_message_rules_show_process_name
                || !config.app_message_rules.is_empty()
                || !config.app_name_only_list.is_empty()
                || filter_capture
        }
    }
}

pub fn should_capture_window_title_for_reporting(config: &ClientConfig) -> bool {
    match config.discord_report_mode {
        DiscordReportMode::Music => false,
        _ => config.report_window_title,
    }
}

pub fn should_capture_media_for_reporting(config: &ClientConfig) -> bool {
    match config.discord_report_mode {
        DiscordReportMode::App => false,
        _ => config.report_media || config.report_play_source,
    }
}

pub fn should_capture_foreground_app_icon_for_reporting(config: &ClientConfig) -> bool {
    config.discord_use_app_artwork && config.discord_report_mode != DiscordReportMode::Music
}

pub fn should_capture_foreground_snapshot_for_reporting(config: &ClientConfig) -> bool {
    should_capture_process_name_for_reporting(config)
        || should_capture_window_title_for_reporting(config)
        || should_capture_foreground_app_icon_for_reporting(config)
}

pub fn normalize_string_list(values: &[String], lowercase: bool) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

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
            if pattern.is_empty() || text.is_empty() {
                continue;
            }
            title_rules.push(AppMessageTitleRule {
                mode: title_rule.mode.clone(),
                pattern,
                text,
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

struct DiscordTemplateValues<'a> {
    activity: &'a str,
    context: &'a str,
    app: &'a str,
    title: &'a str,
    rule: &'a str,
    media: String,
    song: &'a str,
    artist: &'a str,
    album: &'a str,
    source: &'a str,
}

impl<'a> DiscordTemplateValues<'a> {
    fn new(
        activity: &'a str,
        context: Option<&'a str>,
        app: &'a str,
        title: Option<&'a str>,
        rule: Option<&'a str>,
        media: &'a MediaInfo,
        source: Option<&'a str>,
        media_visible: bool,
    ) -> Self {
        Self {
            activity,
            context: context.unwrap_or(""),
            app,
            title: title.unwrap_or(""),
            rule: rule.unwrap_or(""),
            media: if media_visible {
                media.summary()
            } else {
                String::new()
            },
            song: if media_visible {
                media.title.as_str()
            } else {
                ""
            },
            artist: if media_visible {
                media.artist.as_str()
            } else {
                ""
            },
            album: if media_visible {
                media.album.as_str()
            } else {
                ""
            },
            source: source.unwrap_or(""),
        }
    }

    fn value(&self, key: &str) -> Option<&str> {
        match key {
            "activity" => Some(self.activity),
            "context" => Some(self.context),
            "app" | "process" => Some(self.app),
            "title" => Some(self.title),
            "rule" => Some(self.rule),
            "media" => Some(self.media.as_str()),
            "song" => Some(self.song),
            "artist" => Some(self.artist),
            "album" => Some(self.album),
            "source" => Some(self.source),
            _ => None,
        }
    }
}

fn render_discord_template(template: &str, values: &DiscordTemplateValues<'_>) -> Option<String> {
    let template = template.trim();
    if template.is_empty() {
        return None;
    }

    let mut output = String::new();
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            output.push(ch);
            continue;
        }

        let mut key = String::new();
        let mut closed = false;
        while let Some(next) = chars.next() {
            if next == '}' {
                closed = true;
                break;
            }
            key.push(next);
        }

        if closed {
            if let Some(value) = values.value(key.trim()) {
                output.push_str(value);
            } else {
                output.push('{');
                output.push_str(&key);
                output.push('}');
            }
        } else {
            output.push('{');
            output.push_str(&key);
        }
    }

    Some(clean_rendered_text(&output))
}

fn clean_rendered_text(value: &str) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim_matches(|ch: char| {
            ch.is_whitespace() || matches!(ch, '|' | '-' | '·' | '/' | '\\' | ',' | ':' | ';')
        })
        .trim()
        .to_string()
}

fn resolve_rule_addons(rule: &AppMessageRuleGroup) -> ResolvedDiscordAddons {
    let buttons = normalize_discord_buttons(&rule.buttons);
    let party_id = non_empty(rule.party_id.as_str());
    let party_size = match (
        normalize_party_size(rule.party_size_current),
        normalize_party_size(rule.party_size_max),
    ) {
        (Some(current), Some(maximum)) if current <= maximum => Some((current, maximum)),
        _ => None,
    };
    let party = if party_id.is_none() && party_size.is_none() {
        None
    } else {
        Some(ResolvedDiscordParty {
            id: party_id,
            size: party_size,
        })
    };
    let join = non_empty(rule.join_secret.as_str());
    let spectate = non_empty(rule.spectate_secret.as_str());
    let match_secret = non_empty(rule.match_secret.as_str());
    let secrets = if join.is_none() && spectate.is_none() && match_secret.is_none() {
        None
    } else {
        Some(ResolvedDiscordSecrets {
            join,
            spectate,
            match_secret,
        })
    };

    ResolvedDiscordAddons {
        buttons,
        party,
        secrets,
    }
}

fn match_message_rule(
    process_name: &str,
    process_title_for_match: Option<&str>,
    process_title_for_template: Option<&str>,
    rules: &[AppMessageRuleGroup],
) -> Option<MessageRuleMatch> {
    let process_lower = process_name.trim().to_lowercase();
    if process_lower.is_empty() {
        return None;
    }

    for rule in rules {
        let matcher = rule.process_match.trim().to_lowercase();
        if matcher.is_empty() || !process_lower.contains(&matcher) {
            continue;
        }
        let addons = resolve_rule_addons(rule);

        for title_rule in &rule.title_rules {
            if !matches_title_rule(process_title_for_match, title_rule) {
                continue;
            }
            return Some(MessageRuleMatch {
                status_text: Some(render_rule_text(
                    &title_rule.text,
                    process_name,
                    process_title_for_template,
                )),
                addons,
            });
        }

        let template = rule.default_text.trim();
        if !template.is_empty() {
            return Some(MessageRuleMatch {
                status_text: Some(render_rule_text(
                    template,
                    process_name,
                    process_title_for_template,
                )),
                addons,
            });
        }

        if rule.title_rules.is_empty() && !addons.is_empty() {
            return Some(MessageRuleMatch {
                status_text: None,
                addons,
            });
        }

        if template.is_empty() {
            continue;
        }
    }

    None
}

fn matches_title_rule(process_title: Option<&str>, title_rule: &AppMessageTitleRule) -> bool {
    let Some(process_title) = process_title
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };

    match title_rule.mode {
        AppTitleRuleMode::Regex => std::panic::catch_unwind(|| {
            regex::RegexBuilder::new(title_rule.pattern.as_str())
                .case_insensitive(true)
                .build()
                .map(|regex| regex.is_match(process_title))
                .unwrap_or(false)
        })
        .unwrap_or(false),
        AppTitleRuleMode::Plain => process_title
            .to_lowercase()
            .contains(&title_rule.pattern.trim().to_lowercase()),
    }
}

fn render_rule_text(template: &str, process_name: &str, process_title: Option<&str>) -> String {
    template
        .replace("{process}", process_name)
        .replace("{title}", process_title.unwrap_or(""))
        .trim()
        .to_string()
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
mod tests {
    use super::*;
    use crate::{
        models::DiscordReportMode,
        platform::{ForegroundSnapshot, MediaInfo},
    };

    fn base_config() -> ClientConfig {
        ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            ..ClientConfig::default()
        }
    }

    fn sample_media() -> MediaInfo {
        MediaInfo {
            title: "Track Name".into(),
            artist: "Artist Name".into(),
            album: "Album Name".into(),
            is_playing: true,
            ..MediaInfo::default()
        }
    }

    #[test]
    fn music_mode_uses_title_then_artist() {
        let config = base_config();
        let media = sample_media();

        let resolved = build_music_discord_text(&config, &media, false);

        assert_eq!(
            resolved,
            Some(("Track Name".to_string(), Some("Artist Name".to_string())))
        );
    }

    #[test]
    fn music_mode_drops_state_when_artist_missing() {
        let config = base_config();
        let mut media = sample_media();
        media.artist.clear();

        let resolved = build_music_discord_text(&config, &media, false);

        assert_eq!(resolved, Some(("Track Name".to_string(), None)));
    }

    #[test]
    fn music_mode_can_keep_paused_media_visible() {
        let mut config = base_config();
        config.report_stopped_media = true;
        let mut media = sample_media();
        media.is_playing = false;

        let resolved = build_music_discord_text(&config, &media, false);

        assert_eq!(
            resolved,
            Some(("Track Name".to_string(), Some("Artist Name".to_string())))
        );
    }

    #[test]
    fn music_mode_hides_paused_media_by_default() {
        let config = base_config();
        let mut media = sample_media();
        media.is_playing = false;

        let resolved = build_music_discord_text(&config, &media, false);

        assert_eq!(resolved, None);
    }

    #[test]
    fn app_mode_does_not_capture_media_for_reporting() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::App,
            report_media: true,
            report_play_source: true,
            ..ClientConfig::default()
        };

        assert!(!should_capture_media_for_reporting(&config));
    }

    #[test]
    fn music_mode_does_not_capture_window_title_for_reporting() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            report_window_title: true,
            ..ClientConfig::default()
        };

        assert!(!should_capture_window_title_for_reporting(&config));
    }

    #[test]
    fn music_mode_skips_process_name_without_filters() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            report_foreground_app: true,
            ..ClientConfig::default()
        };

        assert!(!should_capture_process_name_for_reporting(&config));
    }

    #[test]
    fn music_mode_keeps_process_name_for_app_filters() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            app_whitelist: vec!["spotify.exe".into()],
            ..ClientConfig::default()
        };

        assert!(should_capture_process_name_for_reporting(&config));
    }

    #[test]
    fn music_mode_does_not_capture_foreground_app_icon() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            discord_use_app_artwork: true,
            ..ClientConfig::default()
        };

        assert!(!should_capture_foreground_app_icon_for_reporting(&config));
    }

    #[test]
    fn music_mode_can_skip_foreground_snapshot_when_unneeded() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            ..ClientConfig::default()
        };

        assert!(!should_capture_foreground_snapshot_for_reporting(&config));
    }

    #[test]
    fn app_mode_still_captures_foreground_snapshot() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::App,
            report_window_title: true,
            ..ClientConfig::default()
        };

        assert!(should_capture_foreground_snapshot_for_reporting(&config));
    }

    #[test]
    fn app_mode_shows_process_name_on_state_when_rule_hits() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::App,
            app_message_rules_show_process_name: true,
            ..ClientConfig::default()
        };

        let resolved = build_discord_text(
            &config,
            "Spotify.exe",
            None,
            &MediaInfo::default(),
            false,
            Some("正在 声破天 听歌中"),
        )
        .expect("activity");

        assert_eq!(resolved.0, "正在 声破天 听歌中".to_string());
        assert_eq!(resolved.1, Some("Spotify.exe".to_string()));
    }

    #[test]
    fn smart_mode_uses_title_then_visible_media_and_keeps_app_in_signature() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            ..ClientConfig::default()
        };
        let snapshot = ForegroundSnapshot {
            process_name: "Code.exe".into(),
            process_title: "repo".into(),
        };
        let media = sample_media();

        let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

        assert_eq!(resolved.discord_details, "repo");
        assert_eq!(
            resolved.discord_state,
            Some("Track Name / Artist Name / Album Name".to_string())
        );
        assert_eq!(
            resolved.summary,
            "repo · Track Name / Artist Name / Album Name".to_string()
        );
        assert_eq!(
            resolved.signature,
            "repo · Track Name / Artist Name / Album Name · Code.exe".to_string()
        );
    }

    #[test]
    fn normalize_client_config_migrates_legacy_artwork_toggle() {
        let mut config = ClientConfig {
            legacy_discord_use_media_artwork: true,
            ..ClientConfig::default()
        };

        normalize_client_config(&mut config);

        assert!(config.discord_use_app_artwork);
        assert!(config.discord_use_music_artwork);
        assert!(!config.legacy_discord_use_media_artwork);
    }

    #[test]
    fn smart_mode_appends_app_name_to_last_line_only_on_rule_hit() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_smart_show_app_name: true,
            ..ClientConfig::default()
        };
        let snapshot = ForegroundSnapshot {
            process_name: "Code.exe".into(),
            process_title: "repo".into(),
        };
        let media = sample_media();

        let resolved = build_discord_text(
            &config,
            snapshot.process_name.as_str(),
            None,
            &media,
            false,
            Some("Coding"),
        )
        .expect("activity");

        assert_eq!(resolved.0, "Coding | Code.exe".to_string());
        assert_eq!(
            resolved.1,
            Some("Track Name / Artist Name / Album Name".to_string())
        );
    }

    #[test]
    fn smart_mode_keeps_last_line_empty_without_music_or_rule_app_name() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            ..ClientConfig::default()
        };
        let snapshot = ForegroundSnapshot {
            process_name: "Code.exe".into(),
            process_title: "repo".into(),
        };
        let media = MediaInfo::default();

        let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

        assert_eq!(resolved.discord_details, "repo");
        assert_eq!(resolved.discord_state, None);
    }

    #[test]
    fn smart_mode_falls_back_to_music_only_when_app_name_is_not_reported() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            report_foreground_app: false,
            ..ClientConfig::default()
        };
        let snapshot = ForegroundSnapshot {
            process_name: "Code.exe".into(),
            process_title: "repo".into(),
        };
        let media = sample_media();

        let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

        assert_eq!(resolved.discord_details, "Track Name");
        assert_eq!(resolved.discord_state, Some("Artist Name".to_string()));
    }

    #[test]
    fn smart_mode_can_show_rule_hit_app_name_even_when_global_app_reporting_is_off() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            report_foreground_app: false,
            discord_smart_show_app_name: true,
            ..ClientConfig::default()
        };
        let media = sample_media();

        let resolved = build_discord_text(&config, "Code.exe", None, &media, false, Some("Coding"))
            .expect("activity");

        assert_eq!(resolved.0, "Coding | Code.exe".to_string());
        assert_eq!(
            resolved.1,
            Some("Track Name / Artist Name / Album Name".to_string())
        );
    }

    #[test]
    fn smart_mode_can_show_app_name_without_music() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_smart_show_app_name: true,
            ..ClientConfig::default()
        };
        let snapshot = ForegroundSnapshot {
            process_name: "Code.exe".into(),
            process_title: "repo".into(),
        };
        let media = MediaInfo::default();

        let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

        assert_eq!(resolved.discord_details, "repo");
        assert_eq!(resolved.discord_state, Some("Code.exe".to_string()));
    }

    #[test]
    fn smart_mode_uses_rule_hit_process_name_on_last_line_without_media() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            app_message_rules_show_process_name: true,
            ..ClientConfig::default()
        };

        let resolved = build_discord_text(
            &config,
            "Code.exe",
            None,
            &MediaInfo::default(),
            false,
            Some("Coding"),
        )
        .expect("activity");

        assert_eq!(resolved.0, "Coding".to_string());
        assert_eq!(resolved.1, Some("Code.exe".to_string()));
    }

    #[test]
    fn custom_mode_applies_global_details_and_state_templates() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Custom,
            discord_details_format: "{app} :: {activity}".into(),
            discord_state_format: "Line 3: {context}".into(),
            ..ClientConfig::default()
        };

        let resolved = build_discord_text(
            &config,
            "Code.exe",
            Some("repo"),
            &MediaInfo::default(),
            false,
            None,
        )
        .expect("activity");

        assert_eq!(resolved.0, "Code.exe :: repo".to_string());
        assert_eq!(resolved.1, Some("Line 3: Code.exe".to_string()));
    }

    #[test]
    fn custom_mode_can_hide_details_line() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Custom,
            discord_details_format: String::new(),
            discord_state_format: "{context}".into(),
            ..ClientConfig::default()
        };

        let resolved = build_discord_text(
            &config,
            "Code.exe",
            Some("repo"),
            &MediaInfo::default(),
            false,
            None,
        )
        .expect("activity");

        assert_eq!(resolved.0, String::new());
        assert_eq!(resolved.1, Some("Code.exe".to_string()));
    }

    #[test]
    fn custom_mode_can_use_literal_custom_text() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Custom,
            discord_details_format: "Coding in {app}".into(),
            discord_state_format: "Working on {title}".into(),
            ..ClientConfig::default()
        };

        let resolved = build_discord_text(
            &config,
            "Code.exe",
            Some("repo"),
            &MediaInfo::default(),
            false,
            None,
        )
        .expect("activity");

        assert_eq!(resolved.0, "Coding in Code.exe".to_string());
        assert_eq!(resolved.1, Some("Working on repo".to_string()));
    }
}
