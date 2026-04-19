use super::{super::*, fixtures::sample_resolved};

#[test]
fn mixed_mode_can_publish_rule_buttons() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        ..ClientConfig::default()
    };
    let mut resolved = sample_resolved();
    resolved.discord_addons = ResolvedDiscordAddons {
        buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
            label: "Open".into(),
            url: "https://example.com".into(),
        }],
        ..ResolvedDiscordAddons::default()
    };

    let addons = select_presence_addons(&config, &resolved);
    let buttons = build_presence_buttons(&addons);

    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0].label, "Open");
}

#[test]
fn custom_addons_override_rule_buttons_when_enabled() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_use_custom_addons_override: true,
        discord_custom_buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
            label: "Profile".into(),
            url: "https://example.com/profile".into(),
        }],
        ..ClientConfig::default()
    };
    let mut resolved = sample_resolved();
    resolved.discord_addons = ResolvedDiscordAddons {
        buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
            label: "Rule".into(),
            url: "https://example.com/rule".into(),
        }],
        ..ResolvedDiscordAddons::default()
    };

    let addons = select_presence_addons(&config, &resolved);
    let buttons = build_presence_buttons(&addons);

    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0].label, "Profile");
}
