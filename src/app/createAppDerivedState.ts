import { DEFAULT_HISTORY_TITLE_LIMIT } from "./appConstants";
import { usesArtworkPublishing } from "./appConfig";
import { appHistoryRawTitles, clampHistoryLimit } from "./appHistory";
import {
  activityText,
  configSignature,
  discordReportModeName,
  localWorkingModeText,
} from "./appFormatting";
import {
  discordModeAppNameMode,
  discordModeCustomAppName,
  discordModeStatusDisplay,
} from "../components/discord/discordOptions";
import { SECTION_COPY, type ViewSection } from "../components/pages/pageSections";
import type {
  AppStatePayload,
  ClientConfig,
  DiscordPresenceSnapshot,
  RealtimeReporterSnapshot,
} from "../types";

interface CreateAppDerivedStateArgs {
  config: ClientConfig;
  baseState: AppStatePayload;
  activeSection: ViewSection;
  activeRuleIndex: number;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordSnapshot: DiscordPresenceSnapshot;
  appliedRuntimeConfigSignature: string | null;
}

export function createAppDerivedState({
  config,
  baseState,
  activeSection,
  activeRuleIndex,
  reporterSnapshot,
  discordSnapshot,
  appliedRuntimeConfigSignature,
}: CreateAppDerivedStateArgs) {
  const currentConfigSignature = configSignature(config);
  const savedConfigSignature = configSignature(baseState.config);
  const dirty = currentConfigSignature !== savedConfigSignature;
  const activeRule = activeRuleIndex >= 0 ? config.appMessageRules[activeRuleIndex] : null;
  const activeCopy = SECTION_COPY[activeSection];
  const discordReady = config.discordApplicationId.trim().length > 0;
  const customDiscordMode = config.discordReportMode === "custom";
  const activeDiscordModeName = discordReportModeName(config.discordReportMode);
  const activeDiscordStatusDisplay = discordModeStatusDisplay(config, config.discordReportMode);
  const activeDiscordAppNameMode = discordModeAppNameMode(config, config.discordReportMode);
  const activeDiscordCustomAppName = discordModeCustomAppName(config, config.discordReportMode);
  const customAppNameEnabled = activeDiscordAppNameMode === "custom";
  const customAdvancedAddonsConfigured = Boolean(
    config.discordCustomPartyId.trim() ||
      config.discordCustomPartySizeCurrent ||
      config.discordCustomPartySizeMax ||
      config.discordCustomJoinSecret.trim() ||
      config.discordCustomSpectateSecret.trim() ||
      config.discordCustomMatchSecret.trim(),
  );
  const customAddonsConfigured = Boolean(config.discordCustomButtons.length || customAdvancedAddonsConfigured);
  const activeRuleAdvancedAddonsConfigured = Boolean(
    activeRule?.partyId.trim() ||
      activeRule?.partySizeCurrent ||
      activeRule?.partySizeMax ||
      activeRule?.joinSecret.trim() ||
      activeRule?.spectateSecret.trim() ||
      activeRule?.matchSecret.trim(),
  );
  const historyTitleLimit = clampHistoryLimit(config.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
  const appRawTitleCount = baseState.appHistory.reduce((total, entry) => total + appHistoryRawTitles(entry).length, 0);
  const artworkPublishingMissing =
    usesArtworkPublishing(config) && config.discordArtworkWorkerUploadUrl.trim().length === 0;
  const runtimeAutostartEnabled = config.runtimeAutostartEnabled;
  const runtimeReady = baseState.config.discordApplicationId.trim().length > 0;
  const runtimeRunning = reporterSnapshot.running || discordSnapshot.running;
  const currentLocalModeText = localWorkingModeText(runtimeRunning ? baseState.config : config);
  const runtimeNeedsRestart =
    runtimeRunning &&
    !dirty &&
    Boolean(appliedRuntimeConfigSignature) &&
    appliedRuntimeConfigSignature !== savedConfigSignature;
  const hasCapturedActivity = Boolean(reporterSnapshot.currentActivity);
  const hasDiscordReport = Boolean(discordSnapshot.currentSummary?.trim());
  const runtimeBlockReason = runtimeReady
    ? null
    : discordReady
      ? "Save the RPC settings first to unlock runtime."
      : "Add a Discord application ID in Settings first.";
  const reportPreviewText = runtimeReady
    ? discordSnapshot.currentSummary?.trim() ||
      (hasCapturedActivity ? activityText(reporterSnapshot) : "Waiting for local activity.")
    : "Save the RPC settings first to unlock reporting.";
  const sidebarStatus = (() => {
    if (!runtimeReady) {
      if (discordReady && dirty) {
        return {
          label: "Save RPC settings",
          detail: "The application ID is only in the draft form. Save it before runtime can start.",
        };
      }
      return {
        label: "RPC not configured",
        detail: "Add and save the Discord application ID first.",
      };
    }
    if (reporterSnapshot.running && discordSnapshot.running) {
      if (!discordSnapshot.connected) {
        return {
          label: "Waiting for Discord",
          detail: "Capture is active. Discord RPC is running but the client is not connected yet.",
        };
      }
      if (hasDiscordReport) {
        return {
          label: "Reporting to Discord",
          detail: "Local capture is active and the processed activity is being pushed into Discord.",
        };
      }
      if (hasCapturedActivity) {
        return {
          label: "Syncing to Discord",
          detail: "Local activity was captured. Waiting for the next Discord presence update to land.",
        };
      }
      return {
        label: "Waiting for activity",
        detail: "Local capture and Discord RPC are running, but no activity has passed the current rules yet.",
      };
    }
    if (reporterSnapshot.running) {
      return {
        label: "Capture running",
        detail: "Local capture is active, but Discord RPC is not running.",
      };
    }
    if (discordSnapshot.running) {
      return {
        label: "RPC ready",
        detail: "Discord RPC is running and waiting for local capture to start.",
      };
    }
    return {
      label: "Ready to start",
      detail: "The RPC profile is saved. Start runtime when you want to report activity.",
    };
  })();

  return {
    currentConfigSignature,
    savedConfigSignature,
    dirty,
    activeRule,
    activeCopy,
    discordReady,
    customDiscordMode,
    activeDiscordModeName,
    activeDiscordStatusDisplay,
    activeDiscordAppNameMode,
    activeDiscordCustomAppName,
    customAppNameEnabled,
    customAdvancedAddonsConfigured,
    customAddonsConfigured,
    activeRuleAdvancedAddonsConfigured,
    historyTitleLimit,
    appRawTitleCount,
    artworkPublishingMissing,
    runtimeAutostartEnabled,
    runtimeReady,
    runtimeRunning,
    currentLocalModeText,
    runtimeNeedsRestart,
    hasCapturedActivity,
    hasDiscordReport,
    runtimeBlockReason,
    reportPreviewText,
    sidebarStatus,
  };
}
