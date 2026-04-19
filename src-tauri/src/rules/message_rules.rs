use crate::models::{AppMessageRuleGroup, AppMessageTitleRule, AppTitleRuleMode};

use super::{
    non_empty, normalize_discord_buttons, normalize_party_size, ResolvedDiscordAddons,
    ResolvedDiscordParty, ResolvedDiscordSecrets,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MessageRuleMatch {
    pub(super) status_text: Option<String>,
    pub(super) addons: ResolvedDiscordAddons,
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

pub(super) fn match_message_rule(
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
