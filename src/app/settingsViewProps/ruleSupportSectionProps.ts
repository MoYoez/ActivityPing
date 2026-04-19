import type { ComponentProps } from "react";

import { RuleSupportSections } from "../../components/rules/RuleSupportSections";
import { appendUniqueListValue } from "../appConfig";
import {
  DEFAULT_HISTORY_RECORD_LIMIT,
  DEFAULT_HISTORY_TITLE_LIMIT,
  MAX_HISTORY_LIMIT,
  MIN_HISTORY_LIMIT,
} from "../appConstants";
import { clampHistoryLimit } from "../appHistory";
import type { CreateSettingsViewPropsArgs } from "../createSettingsViewProps";

export function createRuleSupportSectionProps(
  args: CreateSettingsViewPropsArgs,
): ComponentProps<typeof RuleSupportSections> {
  return {
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
}
