use crate::{
    models::{ClientConfig, DiscordReportMode},
    platform::{ForegroundSnapshot, MediaInfo},
};

use super::{
    discord_text::build_discord_text,
    filters::{in_process_list, is_media_source_blocked, passes_app_filter},
    helpers::non_empty,
    message_rules::match_message_rule,
    types::ResolvedActivity,
};

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
