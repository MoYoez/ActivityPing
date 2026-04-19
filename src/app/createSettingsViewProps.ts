import type { Dispatch, SetStateAction } from "react";
import type { ComponentProps } from "react";

import { DiscordBridgeView } from "../components/discord/DiscordBridgeView";
import { RulesDialogContent } from "../components/rules/RulesDialogContent";
import { RuleGroupsEditorSection } from "../components/rules/RuleGroupsEditorSection";
import { RuleSupportSections } from "../components/rules/RuleSupportSections";
import { SettingsPage } from "../components/pages/SettingsPage";
import { GeneralSettingsSections } from "../components/settings/GeneralSettingsSections";
import { RulesLauncherCard } from "../components/settings/RulesLauncherCard";
import { exportRulesJson, parseRulesJson } from "../lib/rules";
import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordActivityType,
  DiscordAppNameMode,
  DiscordReportMode,
  DiscordRichPresenceButtonConfig,
  DiscordStatusDisplay,
  PlatformSelfTestResult,
} from "../types";

import { appendUniqueListValue, createAppMessageRuleGroup, createDiscordButton } from "./appConfig";
import {
  DEFAULT_HISTORY_RECORD_LIMIT,
  DEFAULT_HISTORY_TITLE_LIMIT,
  MAX_HISTORY_LIMIT,
  MIN_HISTORY_LIMIT,
  RULE_GROUP_PAGE_SIZE,
  TITLE_RULE_PAGE_SIZE,
} from "./appConstants";
import { clampHistoryLimit } from "./appHistory";
import { clampPage, clampRuleIndex, moveItem, pageForIndex } from "./appFormatting";

type UpdateConfig = <K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) => void;

interface CreateSettingsViewPropsArgs {
  capabilities: ClientCapabilities;
  runtimeAutostartEnabled: boolean;
  config: ClientConfig;
  baseState: AppStatePayload;
  platformSelfTest: PlatformSelfTestResult | null;
  currentLocalModeText: string;
  busy: Record<string, boolean>;
  appSuggestions: string[];
  playSourceSuggestions: string[];
  blacklistInput: string;
  whitelistInput: string;
  nameOnlyInput: string;
  mediaSourceInput: string;
  rulesImportOpen: boolean;
  rulesImportValue: string;
  activeRule: AppMessageRuleGroup | null;
  activeRuleIndex: number;
  pagedRuleGroups: AppMessageRuleGroup[];
  ruleGroupPageStart: number;
  safeRuleGroupPage: number;
  ruleGroupTotalPages: number;
  pagedTitleRules: AppMessageTitleRule[];
  titleRulePageStart: number;
  safeTitleRulePage: number;
  titleRuleTotalPages: number;
  activeTitleRuleCount: number;
  activeRuleAdvancedAddonsConfigured: boolean;
  customAddonsConfigured: boolean;
  historyRecordLimit: number;
  historyTitleLimit: number;
  appRawTitleCount: number;
  activeDiscordModeName: string;
  activeDiscordStatusDisplay: DiscordStatusDisplay;
  activeDiscordAppNameMode: DiscordAppNameMode;
  activeDiscordCustomAppName: string;
  customAppNameEnabled: boolean;
  customDiscordMode: boolean;
  customAdvancedAddonsConfigured: boolean;
  discordDetailsForceCustomChoice: boolean;
  discordStateForceCustomChoice: boolean;
  artworkPublishingMissing: boolean;
  discordConnected: boolean;
  discordRunning: boolean;
  discordCurrentSummary: string | null;
  discordLastError: string | null;
  discordActivityTypeText: (value: DiscordActivityType) => string;
  discordReportModeText: (config: ClientConfig) => string;
  panelClass: string;
  cardClass: string;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  textareaClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  badgeClass: string;
  goodBadgeClass: string;
  statCardClass: string;
  subruleCardClass: string;
  toggleTileClass: string;
  radioCardClass: string;
  activeRadioCardClass: string;
  formatDate: (value?: string | null) => string;
  normalizePositiveNumberInput: (value: string) => number | null;
  update: UpdateConfig;
  updateRuntimeAutostart: (enabled: boolean) => void;
  updateDiscordModeSettings: (patch: {
    statusDisplay?: DiscordStatusDisplay;
    appNameMode?: DiscordAppNameMode;
    customAppName?: string;
  }) => void;
  patchRuleAt: (index: number, updater: (rule: AppMessageRuleGroup) => AppMessageRuleGroup) => void;
  patchTitleRuleAt: (
    ruleIndex: number,
    titleRuleIndex: number,
    updater: (rule: AppMessageTitleRule) => AppMessageTitleRule,
  ) => void;
  patchDiscordButtonAt: (
    index: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) => void;
  patchRuleDiscordButtonAt: (
    ruleIndex: number,
    buttonIndex: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) => void;
  persistPayload: (payload: AppStatePayload, syncConfig: boolean) => Promise<void>;
  notify: (tone: "info" | "success" | "warn" | "error", title: string, detail: string) => void;
  onHideToTray: () => void;
  onRunSelfTest: () => void;
  onRequestAccessibilityPermission: () => void;
  onSaveCurrentCustomSettingsAsPreset: () => void;
  onOpenCustomPresets: () => void;
  onOpenRules: () => void;
  setConfig: Dispatch<SetStateAction<ClientConfig>>;
  setRulesImportOpen: Dispatch<SetStateAction<boolean>>;
  setRulesImportValue: Dispatch<SetStateAction<string>>;
  setActiveRuleIndex: Dispatch<SetStateAction<number>>;
  setRuleGroupPage: Dispatch<SetStateAction<number>>;
  setTitleRulePage: Dispatch<SetStateAction<number>>;
  setBlacklistInput: Dispatch<SetStateAction<string>>;
  setWhitelistInput: Dispatch<SetStateAction<string>>;
  setNameOnlyInput: Dispatch<SetStateAction<string>>;
  setMediaSourceInput: Dispatch<SetStateAction<string>>;
  setDiscordDetailsForceCustomChoice: Dispatch<SetStateAction<boolean>>;
  setDiscordStateForceCustomChoice: Dispatch<SetStateAction<boolean>>;
}

