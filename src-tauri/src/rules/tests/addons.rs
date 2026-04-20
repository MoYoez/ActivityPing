use super::super::*;
use crate::{
    models::{
        AppMessageRuleGroup, AppMessageTitleRule, AppTitleRuleMode, ClientConfig,
        DiscordRichPresenceButtonConfig, DiscordReportMode,
    },
    platform::{ForegroundSnapshot, MediaInfo},
};

fn chrome_snapshot(title: &str) -> ForegroundSnapshot {
    ForegroundSnapshot {
        process_name: "chrome.exe".into(),
        process_title: title.into(),
    }
}

fn button(label: &str, url: &str) -> DiscordRichPresenceButtonConfig {
    DiscordRichPresenceButtonConfig {
        label: label.into(),
        url: url.into(),
    }
}

fn sample_rule(title_buttons: Vec<DiscordRichPresenceButtonConfig>) -> AppMessageRuleGroup {
    AppMessageRuleGroup {
        process_match: "chrome.exe".into(),
        default_text: "Browsing".into(),
        title_rules: vec![AppMessageTitleRule {
            mode: AppTitleRuleMode::Plain,
            pattern: "哔哩哔哩".into(),
            text: "Watching Bilibili Now!".into(),
            buttons: title_buttons,
        }],
        buttons: vec![button("Open Website", "https://bilibili.com")],
        ..AppMessageRuleGroup::default()
    }
}

#[test]
fn title_subrule_keeps_group_buttons_when_title_buttons_are_empty() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        app_message_rules: vec![sample_rule(Vec::new())],
        ..ClientConfig::default()
    };

    let resolved = resolve_activity(
        &config,
        &chrome_snapshot("带你亲眼看看……_哔哩哔哩_bilibili - Google Chrome"),
        &MediaInfo::default(),
    )
    .expect("activity");

    assert_eq!(resolved.status_text.as_deref(), Some("Watching Bilibili Now!"));
    assert_eq!(resolved.discord_addons.buttons.len(), 1);
    assert_eq!(resolved.discord_addons.buttons[0].label, "Open Website");
}

#[test]
fn title_subrule_buttons_override_group_buttons_when_present() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        app_message_rules: vec![sample_rule(vec![button(
            "Open Video",
            "https://www.bilibili.com/video/BV1xx411c7mD",
        )])],
        ..ClientConfig::default()
    };

    let resolved = resolve_activity(
        &config,
        &chrome_snapshot("带你亲眼看看……_哔哩哔哩_bilibili - Google Chrome"),
        &MediaInfo::default(),
    )
    .expect("activity");

    assert_eq!(resolved.status_text.as_deref(), Some("Watching Bilibili Now!"));
    assert_eq!(resolved.discord_addons.buttons.len(), 1);
    assert_eq!(resolved.discord_addons.buttons[0].label, "Open Video");
}
