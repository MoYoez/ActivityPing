import type { AppMessageRuleGroup } from "../../types";
export function summarizeRuleGroup(rule: AppMessageRuleGroup) {
  const parts = [];
  parts.push(rule.defaultText || "No default text");
  if (rule.titleRules.length > 0) {
    parts.push(`${rule.titleRules.length} title rule${rule.titleRules.length === 1 ? "" : "s"}`);
  }
  if (rule.buttons.length > 0) {
    parts.push(`${rule.buttons.length} button${rule.buttons.length === 1 ? "" : "s"}`);
  }
  if (rule.partyId.trim() || rule.partySizeCurrent || rule.partySizeMax) {
    parts.push("party");
  }
  if (rule.joinSecret.trim() || rule.spectateSecret.trim() || rule.matchSecret.trim()) {
    parts.push("secrets");
  }
  return parts.join(" · ");
}
