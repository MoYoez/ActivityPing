import type { ComponentProps } from "react";

import { RuleGroupsEditorSection } from "../../components/rules/RuleGroupsEditorSection";
import { exportRulesJson, parseRulesJson } from "../../lib/rules";
import type { CreateSettingsViewPropsArgs } from "../createSettingsViewProps";
import { createAppMessageRuleGroup } from "../appConfig";
import { RULE_GROUP_PAGE_SIZE, TITLE_RULE_PAGE_SIZE } from "../appConstants";
import { clampPage, clampRuleIndex, moveItem, pageForIndex } from "../appFormatting";

export function createRuleGroupsSectionProps(
  args: CreateSettingsViewPropsArgs,
): ComponentProps<typeof RuleGroupsEditorSection> {
  return {
    rulesCount: args.config.appMessageRules.length,
    showProcessName: args.config.appMessageRulesShowProcessName,
    customOverrideEnabled: args.config.discordUseCustomAddonsOverride,
    rulesImportOpen: args.rulesImportOpen,
    rulesImportValue: args.rulesImportValue,
    activeRule: args.activeRule,
    activeRuleIndex: args.activeRuleIndex,
    pagedRuleGroups: args.pagedRuleGroups,
    ruleGroupPageStart: args.ruleGroupPageStart,
    safeRuleGroupPage: args.safeRuleGroupPage,
    ruleGroupTotalPages: args.ruleGroupTotalPages,
    pagedTitleRules: args.pagedTitleRules,
    titleRulePageStart: args.titleRulePageStart,
    safeTitleRulePage: args.safeTitleRulePage,
    titleRuleTotalPages: args.titleRuleTotalPages,
    activeTitleRuleCount: args.activeTitleRuleCount,
    appSuggestions: args.appSuggestions,
    activeRuleAdvancedAddonsConfigured: args.activeRuleAdvancedAddonsConfigured,
    customAddonsConfigured: args.customAddonsConfigured,
    panelClass: args.panelClass,
    cardClass: args.cardClass,
    panelHeadClass: args.panelHeadClass,
    fieldClass: args.fieldClass,
    fieldSpanClass: args.fieldSpanClass,
    inputClass: args.inputClass,
    textareaClass: args.textareaClass,
    selectClass: args.selectClass,
    buttonClass: args.buttonClass,
    primaryButtonClass: args.primaryButtonClass,
    dangerButtonClass: args.dangerButtonClass,
    badgeClass: args.badgeClass,
    subruleCardClass: args.subruleCardClass,
    toggleTileClass: args.toggleTileClass,
    onShowProcessNameChange: (value) => args.update("appMessageRulesShowProcessName", value),
    onCustomOverrideChange: (value) => args.update("discordUseCustomAddonsOverride", value),
    onAddRuleGroup: () => {
      const nextIndex = args.config.appMessageRules.length;
      args.setConfig((current) => ({
        ...current,
        appMessageRules: [...current.appMessageRules, createAppMessageRuleGroup()],
      }));
      args.setActiveRuleIndex(nextIndex);
      args.setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
      args.setTitleRulePage(0);
    },
    onCopyRulesJson: () =>
      void navigator.clipboard
        .writeText(exportRulesJson(args.config))
        .then(() => args.notify("success", "Rules copied", "The local rule JSON was copied to the clipboard."))
        .catch(() => args.notify("error", "Copy failed", "Clipboard access was not available.")),
    onToggleImport: () => args.setRulesImportOpen((current) => !current),
    onRulesImportValueChange: args.setRulesImportValue,
    onApplyImportedRules: () => {
      const parsed = parseRulesJson(args.rulesImportValue);
      if (!parsed.ok) {
        args.notify("error", "Import failed", parsed.error);
        return;
      }
      args.setConfig((current) => ({
        ...current,
        appMessageRules: parsed.data.appMessageRules,
        appMessageRulesShowProcessName: parsed.data.appMessageRulesShowProcessName,
        discordUseCustomAddonsOverride: parsed.data.discordUseCustomAddonsOverride,
        discordCustomPresets: parsed.data.discordCustomPresets,
        appFilterMode: parsed.data.appFilterMode,
        appBlacklist: parsed.data.appBlacklist,
        appWhitelist: parsed.data.appWhitelist,
        appNameOnlyList: parsed.data.appNameOnlyList,
        mediaPlaySourceBlocklist: parsed.data.mediaPlaySourceBlocklist,
      }));
      args.setRulesImportOpen(false);
      args.setRulesImportValue("");
      args.setActiveRuleIndex(0);
      args.setRuleGroupPage(0);
      args.setTitleRulePage(0);
      args.notify("success", "Rules imported", "The rule JSON was written into the current form.");
    },
    onSelectRule: (index) => {
      args.setActiveRuleIndex(index);
      args.setTitleRulePage(0);
    },
    onRuleGroupPageChange: (page) =>
      args.setRuleGroupPage(() => clampPage(page, args.config.appMessageRules.length, RULE_GROUP_PAGE_SIZE)),
    onMoveActiveRuleUp: () => {
      const nextIndex = args.activeRuleIndex - 1;
      args.setConfig((current) => ({
        ...current,
        appMessageRules: moveItem(current.appMessageRules, args.activeRuleIndex, nextIndex),
      }));
      args.setActiveRuleIndex(nextIndex);
      args.setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
    },
    onMoveActiveRuleDown: () => {
      const nextIndex = args.activeRuleIndex + 1;
      args.setConfig((current) => ({
        ...current,
        appMessageRules: moveItem(current.appMessageRules, args.activeRuleIndex, nextIndex),
      }));
      args.setActiveRuleIndex(nextIndex);
      args.setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
    },
    onDeleteActiveRule: () => {
      const nextIndex = clampRuleIndex(args.activeRuleIndex, args.config.appMessageRules.length - 1);
      args.setConfig((current) => ({
        ...current,
        appMessageRules: current.appMessageRules.filter((_, index) => index !== args.activeRuleIndex),
      }));
      args.setActiveRuleIndex(nextIndex);
      args.setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
      args.setTitleRulePage(0);
    },
    onActiveRuleProcessMatchChange: (value) =>
      args.patchRuleAt(args.activeRuleIndex, (rule) => ({ ...rule, processMatch: value })),
    onActiveRuleDefaultTextChange: (value) =>
      args.patchRuleAt(args.activeRuleIndex, (rule) => ({ ...rule, defaultText: value })),
    onAddTitleRule: () => {
      const nextIndex = args.activeRule?.titleRules.length ?? 0;
      args.patchRuleAt(args.activeRuleIndex, (rule) => ({
        ...rule,
        titleRules: [...rule.titleRules, { mode: "plain", pattern: "", text: "" }],
      }));
      args.setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
    },
    onTitleRulePageChange: (page) =>
      args.setTitleRulePage(() => clampPage(page, args.activeTitleRuleCount, TITLE_RULE_PAGE_SIZE)),
    onMoveTitleRuleUp: (titleRuleIndex) => {
      const nextIndex = titleRuleIndex - 1;
      args.patchRuleAt(args.activeRuleIndex, (rule) => ({
        ...rule,
        titleRules: moveItem(rule.titleRules, titleRuleIndex, nextIndex),
      }));
      args.setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
    },
    onMoveTitleRuleDown: (titleRuleIndex) => {
      const nextIndex = titleRuleIndex + 1;
      args.patchRuleAt(args.activeRuleIndex, (rule) => ({
        ...rule,
        titleRules: moveItem(rule.titleRules, titleRuleIndex, nextIndex),
      }));
      args.setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
    },
    onRemoveTitleRule: (titleRuleIndex) => {
      const nextIndex = clampRuleIndex(titleRuleIndex, (args.activeRule?.titleRules.length ?? 0) - 1);
      args.patchRuleAt(args.activeRuleIndex, (rule) => ({
        ...rule,
        titleRules: rule.titleRules.filter((_, index) => index !== titleRuleIndex),
      }));
      args.setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
    },
    onTitleRuleModeChange: (titleRuleIndex, mode) =>
      args.patchTitleRuleAt(args.activeRuleIndex, titleRuleIndex, (rule) => ({ ...rule, mode })),
    onTitleRulePatternChange: (titleRuleIndex, value) =>
      args.patchTitleRuleAt(args.activeRuleIndex, titleRuleIndex, (rule) => ({ ...rule, pattern: value })),
    onTitleRuleTextChange: (titleRuleIndex, value) =>
      args.patchTitleRuleAt(args.activeRuleIndex, titleRuleIndex, (rule) => ({ ...rule, text: value })),
    patchRuleAt: args.patchRuleAt,
    patchRuleDiscordButtonAt: args.patchRuleDiscordButtonAt,
    normalizePositiveNumberInput: args.normalizePositiveNumberInput,
  };
}
