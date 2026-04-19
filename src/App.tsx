import { startTransition, useEffect, useMemo, useRef } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useShallow } from "zustand/react/shallow";

import "./App.css";
import {
  ACTIVE_RADIO_CARD_CLASS,
  BADGE_CLASS,
  BUTTON_CLASS,
  CARD_CLASS,
  CUSTOM_PRESET_PAGE_SIZE,
  DANGER_BUTTON_CLASS,
  DEFAULT_HISTORY_RECORD_LIMIT,
  DEFAULT_HISTORY_TITLE_LIMIT,
  FIELD_CLASS,
  FIELD_SPAN_CLASS,
  GITHUB_URL,
  GOOD_BADGE_CLASS,
  INPUT_CLASS,
  MAX_RUNTIME_LOGS,
  PANEL_CLASS,
  PANEL_HEAD_CLASS,
  PRIMARY_BUTTON_CLASS,
  RADIO_CARD_CLASS,
  RUNTIME_LOG_PAGE_SIZE,
  RULE_GROUP_PAGE_SIZE,
  SELECT_CLASS,
  STAT_CARD_CLASS,
  SUBRULE_CARD_CLASS,
  TEXTAREA_CLASS,
  TITLE_RULE_PAGE_SIZE,
  TOGGLE_TILE_CLASS,
} from "./app/appConstants";
import {
  buildPayload,
  createDiscordCustomPresetFromConfig,
  normalizePositiveNumberInput,
  summarizeDiscordCustomPreset,
  validateArtworkPublishing,
  validateRuleRegex,
} from "./app/appConfig";
import { createOverlayProps } from "./app/createOverlayProps";
import {
  appHistoryRawTitles,
  clampHistoryLimit,
  mergeAppHistory,
  mergePlaySourceHistory,
  normalizeAppHistory,
  normalizePlaySourceHistory,
  shouldCaptureHistoryActivity,
  uniqueHistoryValues,
} from "./app/appHistory";
import {
  activityMeta,
  activityText,
  alertClass,
  captureModeText,
  clampPage,
  clampRuleIndex,
  configSignature,
  discordActivityTypeText,
  discordReportModeName,
  discordReportModeText,
  formatDate,
  limitReporterSnapshotLogs,
  localWorkingModeText,
  logEntryClass,
  pageCount,
  pageForIndex,
  sameJsonValue,
} from "./app/appFormatting";
import { createSettingsViewProps } from "./app/createSettingsViewProps";
import { createRuntimeViewProps } from "./app/createRuntimeViewProps";
import appIcon from "./assets/app-icon-base.png";
import { AppOverlays } from "./components/app/AppOverlays";
import { AboutPage } from "./components/pages/AboutPage";
import { RuntimePage } from "./components/pages/RuntimePage";
import { SECTION_COPY, SECTION_ORDER, type ViewSection } from "./components/pages/pageSections";
import { SettingsPage } from "./components/pages/SettingsPage";
import {
  discordModeAppNameMode,
  discordModeCustomAppName,
  discordModeStatusDisplay,
  normalizeDiscordLineTemplate,
  patchDiscordModeSettings,
} from "./components/discord/discordOptions";
import { AppShellLayout } from "./components/layout/AppShellLayout";
import { RulesDialogContent } from "./components/rules/RulesDialogContent";
import {
  getClientCapabilities,
  getDiscordPresenceSnapshot,
  getRealtimeReporterSnapshot,
  hideToTray,
  loadAppState,
  requestAccessibilityPermission,
  resolveApiError,
  runPlatformSelfTest,
  saveAppState,
  setAutostartEnabled,
  startDiscordPresenceSync,
  startRealtimeReporter,
  stopDiscordPresenceSync,
  stopRealtimeReporter,
} from "./lib/api";
import { normalizeClientConfig } from "./lib/rules";
import { useAppUiStore, type NoticeTone } from "./store/appUiStore";
import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppStatePayload,
  ClientConfig,
  DiscordAppNameMode,
  DiscordCustomPreset,
  DiscordDebugPayload,
  DiscordRichPresenceButtonConfig,
  DiscordStatusDisplay,
  ReporterLogEntry,
} from "./types";