export function createSettingsViewProps(args: CreateSettingsViewPropsArgs) {
  const generalSettingsProps: ComponentProps<typeof GeneralSettingsSections> = {
    capabilities: args.capabilities,
    runtimeAutostartEnabled: args.runtimeAutostartEnabled,
    launchOnStartup: args.config.launchOnStartup,
    pollIntervalMs: args.config.pollIntervalMs,
    heartbeatIntervalMs: args.config.heartbeatIntervalMs,
    platformSelfTest: args.platformSelfTest,
    currentLocalModeText: args.currentLocalModeText,
    ruleGroupCount: args.config.appMessageRules.length,
    savedAppsCount: args.baseState.appHistory.length,
    mediaSourceCount: args.baseState.playSourceHistory.length,
    panelClass: args.panelClass,
    panelHeadClass: args.panelHeadClass,
    fieldClass: args.fieldClass,
    inputClass: args.inputClass,
    buttonClass: args.buttonClass,
    statCardClass: args.statCardClass,
    toggleTileClass: args.toggleTileClass,
    busyPlatformSelfTest: args.busy.platformSelfTest,
    busyAccessibilityPermission: args.busy.accessibilityPermission,
    onRuntimeAutostartChange: args.updateRuntimeAutostart,
    onLaunchOnStartupChange: (value) => args.update("launchOnStartup", value),
    onPollIntervalChange: (value) => args.update("pollIntervalMs", Number(value) || 1000),
    onHeartbeatIntervalChange: (value) => args.update("heartbeatIntervalMs", Number(value) || 0),
    onHideToTray: args.onHideToTray,
    onRunSelfTest: args.onRunSelfTest,
    onRequestAccessibilityPermission: args.onRequestAccessibilityPermission,
  };

  const ruleGroupsSectionProps: ComponentProps<typeof RuleGroupsEditorSection> = {
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

  const ruleSupportSectionProps: ComponentProps<typeof RuleSupportSections> = {
    appFilterMode: args.config.appFilterMode,
    appBlacklist: args.config.appBlacklist,
    appWhitelist: args.config.appWhitelist,
    appNameOnlyList: args.config.appNameOnlyList,
    mediaPlaySourceBlocklist: args.config.mediaPlaySourceBlocklist,
    appSuggestions: args.appSuggestions,
    playSourceSuggestions: args.playSourceSuggestions,
    blacklistInput: args.blacklistInput,
    whitelistInput: args.whitelistInput,
    nameOnlyInput: args.nameOnlyInput,
    mediaSourceInput: args.mediaSourceInput,
    captureReportedAppsEnabled: args.config.captureReportedAppsEnabled,
    historyRecordLimit: args.historyRecordLimit,
    historyTitleLimit: args.historyTitleLimit,
    appHistory: args.baseState.appHistory,
    playSourceHistory: args.baseState.playSourceHistory,
    appRawTitleCount: args.appRawTitleCount,
    panelClass: args.panelClass,
    panelHeadClass: args.panelHeadClass,
    fieldClass: args.fieldClass,
    inputClass: args.inputClass,
    badgeClass: args.badgeClass,
    buttonClass: args.buttonClass,
    dangerButtonClass: args.dangerButtonClass,
    toggleTileClass: args.toggleTileClass,
    minHistoryLimit: MIN_HISTORY_LIMIT,
    maxHistoryLimit: MAX_HISTORY_LIMIT,
    onAppFilterModeChange: (mode) => args.update("appFilterMode", mode),
    onBlacklistInputChange: args.setBlacklistInput,
    onWhitelistInputChange: args.setWhitelistInput,
    onNameOnlyInputChange: args.setNameOnlyInput,
    onMediaSourceInputChange: args.setMediaSourceInput,
    onAddBlacklist: () => {
      const value = args.blacklistInput.trim();
      if (!value) return;
      args.setConfig((current) => ({ ...current, appBlacklist: appendUniqueListValue(current.appBlacklist, value, false) }));
      args.setBlacklistInput("");
    },
    onAddWhitelist: () => {
      const value = args.whitelistInput.trim();
      if (!value) return;
      args.setConfig((current) => ({ ...current, appWhitelist: appendUniqueListValue(current.appWhitelist, value, false) }));
      args.setWhitelistInput("");
    },
    onAddNameOnly: () => {
      const value = args.nameOnlyInput.trim();
      if (!value) return;
      args.setConfig((current) => ({ ...current, appNameOnlyList: appendUniqueListValue(current.appNameOnlyList, value, false) }));
      args.setNameOnlyInput("");
    },
    onAddMediaSource: () => {
      const value = args.mediaSourceInput.trim().toLowerCase();
      if (!value) return;
      args.setConfig((current) => ({
        ...current,
        mediaPlaySourceBlocklist: appendUniqueListValue(current.mediaPlaySourceBlocklist, value, true),
      }));
      args.setMediaSourceInput("");
    },
    onRemoveBlacklist: (index) =>
      args.setConfig((current) => ({ ...current, appBlacklist: current.appBlacklist.filter((_, itemIndex) => itemIndex !== index) })),
    onRemoveWhitelist: (index) =>
      args.setConfig((current) => ({ ...current, appWhitelist: current.appWhitelist.filter((_, itemIndex) => itemIndex !== index) })),
    onRemoveNameOnly: (index) =>
      args.setConfig((current) => ({ ...current, appNameOnlyList: current.appNameOnlyList.filter((_, itemIndex) => itemIndex !== index) })),
    onRemoveMediaSource: (index) =>
      args.setConfig((current) => ({
        ...current,
        mediaPlaySourceBlocklist: current.mediaPlaySourceBlocklist.filter((_, itemIndex) => itemIndex !== index),
      })),
    onCaptureReportedAppsChange: (value) => args.update("captureReportedAppsEnabled", value),
    onHistoryRecordLimitChange: (value) =>
      args.update("captureHistoryRecordLimit", clampHistoryLimit(value, DEFAULT_HISTORY_RECORD_LIMIT)),
    onHistoryTitleLimitChange: (value) =>
      args.update("captureHistoryTitleLimit", clampHistoryLimit(value, DEFAULT_HISTORY_TITLE_LIMIT)),
    formatDate: args.formatDate,
    onCopyHistoryJson: () =>
      void navigator.clipboard
        .writeText(JSON.stringify({ appHistory: args.baseState.appHistory, playSourceHistory: args.baseState.playSourceHistory }, null, 2))
        .then(() => args.notify("success", "History copied", "Local history records were copied to the clipboard."))
        .catch(() => args.notify("error", "Copy failed", "Clipboard access was not available.")),
    onClearHistory: () => {
      const payload = { ...args.baseState, appHistory: [], playSourceHistory: [] };
      void args.persistPayload(payload, false).then(() =>
        args.notify("info", "History cleared", "Local rule suggestion history was cleared."),
      );
    },
  };

  const rulesDialogContentProps: ComponentProps<typeof RulesDialogContent> = {
    ruleGroupsSectionProps,
    ruleSupportSectionProps,
  };

  const rulesLauncherProps: ComponentProps<typeof RulesLauncherCard> = {
    panelClass: args.panelClass,
    panelHeadClass: args.panelHeadClass,
    badgeClass: args.badgeClass,
    primaryButtonClass: args.primaryButtonClass,
    ruleGroupCount: args.config.appMessageRules.length,
    appFilterMode: args.config.appFilterMode,
    nameOnlyCount: args.config.appNameOnlyList.length,
    mediaBlockCount: args.config.mediaPlaySourceBlocklist.length,
    onOpenRules: args.onOpenRules,
  };

  const discordBridgeProps: ComponentProps<typeof DiscordBridgeView> = {
    config: args.config,
    discordConnected: args.discordConnected,
    activeDiscordModeName: args.activeDiscordModeName,
    activeDiscordStatusDisplay: args.activeDiscordStatusDisplay,
    activeDiscordAppNameMode: args.activeDiscordAppNameMode,
    activeDiscordCustomAppName: args.activeDiscordCustomAppName,
    customAppNameEnabled: args.customAppNameEnabled,
    customDiscordMode: args.customDiscordMode,
    customAdvancedAddonsConfigured: args.customAdvancedAddonsConfigured,
    discordDetailsForceCustomChoice: args.discordDetailsForceCustomChoice,
    discordStateForceCustomChoice: args.discordStateForceCustomChoice,
    artworkPublishingMissing: args.artworkPublishingMissing,
    panelClass: args.panelClass,
    panelHeadClass: args.panelHeadClass,
    fieldClass: args.fieldClass,
    fieldSpanClass: args.fieldSpanClass,
    inputClass: args.inputClass,
    selectClass: args.selectClass,
    buttonClass: args.buttonClass,
    primaryButtonClass: args.primaryButtonClass,
    dangerButtonClass: args.dangerButtonClass,
    badgeClass: args.badgeClass,
    goodBadgeClass: args.goodBadgeClass,
    statCardClass: args.statCardClass,
    toggleTileClass: args.toggleTileClass,
    radioCardClass: args.radioCardClass,
    activeRadioCardClass: args.activeRadioCardClass,
    discordActivityTypeText: args.discordActivityTypeText,
    discordReportModeText: args.discordReportModeText,
    linkStateText: args.discordRunning ? (args.discordConnected ? "Connected" : "Waiting for Discord") : "Stopped",
    currentSummaryText: args.discordCurrentSummary || "No local activity is being mirrored yet.",
    lastErrorText: args.discordLastError || "No Discord runtime error recorded.",
    onDiscordApplicationIdChange: (value) => args.update("discordApplicationId", value),
    onDiscordReportModeChange: (value: DiscordReportMode) => args.update("discordReportMode", value),
    onDiscordModeSettingsChange: args.updateDiscordModeSettings,
    onDiscordActivityTypeChange: (value) => args.update("discordActivityType", value),
    onDiscordDetailsForceCustomChoiceChange: args.setDiscordDetailsForceCustomChoice,
    onDiscordStateForceCustomChoiceChange: args.setDiscordStateForceCustomChoice,
    onDiscordDetailsFormatChange: (value) => args.update("discordDetailsFormat", value),
    onDiscordStateFormatChange: (value) => args.update("discordStateFormat", value),
    onPatchDiscordButtonAt: args.patchDiscordButtonAt,
    onRemoveDiscordButtonAt: (index) =>
      args.setConfig((current) => ({
        ...current,
        discordCustomButtons: current.discordCustomButtons.filter((_, itemIndex) => itemIndex !== index),
      })),
    onAddDiscordButton: () =>
      args.setConfig((current) => ({
        ...current,
        discordCustomButtons: [...current.discordCustomButtons, createDiscordButton()],
      })),
    onDiscordCustomPartyIdChange: (value) => args.update("discordCustomPartyId", value),
    onDiscordCustomPartySizeCurrentChange: (value) =>
      args.update("discordCustomPartySizeCurrent", args.normalizePositiveNumberInput(value)),
    onDiscordCustomPartySizeMaxChange: (value) =>
      args.update("discordCustomPartySizeMax", args.normalizePositiveNumberInput(value)),
    onDiscordCustomJoinSecretChange: (value) => args.update("discordCustomJoinSecret", value),
    onDiscordCustomSpectateSecretChange: (value) => args.update("discordCustomSpectateSecret", value),
    onDiscordCustomMatchSecretChange: (value) => args.update("discordCustomMatchSecret", value),
    onSaveCurrentCustomSettingsAsPreset: args.onSaveCurrentCustomSettingsAsPreset,
    onOpenCustomPresets: args.onOpenCustomPresets,
    onDiscordSmartEnableMusicCountdownChange: (value) => args.update("discordSmartEnableMusicCountdown", value),
    onDiscordSmartShowAppNameChange: (value) => args.update("discordSmartShowAppName", value),
    onReportStoppedMediaChange: (value) => args.update("reportStoppedMedia", value),
    onDiscordUseAppArtworkChange: (value) => args.update("discordUseAppArtwork", value),
    onDiscordUseMusicArtworkChange: (value) => args.update("discordUseMusicArtwork", value),
    onDiscordArtworkWorkerUploadUrlChange: (value) => args.update("discordArtworkWorkerUploadUrl", value),
    onDiscordArtworkWorkerTokenChange: (value) => args.update("discordArtworkWorkerToken", value),
  };

  const settingsPageProps: ComponentProps<typeof SettingsPage> = {
    discordBridgeProps,
    rulesLauncherProps,
    generalSettingsProps,
  };

  return {
    settingsPageProps,
    rulesDialogContentProps,
  };
}
