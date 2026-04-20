use crate::models::{AppMessageRuleGroup, AppMessageTitleRule};

use super::{normalize_discord_buttons, normalize_party_size};

pub(super) fn normalize_rule_groups(rules: &[AppMessageRuleGroup]) -> Vec<AppMessageRuleGroup> {
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
