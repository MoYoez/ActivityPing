mod preset;
mod target;

use crate::{backend_locale::BackendLocale, models::ClientConfig};

use super::notice::AppliedSelection;

pub(super) fn active_custom_preset_index(config: &ClientConfig) -> Option<usize> {
    preset::active_custom_preset_index(config)
}

pub(super) fn custom_preset_menu_id(index: usize) -> String {
    target::custom_preset_menu_id(index)
}

pub(super) fn parse_switch_target(id: &str) -> Option<target::TraySwitchTarget> {
    target::parse_switch_target(id)
}

pub(super) fn apply_switch_target(
    config: &mut ClientConfig,
    target: &target::TraySwitchTarget,
    locale: BackendLocale,
) -> Result<AppliedSelection, String> {
    preset::apply_switch_target(config, target, locale)
}

pub(super) fn configs_equal(left: &ClientConfig, right: &ClientConfig) -> bool {
    preset::configs_equal(left, right)
}
