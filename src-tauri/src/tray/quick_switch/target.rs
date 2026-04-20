use crate::models::DiscordReportMode;

use super::super::ids::{
    MENU_ID_SWITCH_APP, MENU_ID_SWITCH_CUSTOM_CURRENT, MENU_ID_SWITCH_CUSTOM_PRESET_PREFIX,
    MENU_ID_SWITCH_MIXED, MENU_ID_SWITCH_MUSIC,
};

#[derive(Clone)]
pub(in crate::tray) enum TraySwitchTarget {
    Mode(DiscordReportMode),
    CustomCurrent,
    CustomPreset(usize),
}

pub(in crate::tray) fn custom_preset_menu_id(index: usize) -> String {
    format!("{MENU_ID_SWITCH_CUSTOM_PRESET_PREFIX}{index}")
}

pub(in crate::tray) fn parse_switch_target(id: &str) -> Option<TraySwitchTarget> {
    match id {
        MENU_ID_SWITCH_MIXED => Some(TraySwitchTarget::Mode(DiscordReportMode::Mixed)),
        MENU_ID_SWITCH_MUSIC => Some(TraySwitchTarget::Mode(DiscordReportMode::Music)),
        MENU_ID_SWITCH_APP => Some(TraySwitchTarget::Mode(DiscordReportMode::App)),
        MENU_ID_SWITCH_CUSTOM_CURRENT => Some(TraySwitchTarget::CustomCurrent),
        _ => id
            .strip_prefix(MENU_ID_SWITCH_CUSTOM_PRESET_PREFIX)
            .and_then(|value| value.parse::<usize>().ok())
            .map(TraySwitchTarget::CustomPreset),
    }
}