function App() {
  const {
    capabilities,
    baseState,
    config,
    reporterSnapshot,
    discordSnapshot,
    platformSelfTest,
    notices,
    hydrated,
    busy,
    activeSection,
    activeRuleIndex,
    rulesImportOpen,
    rulesImportValue,
    blacklistInput,
    whitelistInput,
    nameOnlyInput,
    mediaSourceInput,
    rulesDialogOpen,
    customRulesDialogOpen,
    customPresetPage,
    activeCustomPresetIndex,
    discordDetailsForceCustomChoice,
    discordStateForceCustomChoice,
    presetDetailsForceCustomChoice,
    presetStateForceCustomChoice,
    discardDialogOpen,
    appliedRuntimeConfigSignature,
    ruleGroupPage,
    titleRulePage,
    runtimeLogPage,
    jsonViewer,
    setCapabilities,
    setBaseState,
    setConfig,
    setReporterSnapshot,
    setDiscordSnapshot,
    setPlatformSelfTest,
    setHydrated,
    setBusy,
    setActiveSection,
    setActiveRuleIndex,
    setRulesImportOpen,
    setRulesImportValue,
    setBlacklistInput,
    setWhitelistInput,
    setNameOnlyInput,
    setMediaSourceInput,
    setRulesDialogOpen,
    setCustomRulesDialogOpen,
    setCustomPresetPage,
    setActiveCustomPresetIndex,
    setDiscordDetailsForceCustomChoice,
    setDiscordStateForceCustomChoice,
    setPresetDetailsForceCustomChoice,
    setPresetStateForceCustomChoice,
    setDiscardDialogOpen,
    setAppliedRuntimeConfigSignature,
    setRuleGroupPage,
    setTitleRulePage,
    setRuntimeLogPage,
    setJsonViewer,
    addNotice,
    removeNotice,
  } = useAppUiStore(
    useShallow((state) => ({
      capabilities: state.capabilities,
      baseState: state.baseState,
      config: state.config,
      reporterSnapshot: state.reporterSnapshot,
      discordSnapshot: state.discordSnapshot,
      platformSelfTest: state.platformSelfTest,
      notices: state.notices,
      hydrated: state.hydrated,
      busy: state.busy,
      activeSection: state.activeSection,
      activeRuleIndex: state.activeRuleIndex,
      rulesImportOpen: state.rulesImportOpen,
      rulesImportValue: state.rulesImportValue,
      blacklistInput: state.blacklistInput,
      whitelistInput: state.whitelistInput,
      nameOnlyInput: state.nameOnlyInput,
      mediaSourceInput: state.mediaSourceInput,
      rulesDialogOpen: state.rulesDialogOpen,
      customRulesDialogOpen: state.customRulesDialogOpen,
      customPresetPage: state.customPresetPage,
      activeCustomPresetIndex: state.activeCustomPresetIndex,
      discordDetailsForceCustomChoice: state.discordDetailsForceCustomChoice,
      discordStateForceCustomChoice: state.discordStateForceCustomChoice,
      presetDetailsForceCustomChoice: state.presetDetailsForceCustomChoice,
      presetStateForceCustomChoice: state.presetStateForceCustomChoice,
      discardDialogOpen: state.discardDialogOpen,
      appliedRuntimeConfigSignature: state.appliedRuntimeConfigSignature,
      ruleGroupPage: state.ruleGroupPage,
      titleRulePage: state.titleRulePage,
      runtimeLogPage: state.runtimeLogPage,
      jsonViewer: state.jsonViewer,
      setCapabilities: state.setCapabilities,
      setBaseState: state.setBaseState,
      setConfig: state.setConfig,
      setReporterSnapshot: state.setReporterSnapshot,
      setDiscordSnapshot: state.setDiscordSnapshot,
      setPlatformSelfTest: state.setPlatformSelfTest,
      setHydrated: state.setHydrated,
      setBusy: state.setBusy,
      setActiveSection: state.setActiveSection,
      setActiveRuleIndex: state.setActiveRuleIndex,
      setRulesImportOpen: state.setRulesImportOpen,
      setRulesImportValue: state.setRulesImportValue,
      setBlacklistInput: state.setBlacklistInput,
      setWhitelistInput: state.setWhitelistInput,
      setNameOnlyInput: state.setNameOnlyInput,
      setMediaSourceInput: state.setMediaSourceInput,
      setRulesDialogOpen: state.setRulesDialogOpen,
      setCustomRulesDialogOpen: state.setCustomRulesDialogOpen,
      setCustomPresetPage: state.setCustomPresetPage,
      setActiveCustomPresetIndex: state.setActiveCustomPresetIndex,
      setDiscordDetailsForceCustomChoice: state.setDiscordDetailsForceCustomChoice,
      setDiscordStateForceCustomChoice: state.setDiscordStateForceCustomChoice,
      setPresetDetailsForceCustomChoice: state.setPresetDetailsForceCustomChoice,
      setPresetStateForceCustomChoice: state.setPresetStateForceCustomChoice,
      setDiscardDialogOpen: state.setDiscardDialogOpen,
      setAppliedRuntimeConfigSignature: state.setAppliedRuntimeConfigSignature,
      setRuleGroupPage: state.setRuleGroupPage,
      setTitleRulePage: state.setTitleRulePage,
      setRuntimeLogPage: state.setRuntimeLogPage,
      setJsonViewer: state.setJsonViewer,
      addNotice: state.addNotice,
      removeNotice: state.removeNotice,
    })),
  );
  const runtimeAutostartAttemptedRef = useRef(false);

  function notify(tone: NoticeTone, title: string, detail: string) {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    addNotice({ id, tone, title, detail });
    window.setTimeout(() => {
      startTransition(() => removeNotice(id));
    }, 4200);
  }

  async function refreshReporter() {
    const result = await getRealtimeReporterSnapshot();
    if (result.success && result.data) setReporterSnapshot(limitReporterSnapshotLogs(result.data));
  }

  async function refreshDiscord() {
    const result = await getDiscordPresenceSnapshot();
    if (result.success && result.data) setDiscordSnapshot(result.data);
  }

  async function persistPayload(payload: AppStatePayload, syncConfig: boolean) {
    await saveAppState(payload);
    setBaseState(payload);
    if (syncConfig) setConfig(payload.config);
  }

  async function persist(nextConfig: ClientConfig, syncConfig = true) {
    await persistPayload(buildPayload(baseState, nextConfig), syncConfig);
  }

  async function refreshPlatformSelfTest(showNotice = false) {
    const result = await runPlatformSelfTest();
    if (!result.success || !result.data) {
      if (showNotice) {
        notify("error", "Self-test failed", resolveApiError(result, "Platform self-test could not be completed."));
      }
      return;
    }

    setPlatformSelfTest(result.data);
    if (showNotice) {
      notify("info", "Self-test updated", "Platform capture checks were refreshed.");
    }
  }

  async function requestPlatformAccessibilityPermission() {
    const result = await requestAccessibilityPermission();
    if (!result.success) {
      notify("warn", "Permission request unavailable", resolveApiError(result, "Accessibility permission could not be requested."));
      return;
    }

    if (result.data) {
      notify("success", "Permission granted", "Accessibility permission is available for window title capture.");
    } else {
      notify(
        "info",
        "Permission prompt opened",
        "Grant Accessibility permission in System Settings > Privacy & Security > Accessibility, then refresh the self-test.",
      );
    }

    await refreshPlatformSelfTest(false);
  }

  async function saveProfile(successTitle: string, successDetail: string) {
    const normalized = normalizeClientConfig(config);
    const regexError = validateRuleRegex(normalized);
    if (regexError) {
      notify("error", "Rules not saved", regexError);
      return;
    }
    const artworkPublishingError = validateArtworkPublishing(normalized);
    if (artworkPublishingError) {
      setActiveSection("settings");
      notify("warn", "Artwork publishing required", artworkPublishingError);
      return;
    }
    try {
      if (capabilities.autostart && normalized.launchOnStartup !== baseState.config.launchOnStartup) {
        await setAutostartEnabled(normalized.launchOnStartup);
      }
      await persist(normalized);
      notify("success", successTitle, successDetail);
    } catch (error) {
      notify(
        "error",
        "Save failed",
        error instanceof Error ? error.message : "The current settings could not be saved.",
      );
    }
  }

  function discardDraftChanges() {
    setConfig(normalizeClientConfig(baseState.config));
    setRulesImportOpen(false);
    setRulesImportValue("");
    notify("info", "Draft reverted", "The current form was reset to the last saved settings.");
  }

  async function startRuntimeSession() {
    if (!runtimeReady) {
      notify("warn", "Runtime locked", runtimeBlockReason || "Save the RPC settings first.");
      return;
    }

    const normalized = normalizeClientConfig(config);
    let startedDiscordThisRun = false;

    if (!discordSnapshot.running) {
      const discordResult = await startDiscordPresenceSync(normalized);
      if (!discordResult.success || !discordResult.data) {
        notify("error", "Runtime start failed", resolveApiError(discordResult, "Discord RPC could not be started."));
        return;
      }
      startedDiscordThisRun = true;
      setDiscordSnapshot(discordResult.data);
    }

    if (!reporterSnapshot.running) {
      const reporterResult = await startRealtimeReporter(normalized);
      if (!reporterResult.success || !reporterResult.data) {
        if (startedDiscordThisRun) {
          const rollbackResult = await stopDiscordPresenceSync();
          if (rollbackResult.success && rollbackResult.data) {
            setDiscordSnapshot(rollbackResult.data);
          }
        }
        notify("error", "Runtime start failed", resolveApiError(reporterResult, "The local monitor could not be started."));
        return;
      }
      setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
    }

    setAppliedRuntimeConfigSignature(configSignature(normalized));
    notify("success", "Runtime online", "Local capture and Discord RPC are now running.");
  }

  async function stopRuntimeSession() {
    const failures: string[] = [];

    if (reporterSnapshot.running) {
      const reporterResult = await stopRealtimeReporter();
      if (!reporterResult.success || !reporterResult.data) {
        failures.push(resolveApiError(reporterResult, "The local monitor could not be stopped."));
      } else {
        setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
      }
    }

    if (discordSnapshot.running) {
      const discordResult = await stopDiscordPresenceSync();
      if (!discordResult.success || !discordResult.data) {
        failures.push(resolveApiError(discordResult, "Discord RPC could not be stopped."));
      } else {
        setDiscordSnapshot(discordResult.data);
      }
    }

    if (failures.length > 0) {
      notify("warn", "Runtime partially stopped", failures[0]);
      return;
    }

    setAppliedRuntimeConfigSignature(null);
    notify("info", "Runtime stopped", "Local capture and Discord RPC have been stopped.");
  }

  async function restartRuntimeSession() {
    if (!runtimeReady) {
      notify("warn", "Runtime locked", runtimeBlockReason || "Save the RPC settings first.");
      return;
    }

    const normalized = normalizeClientConfig(config);
    const failures: string[] = [];

    if (reporterSnapshot.running) {
      const reporterResult = await stopRealtimeReporter();
      if (!reporterResult.success || !reporterResult.data) {
        failures.push(resolveApiError(reporterResult, "The local monitor could not be stopped."));
      } else {
        setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
      }
    }

    if (discordSnapshot.running) {
      const discordResult = await stopDiscordPresenceSync();
      if (!discordResult.success || !discordResult.data) {
        failures.push(resolveApiError(discordResult, "Discord RPC could not be stopped."));
      } else {
        setDiscordSnapshot(discordResult.data);
      }
    }

    if (failures.length > 0) {
      notify("warn", "Runtime restart blocked", failures[0]);
      return;
    }

    const discordResult = await startDiscordPresenceSync(normalized);
    if (!discordResult.success || !discordResult.data) {
      setAppliedRuntimeConfigSignature(null);
      notify("error", "Runtime restart failed", resolveApiError(discordResult, "Discord RPC could not be started."));
      return;
    }
    setDiscordSnapshot(discordResult.data);

    const reporterResult = await startRealtimeReporter(normalized);
    if (!reporterResult.success || !reporterResult.data) {
      const rollbackResult = await stopDiscordPresenceSync();
      if (rollbackResult.success && rollbackResult.data) {
        setDiscordSnapshot(rollbackResult.data);
      }
      setAppliedRuntimeConfigSignature(null);
      notify("error", "Runtime restart failed", resolveApiError(reporterResult, "The local monitor could not be started."));
      return;
    }

    setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
    setAppliedRuntimeConfigSignature(configSignature(normalized));
    notify("success", "Runtime restarted", "Saved configuration changes are now running.");
  }

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const caps = await getClientCapabilities();
      if (!cancelled && caps.success && caps.data) setCapabilities(caps.data);
      const state = await loadAppState();
      if (cancelled) return;
      const resolvedConfig = normalizeClientConfig(state.config);
      const historyRecordLimit = clampHistoryLimit(resolvedConfig.captureHistoryRecordLimit, DEFAULT_HISTORY_RECORD_LIMIT);
      const historyTitleLimit = clampHistoryLimit(resolvedConfig.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
      const payload = {
        config: resolvedConfig,
        appHistory: normalizeAppHistory(state.appHistory, historyRecordLimit, historyTitleLimit),
        playSourceHistory: normalizePlaySourceHistory(state.playSourceHistory, historyRecordLimit),
        locale: "en-US",
      };
      setBaseState(payload);
      setConfig(payload.config);
      setHydrated(true);
      await refreshReporter();
      await refreshDiscord();
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!hydrated || (!reporterSnapshot.running && !discordSnapshot.running)) return;
    const timer = window.setInterval(() => {
      if (document.visibilityState !== "visible") return;
      void refreshReporter();
      void refreshDiscord();
    }, 4000);
    return () => window.clearInterval(timer);
  }, [hydrated, reporterSnapshot.running, discordSnapshot.running]);

  useEffect(() => {
    if (
      !hydrated ||
      runtimeAutostartAttemptedRef.current ||
      !baseState.config.runtimeAutostartEnabled ||
      !baseState.config.discordApplicationId.trim() ||
      reporterSnapshot.running ||
      discordSnapshot.running
    ) {
      return;
    }

    runtimeAutostartAttemptedRef.current = true;
    const timer = window.setTimeout(() => {
      void runAction("startRuntime", startRuntimeSession);
    }, 1200);
    return () => window.clearTimeout(timer);
  }, [
    hydrated,
    baseState.config.runtimeAutostartEnabled,
    baseState.config.discordApplicationId,
    reporterSnapshot.running,
    discordSnapshot.running,
  ]);

  useEffect(() => {
    const runtimeActive = reporterSnapshot.running || discordSnapshot.running;
    if (!hydrated || !runtimeActive || appliedRuntimeConfigSignature) return;
    setAppliedRuntimeConfigSignature(configSignature(baseState.config));
  }, [hydrated, reporterSnapshot.running, discordSnapshot.running, appliedRuntimeConfigSignature, baseState.config]);

  useEffect(() => {
    const runtimeActive = reporterSnapshot.running || discordSnapshot.running;
    if (runtimeActive || !appliedRuntimeConfigSignature) return;
    setAppliedRuntimeConfigSignature(null);
  }, [reporterSnapshot.running, discordSnapshot.running, appliedRuntimeConfigSignature]);

  useEffect(() => {
    setActiveRuleIndex((current) => clampRuleIndex(current, config.appMessageRules.length));
  }, [config.appMessageRules.length]);

  useEffect(() => {
    setRuleGroupPage((current) => {
      if (activeRuleIndex >= 0) {
        return pageForIndex(activeRuleIndex, RULE_GROUP_PAGE_SIZE);
      }
      return clampPage(current, config.appMessageRules.length, RULE_GROUP_PAGE_SIZE);
    });
  }, [activeRuleIndex, config.appMessageRules.length]);

  useEffect(() => {
    setTitleRulePage(0);
  }, [activeRuleIndex]);

  useEffect(() => {
    const activeTitleRuleCount = config.appMessageRules[activeRuleIndex]?.titleRules.length ?? 0;
    setTitleRulePage((current) => clampPage(current, activeTitleRuleCount, TITLE_RULE_PAGE_SIZE));
  }, [activeRuleIndex, config.appMessageRules]);

  useEffect(() => {
    setCustomPresetPage((current) => clampPage(current, config.discordCustomPresets.length, CUSTOM_PRESET_PAGE_SIZE));
  }, [config.discordCustomPresets.length]);

  useEffect(() => {
    if (activeCustomPresetIndex !== null && !config.discordCustomPresets[activeCustomPresetIndex]) {
      setActiveCustomPresetIndex(null);
    }
  }, [activeCustomPresetIndex, config.discordCustomPresets]);

  useEffect(() => {
    if (activeCustomPresetIndex === null) {
      setPresetDetailsForceCustomChoice(false);
      setPresetStateForceCustomChoice(false);
    }
  }, [activeCustomPresetIndex]);

  useEffect(() => {
    if (!rulesDialogOpen && !customRulesDialogOpen && activeCustomPresetIndex === null && !discardDialogOpen && !jsonViewer) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (jsonViewer) {
          setJsonViewer(null);
          return;
        }
        if (activeCustomPresetIndex !== null) {
          setActiveCustomPresetIndex(null);
          return;
        }
        if (discardDialogOpen) {
          setDiscardDialogOpen(false);
          return;
        }
        if (customRulesDialogOpen) {
          setCustomRulesDialogOpen(false);
          return;
        }
        setRulesDialogOpen(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [rulesDialogOpen, customRulesDialogOpen, activeCustomPresetIndex, discardDialogOpen, jsonViewer]);

  useEffect(() => {
    if (!hydrated || !config.captureReportedAppsEnabled) return;
    const historyRecordLimit = clampHistoryLimit(config.captureHistoryRecordLimit, DEFAULT_HISTORY_RECORD_LIMIT);
    const historyTitleLimit = clampHistoryLimit(config.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
    const nextAppHistory = shouldCaptureHistoryActivity(reporterSnapshot.currentActivity)
      ? mergeAppHistory(baseState.appHistory, reporterSnapshot.currentActivity, historyRecordLimit, historyTitleLimit)
      : baseState.appHistory;
    const nextPlaySourceHistory = mergePlaySourceHistory(
      baseState.playSourceHistory,
      reporterSnapshot.currentActivity,
      historyRecordLimit,
    );
    if (sameJsonValue(nextAppHistory, baseState.appHistory) && sameJsonValue(nextPlaySourceHistory, baseState.playSourceHistory)) return;
    const payload = { ...baseState, appHistory: nextAppHistory, playSourceHistory: nextPlaySourceHistory };
    void saveAppState(payload).then(() => setBaseState(payload)).catch(() => {});
  }, [
    hydrated,
    config.captureReportedAppsEnabled,
    config.captureHistoryRecordLimit,
    config.captureHistoryTitleLimit,
    reporterSnapshot.currentActivity?.processName,
    reporterSnapshot.currentActivity?.processTitle,
    reporterSnapshot.currentActivity?.rawProcessTitle,
    reporterSnapshot.currentActivity?.statusText,
    reporterSnapshot.currentActivity?.mediaTitle,
    reporterSnapshot.currentActivity?.mediaArtist,
    reporterSnapshot.currentActivity?.mediaAlbum,
    reporterSnapshot.currentActivity?.mediaSummary,
    reporterSnapshot.currentActivity?.playSource,
    baseState,
  ]);

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
  const historyRecordLimit = clampHistoryLimit(config.captureHistoryRecordLimit, DEFAULT_HISTORY_RECORD_LIMIT);
  const historyTitleLimit = clampHistoryLimit(config.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
  const appRawTitleCount = baseState.appHistory.reduce((total, entry) => total + appHistoryRawTitles(entry).length, 0);
  const artworkPublishingMissing =
    (config.discordUseAppArtwork || config.discordUseMusicArtwork) &&
    config.discordArtworkWorkerUploadUrl.trim().length === 0;
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
  const appSuggestions = useMemo(
    () => uniqueHistoryValues(baseState.appHistory.map((entry) => entry.processName)),
    [baseState.appHistory],
  );
  const playSourceSuggestions = useMemo(
    () => uniqueHistoryValues(baseState.playSourceHistory.map((entry) => entry.source)),
    [baseState.playSourceHistory],
  );
  const runtimeLogs = useMemo(() => reporterSnapshot.logs.slice(0, MAX_RUNTIME_LOGS), [reporterSnapshot.logs]);
  const runtimeLogPageCount = pageCount(runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE);
  const safeRuntimeLogPage = clampPage(runtimeLogPage, runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE);
  const visibleRuntimeLogs = useMemo(
    () => runtimeLogs.slice(safeRuntimeLogPage * RUNTIME_LOG_PAGE_SIZE, (safeRuntimeLogPage + 1) * RUNTIME_LOG_PAGE_SIZE),
    [safeRuntimeLogPage, runtimeLogs],
  );
  const runtimeLogPageStart = runtimeLogs.length === 0 ? 0 : safeRuntimeLogPage * RUNTIME_LOG_PAGE_SIZE + 1;
  const runtimeLogPageEnd = Math.min(runtimeLogs.length, (safeRuntimeLogPage + 1) * RUNTIME_LOG_PAGE_SIZE);
  useEffect(() => {
    setRuntimeLogPage((current) => clampPage(current, runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE));
  }, [runtimeLogs.length]);
  const discordDebugPayload = useMemo<DiscordDebugPayload | null>(
    () => discordSnapshot.debugPayload ?? null,
    [discordSnapshot.debugPayload],
  );
  const jsonViewerJson = useMemo(() => (jsonViewer?.value ? JSON.stringify(jsonViewer.value, null, 2) : ""), [jsonViewer]);
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

  function update<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
    setConfig((current) => ({ ...current, [key]: value }));
  }

  function updateDiscordModeSettings(patch: {
    statusDisplay?: DiscordStatusDisplay;
    appNameMode?: DiscordAppNameMode;
    customAppName?: string;
  }) {
    setConfig((current) => patchDiscordModeSettings(current, current.discordReportMode, patch));
  }

  function updateRuntimeAutostart(enabled: boolean) {
    setConfig((current) => ({
      ...current,
      runtimeAutostartEnabled: enabled,
    }));
  }

  function patchRuleAt(index: number, updater: (rule: AppMessageRuleGroup) => AppMessageRuleGroup) {
    setConfig((current) => {
      const next = [...current.appMessageRules];
      if (!next[index]) {
        return current;
      }
      next[index] = updater(next[index]);
      return { ...current, appMessageRules: next };
    });
  }

  function patchTitleRuleAt(ruleIndex: number, titleRuleIndex: number, updater: (rule: AppMessageTitleRule) => AppMessageTitleRule) {
    patchRuleAt(ruleIndex, (rule) => {
      const next = [...rule.titleRules];
      if (!next[titleRuleIndex]) {
        return rule;
      }
      next[titleRuleIndex] = updater(next[titleRuleIndex]);
      return { ...rule, titleRules: next };
    });
  }

  function patchDiscordCustomPresetAt(index: number, updater: (preset: DiscordCustomPreset) => DiscordCustomPreset) {
    setConfig((current) => {
      const next = [...current.discordCustomPresets];
      if (!next[index]) {
        return current;
      }
      next[index] = updater(next[index]);
      return { ...current, discordCustomPresets: next };
    });
  }

  function patchDiscordButtonAt(index: number, updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig) {
    setConfig((current) => {
      const next = [...current.discordCustomButtons];
      if (!next[index]) {
        return current;
      }
      next[index] = updater(next[index]);
      return { ...current, discordCustomButtons: next };
    });
  }

  function patchRuleDiscordButtonAt(
    ruleIndex: number,
    buttonIndex: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) {
    patchRuleAt(ruleIndex, (rule) => {
      const next = [...rule.buttons];
      if (!next[buttonIndex]) {
        return rule;
      }
      next[buttonIndex] = updater(next[buttonIndex]);
      return { ...rule, buttons: next };
    });
  }

  function saveCurrentCustomSettingsAsPreset() {
    const nextPreset = createDiscordCustomPresetFromConfig(config);
    const nextIndex = config.discordCustomPresets.length;
    setConfig((current) => ({
      ...current,
      discordCustomPresets: [...current.discordCustomPresets, nextPreset],
    }));
    setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
    setActiveCustomPresetIndex(nextIndex);
    setCustomRulesDialogOpen(true);
  }

  function applyDiscordCustomPreset(preset: DiscordCustomPreset) {
    setConfig((current) => ({
      ...current,
      discordReportMode: "custom",
      discordActivityType: preset.activityType,
      discordCustomModeStatusDisplay: preset.statusDisplay,
      discordCustomModeAppNameMode: preset.appNameMode,
      discordCustomModeCustomAppName: preset.customAppName,
      discordDetailsFormat: normalizeDiscordLineTemplate(preset.detailsFormat),
      discordStateFormat: normalizeDiscordLineTemplate(preset.stateFormat),
      discordCustomButtons: preset.buttons.map((button) => ({ ...button })),
      discordCustomPartyId: preset.partyId,
      discordCustomPartySizeCurrent: preset.partySizeCurrent ?? null,
      discordCustomPartySizeMax: preset.partySizeMax ?? null,
      discordCustomJoinSecret: preset.joinSecret,
      discordCustomSpectateSecret: preset.spectateSecret,
      discordCustomMatchSecret: preset.matchSecret,
    }));
  }

  async function runAction(name: string, work: () => Promise<void>) {
    setBusy((current) => ({ ...current, [name]: true }));
    try {
      await work();
    } catch (error) {
      notify(
        "error",
        "Action failed",
        error instanceof Error ? error.message : "The requested action could not be completed.",
      );
    } finally {
      setBusy((current) => ({ ...current, [name]: false }));
    }
  }

  function openDiscordPayloadJson() {
    setJsonViewer({
      eyebrow: "Debug",
      title: "Discord payload JSON",
      description: "Current payload snapshot from the moment this dialog opened.",
      value: discordDebugPayload,
      emptyText:
        "No Discord payload has been published yet. Start runtime and wait for a captured activity to pass the current rules.",
    });
  }

  function openLogPayloadJson(entry: ReporterLogEntry) {
    setJsonViewer({
      eyebrow: "Log",
      title: "Reported record JSON",
      description: `${entry.title} · ${formatDate(entry.timestamp)}`,
      value: entry.payload ?? null,
      emptyText: "This log entry does not include a reported record payload.",
    });
  }

  const { settingsPageProps, rulesDialogContentProps } = createSettingsViewProps({
    capabilities,
    runtimeAutostartEnabled,
    config,
    baseState,
    platformSelfTest: platformSelfTest ?? null,
    currentLocalModeText,
    busy,
    appSuggestions,
    playSourceSuggestions,
    blacklistInput,
    whitelistInput,
    nameOnlyInput,
    mediaSourceInput,
    rulesImportOpen,
    rulesImportValue,
    activeRule,
    activeRuleIndex,
    pagedRuleGroups,
    ruleGroupPageStart,
    safeRuleGroupPage,
    ruleGroupTotalPages,
    pagedTitleRules,
    titleRulePageStart,
    safeTitleRulePage,
    titleRuleTotalPages,
    activeTitleRuleCount,
    activeRuleAdvancedAddonsConfigured,
    customAddonsConfigured,
    historyRecordLimit,
    historyTitleLimit,
    appRawTitleCount,
    activeDiscordModeName,
    activeDiscordStatusDisplay,
    activeDiscordAppNameMode,
    activeDiscordCustomAppName,
    customAppNameEnabled,
    customDiscordMode,
    customAdvancedAddonsConfigured,
    discordDetailsForceCustomChoice,
    discordStateForceCustomChoice,
    artworkPublishingMissing,
    discordConnected: discordSnapshot.connected,
    discordRunning: discordSnapshot.running,
    discordCurrentSummary: discordSnapshot.currentSummary ?? null,
    discordLastError: discordSnapshot.lastError ?? null,
    discordActivityTypeText,
    discordReportModeText,
    panelClass: PANEL_CLASS,
    cardClass: CARD_CLASS,
    panelHeadClass: PANEL_HEAD_CLASS,
    fieldClass: FIELD_CLASS,
    fieldSpanClass: FIELD_SPAN_CLASS,
    inputClass: INPUT_CLASS,
    textareaClass: TEXTAREA_CLASS,
    selectClass: SELECT_CLASS,
    buttonClass: BUTTON_CLASS,
    primaryButtonClass: PRIMARY_BUTTON_CLASS,
    dangerButtonClass: DANGER_BUTTON_CLASS,
    badgeClass: BADGE_CLASS,
    goodBadgeClass: GOOD_BADGE_CLASS,
    statCardClass: STAT_CARD_CLASS,
    subruleCardClass: SUBRULE_CARD_CLASS,
    toggleTileClass: TOGGLE_TILE_CLASS,
    radioCardClass: RADIO_CARD_CLASS,
    activeRadioCardClass: ACTIVE_RADIO_CARD_CLASS,
    formatDate,
    normalizePositiveNumberInput,
    update,
    updateRuntimeAutostart,
    updateDiscordModeSettings,
    patchRuleAt,
    patchTitleRuleAt,
    patchDiscordButtonAt,
    patchRuleDiscordButtonAt,
    persistPayload,
    notify,
    onHideToTray: () =>
      void hideToTray().catch((error: unknown) =>
        notify("warn", "Tray hide failed", error instanceof Error ? error.message : "The window could not be hidden."),
      ),
    onRunSelfTest: () => runAction("platformSelfTest", async () => refreshPlatformSelfTest(true)),
    onRequestAccessibilityPermission: () =>
      runAction("accessibilityPermission", requestPlatformAccessibilityPermission),
    onSaveCurrentCustomSettingsAsPreset: saveCurrentCustomSettingsAsPreset,
    onOpenCustomPresets: () => setCustomRulesDialogOpen(true),
    onOpenRules: () => setRulesDialogOpen(true),
    setConfig,
    setRulesImportOpen,
    setRulesImportValue,
    setActiveRuleIndex,
    setRuleGroupPage,
    setTitleRulePage,
    setBlacklistInput,
    setWhitelistInput,
    setNameOnlyInput,
    setMediaSourceInput,
    setDiscordDetailsForceCustomChoice,
    setDiscordStateForceCustomChoice,
  });

  const rulesDialogContent = <RulesDialogContent {...rulesDialogContentProps} />;

  const settingsView = <SettingsPage {...settingsPageProps} />;

  const { runtimePageProps, aboutPageProps } = createRuntimeViewProps({
    runtimeReady,
    runtimeBlockReason,
    runtimeRunning,
    currentActivityText: activityText(reporterSnapshot),
    attachedMeta: activityMeta(reporterSnapshot),
    captureModeText: captureModeText(config),
    lastHeartbeatText: formatDate(reporterSnapshot.lastHeartbeatAt),
    runtimeLogs,
    visibleRuntimeLogs,
    runtimeLogPageStart,
    runtimeLogPageEnd,
    safeRuntimeLogPage,
    runtimeLogPageCount,
    discordDebugPayload,
    panelClass: PANEL_CLASS,
    panelHeadClass: PANEL_HEAD_CLASS,
    statCardClass: STAT_CARD_CLASS,
    primaryButtonClass: PRIMARY_BUTTON_CLASS,
    buttonClass: BUTTON_CLASS,
    goodBadgeClass: GOOD_BADGE_CLASS,
    badgeClass: BADGE_CLASS,
    appIconSrc: appIcon,
    githubUrl: GITHUB_URL,
    openGithubBusy: busy.openGithub,
    startBusy: busy.startRuntime,
    stopBusy: busy.stopRuntime,
    refreshBusy: busy.refreshRuntime,
    restartBusy: busy.restartRuntime,
    formatDate,
    logEntryClass,
    onOpenSettings: () => setActiveSection("settings"),
    onStart: () => runAction("startRuntime", startRuntimeSession),
    onStop: () => runAction("stopRuntime", stopRuntimeSession),
    onRefresh: () =>
      runAction("refreshRuntime", async () => {
        await refreshReporter();
        await refreshDiscord();
        notify("info", "Runtime refreshed", "The live status panels were updated.");
      }),
    onRuntimeLogPageChange: (page) => setRuntimeLogPage(() => clampPage(page, runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE)),
    onOpenLogPayload: openLogPayloadJson,
    onOpenDiscordPayload: openDiscordPayloadJson,
    onOpenGithub: () =>
      runAction("openGithub", async () => {
        await openUrl(GITHUB_URL);
      }),
  });

  const runtimeView = <RuntimePage {...runtimePageProps} />;

  const aboutView = <AboutPage {...aboutPageProps} />;

  const activeSectionView = activeSection === "settings" ? settingsView : activeSection === "about" ? aboutView : runtimeView;

  const {
    rulesEditorDialogProps,
    customPresetsDialogProps,
    customPresetEditorProps,
    discardDialogProps,
    jsonViewerDialogProps,
    notificationsProps,
  } = createOverlayProps({
    config,
    activeCustomPresetIndex,
    pagedCustomPresets,
    customPresetPageStart,
    safeCustomPresetPage,
    customPresetTotalPages,
    activeCustomPreset,
    activeCustomPresetAdvancedAddonsConfigured,
    presetDetailsForceCustomChoice,
    presetStateForceCustomChoice,
    rulesDialogOpen,
    customRulesDialogOpen,
    discardDialogOpen,
    jsonViewer,
    jsonViewerJson,
    runtimeNeedsRestart,
    dirty,
    notices,
    busy,
    panelHeadClass: PANEL_HEAD_CLASS,
    buttonClass: BUTTON_CLASS,
    primaryButtonClass: PRIMARY_BUTTON_CLASS,
    dangerButtonClass: DANGER_BUTTON_CLASS,
    subruleCardClass: SUBRULE_CARD_CLASS,
    fieldClass: FIELD_CLASS,
    fieldSpanClass: FIELD_SPAN_CLASS,
    inputClass: INPUT_CLASS,
    selectClass: SELECT_CLASS,
    normalizePositiveNumberInput,
    summarizePreset: summarizeDiscordCustomPreset,
    alertClass,
    applyPreset: applyDiscordCustomPreset,
    patchPresetAt: patchDiscordCustomPresetAt,
    onCloseRulesDialog: () => setRulesDialogOpen(false),
    onSaveCurrentAsPreset: saveCurrentCustomSettingsAsPreset,
    onCloseCustomPresets: () => setCustomRulesDialogOpen(false),
    onCloseDiscardDialog: () => setDiscardDialogOpen(false),
    onConfirmDiscard: () => {
      discardDraftChanges();
      setDiscardDialogOpen(false);
    },
    onCloseJsonViewer: () => setJsonViewer(null),
    onRestartRuntime: () => runAction("restartRuntime", restartRuntimeSession),
    onOpenDiscardDialog: () => setDiscardDialogOpen(true),
    onSaveDraft: () => runAction("saveDraft", async () => saveProfile("Changes saved", "The current settings were saved.")),
    setConfig,
    setCustomPresetPage,
    setActiveCustomPresetIndex,
    setCustomRulesDialogOpen,
    setPresetDetailsForceCustomChoice,
    setPresetStateForceCustomChoice,
  });

  return (
    <>
      <AppShellLayout
        appIconSrc={appIcon}
        activeSection={activeSection}
        sections={SECTION_ORDER.map((section) => ({
          id: section,
          kicker: SECTION_COPY[section].kicker,
          title: SECTION_COPY[section].title,
        }))}
        sidebarStatusLabel={sidebarStatus.label}
        sidebarStatusDetail={sidebarStatus.detail}
        reportPreviewText={reportPreviewText}
        activeKicker={activeCopy.kicker}
        activeTitle={activeCopy.title}
        activeDescription={activeCopy.description}
        onSectionChange={(section) => setActiveSection(section as ViewSection)}
      >
        {activeSectionView}
      </AppShellLayout>
      <AppOverlays
        rulesDialogOpen={rulesDialogOpen}
        rulesEditorDialogProps={rulesEditorDialogProps}
        rulesDialogContent={rulesDialogContent}
        customRulesDialogOpen={customRulesDialogOpen}
        customPresetsDialogProps={customPresetsDialogProps}
        customPresetEditorProps={customPresetEditorProps}
        discardDialogOpen={discardDialogOpen}
        discardDialogProps={discardDialogProps}
        jsonViewerDialogProps={jsonViewerDialogProps}
        notificationsProps={notificationsProps}
      />
    </>
  );
}

export default App;
