use std::collections::HashSet;

use crate::{
    models::{
        AppFilterMode, AppMessageRuleGroup, AppMessageTitleRule, AppTitleRuleMode, ClientConfig,
        DiscordReportMode,
    },
    platform::{ForegroundSnapshot, MediaInfo},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedActivity {
    pub process_name: String,
    pub process_title: Option<String>,
    pub media_summary: Option<String>,
    pub play_source: Option<String>,
    pub status_text: Option<String>,
    pub discord_details: String,
    pub discord_state: Option<String>,
    pub summary: String,
    pub signature: String,
}

pub fn normalize_client_config(config: &mut ClientConfig) {
    config.poll_interval_ms = config.poll_interval_ms.max(1_000);
    config.heartbeat_interval_ms = config.heartbeat_interval_ms.max(0);
    config.runtime_autostart_enabled = config.runtime_autostart_enabled
        || config.legacy_reporter_enabled
        || config.legacy_discord_enabled;
    config.legacy_reporter_enabled = false;
    config.legacy_discord_enabled = false;
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
    config.discord_details_format = normalize_details_format(&config.discord_details_format);
    config.discord_state_format = config.discord_state_format.trim().to_string();
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
    let play_source = if config.report_play_source && media.is_active() && !media_hidden {
        raw_play_source.clone()
    } else {
        None
    };
    let media_summary = if config.report_media && media.is_active() && !media_hidden {
        non_empty(media.summary().as_str())
    } else {
        None
    };

    let status_text = apply_message_rule(
        &process_name,
        process_title_raw.as_deref(),
        process_title_masked.as_deref(),
        &config.app_message_rules,
    )
    .map(|text| {
        if config.app_message_rules_show_process_name && !process_name.is_empty() {
            format!("{text} | {process_name}")
        } else {
            text
        }
    });

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

    let summary = discord_state
        .as_ref()
        .map(|state| format!("{discord_details} · {state}"))
        .unwrap_or_else(|| discord_details.clone());
    let signature = summary.clone();

    Some(ResolvedActivity {
        process_name,
        process_title: display_title,
        media_summary,
        play_source,
        status_text,
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

        if default_text.is_empty() && title_rules.is_empty() {
            continue;
        }

        normalized.push(AppMessageRuleGroup {
            process_match,
            default_text,
            title_rules,
        });
    }

    normalized
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

fn normalize_details_format(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "{activity}".into()
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
    let values = DiscordTemplateValues::new(
        &base_details,
        base_state.as_deref(),
        process_name,
        process_title,
        status_text,
        media,
        if media.is_active() && config.report_play_source && !media_hidden {
            Some(media.source_app_id.as_str())
        } else {
            None
        },
        media.is_active() && !media_hidden,
    );
    let state = render_discord_template(&config.discord_state_format, &values)
        .filter(|value| !value.is_empty());
    let details = render_discord_template(&config.discord_details_format, &values)
        .filter(|value| !value.is_empty())
        .unwrap_or(base_details);

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
    if let Some(media_text) = build_smart_media_text(config, media, media_hidden) {
        if let Some(app_text) =
            build_smart_primary_app_text(config, process_name, process_title, status_text)
        {
            return Some((app_text, Some(media_text)));
        }

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

fn build_smart_primary_app_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    status_text: Option<&str>,
) -> Option<String> {
    if let Some(status_text) = status_text.and_then(non_empty) {
        return Some(status_text);
    }

    if config.report_foreground_app {
        if let Some(process_name) = non_empty(process_name) {
            return Some(process_name);
        }
    }

    process_title.and_then(non_empty)
}

fn build_smart_media_text(
    config: &ClientConfig,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<String> {
    if !config.report_media || media_hidden || !media.is_active() {
        return None;
    }

    non_empty(media.summary().as_str())
}

fn build_mixed_music_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<(String, Option<String>)> {
    if !config.report_media || media_hidden || !media.is_active() {
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
    if !config.report_media || media_hidden || !media.is_active() {
        return None;
    }

    let details = first_non_empty(&[media.title.as_str(), media.summary().as_str()])?;
    let state = join_non_empty(&[media.artist.as_str(), media.album.as_str()]);
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
        return Some((details, None));
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

fn apply_message_rule(
    process_name: &str,
    process_title_for_match: Option<&str>,
    process_title_for_template: Option<&str>,
    rules: &[AppMessageRuleGroup],
) -> Option<String> {
    let process_lower = process_name.trim().to_lowercase();
    if process_lower.is_empty() {
        return None;
    }

    for rule in rules {
        let matcher = rule.process_match.trim().to_lowercase();
        if matcher.is_empty() || !process_lower.contains(&matcher) {
            continue;
        }

        for title_rule in &rule.title_rules {
            if !matches_title_rule(process_title_for_match, title_rule) {
                continue;
            }
            return Some(render_rule_text(
                &title_rule.text,
                process_name,
                process_title_for_template,
            ));
        }

        let template = rule.default_text.trim();
        if template.is_empty() {
            continue;
        }
        return Some(render_rule_text(
            template,
            process_name,
            process_title_for_template,
        ));
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
