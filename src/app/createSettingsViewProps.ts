import type { Dispatch, SetStateAction } from "react";
import type { ComponentProps } from "react";

import { RulesDialogContent } from "../components/rules/RulesDialogContent";
import { SettingsPage } from "../components/pages/SettingsPage";
import { GeneralSettingsSections } from "../components/settings/GeneralSettingsSections";
import { RulesLauncherCard } from "../components/settings/RulesLauncherCard";
import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordActivityType,
  DiscordAppNameMode,
  DiscordRichPresenceButtonConfig,
  DiscordStatusDisplay,
  PlatformSelfTestResult,
} from "../types";

import { createDiscordBridgeProps } from "./settingsViewProps/discordBridgeProps";
import { createRuleGroupsSectionProps } from "./settingsViewProps/ruleGroupsSectionProps";
import { createRuleSupportSectionProps } from "./settingsViewProps/ruleSupportSectionProps";

type UpdateConfig = <K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) => void;

export interface CreateSettingsViewPropsArgs {
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
  appliedCustomPresetName: string | null;
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
  onImportDiscordCustomAssetFiles: (files: File[]) => void;
  onDeleteDiscordCustomAsset: (assetId: string) => void;
  setConfig: Dispatch<SetStateAction<ClientConfig>>;
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

  const ruleGroupsSectionProps = createRuleGroupsSectionProps(args);

  const ruleSupportSectionProps = createRuleSupportSectionProps(args);

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

  const discordBridgeProps = createDiscordBridgeProps(args);

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
