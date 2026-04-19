import {
  CUSTOM_PRESET_PAGE_SIZE,
  MAX_RUNTIME_LOGS,
  RUNTIME_LOG_PAGE_SIZE,
  RULE_GROUP_PAGE_SIZE,
  TITLE_RULE_PAGE_SIZE,
} from "./appConstants";
import { clampPage, pageCount } from "./appFormatting";
import type { AppMessageRuleGroup, ClientConfig, RealtimeReporterSnapshot } from "../types";

interface CreateAppPaginationStateArgs {
  config: ClientConfig;
  reporterSnapshot: RealtimeReporterSnapshot;
  activeRule: AppMessageRuleGroup | null;
  runtimeLogPage: number;
  ruleGroupPage: number;
  titleRulePage: number;
  customPresetPage: number;
  activeCustomPresetIndex: number | null;
}

export function createAppPaginationState({
  config,
  reporterSnapshot,
  activeRule,
  runtimeLogPage,
  ruleGroupPage,
  titleRulePage,
  customPresetPage,
  activeCustomPresetIndex,
}: CreateAppPaginationStateArgs) {
  const runtimeLogs = reporterSnapshot.logs.slice(0, MAX_RUNTIME_LOGS);
  const runtimeLogPageCount = pageCount(runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE);
  const safeRuntimeLogPage = clampPage(runtimeLogPage, runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE);
  const visibleRuntimeLogs = runtimeLogs.slice(
    safeRuntimeLogPage * RUNTIME_LOG_PAGE_SIZE,
    (safeRuntimeLogPage + 1) * RUNTIME_LOG_PAGE_SIZE,
  );
  const runtimeLogPageStart = runtimeLogs.length === 0 ? 0 : safeRuntimeLogPage * RUNTIME_LOG_PAGE_SIZE + 1;
  const runtimeLogPageEnd = Math.min(runtimeLogs.length, (safeRuntimeLogPage + 1) * RUNTIME_LOG_PAGE_SIZE);
  const ruleGroupTotalPages = pageCount(config.appMessageRules.length, RULE_GROUP_PAGE_SIZE);
  const safeRuleGroupPage = clampPage(ruleGroupPage, config.appMessageRules.length, RULE_GROUP_PAGE_SIZE);
  const ruleGroupPageStart = safeRuleGroupPage * RULE_GROUP_PAGE_SIZE;
  const pagedRuleGroups = config.appMessageRules.slice(ruleGroupPageStart, ruleGroupPageStart + RULE_GROUP_PAGE_SIZE);
  const activeTitleRuleCount = activeRule?.titleRules.length ?? 0;
  const titleRuleTotalPages = pageCount(activeTitleRuleCount, TITLE_RULE_PAGE_SIZE);
  const safeTitleRulePage = clampPage(titleRulePage, activeTitleRuleCount, TITLE_RULE_PAGE_SIZE);
  const titleRulePageStart = safeTitleRulePage * TITLE_RULE_PAGE_SIZE;
  const pagedTitleRules = activeRule?.titleRules.slice(titleRulePageStart, titleRulePageStart + TITLE_RULE_PAGE_SIZE) ?? [];
  const customPresetTotalPages = pageCount(config.discordCustomPresets.length, CUSTOM_PRESET_PAGE_SIZE);
  const safeCustomPresetPage = clampPage(customPresetPage, config.discordCustomPresets.length, CUSTOM_PRESET_PAGE_SIZE);
  const customPresetPageStart = safeCustomPresetPage * CUSTOM_PRESET_PAGE_SIZE;
  const pagedCustomPresets = config.discordCustomPresets.slice(
    customPresetPageStart,
    customPresetPageStart + CUSTOM_PRESET_PAGE_SIZE,
  );
  const activeCustomPreset =
    activeCustomPresetIndex === null ? null : config.discordCustomPresets[activeCustomPresetIndex] ?? null;
  const activeCustomPresetAdvancedAddonsConfigured = Boolean(
    activeCustomPreset?.partyId.trim() ||
      activeCustomPreset?.partySizeCurrent ||
      activeCustomPreset?.partySizeMax ||
      activeCustomPreset?.joinSecret.trim() ||
      activeCustomPreset?.spectateSecret.trim() ||
      activeCustomPreset?.matchSecret.trim(),
  );

  return {
    runtimeLogs,
    runtimeLogPageCount,
    safeRuntimeLogPage,
    visibleRuntimeLogs,
    runtimeLogPageStart,
    runtimeLogPageEnd,
    ruleGroupTotalPages,
    safeRuleGroupPage,
    ruleGroupPageStart,
    pagedRuleGroups,
    activeTitleRuleCount,
    titleRuleTotalPages,
    safeTitleRulePage,
    titleRulePageStart,
    pagedTitleRules,
    customPresetTotalPages,
    safeCustomPresetPage,
    customPresetPageStart,
    pagedCustomPresets,
    activeCustomPreset,
    activeCustomPresetAdvancedAddonsConfigured,
  };
}
