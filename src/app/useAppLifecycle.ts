import { useEffect, useRef, type SetStateAction } from "react";

import {
  CUSTOM_PRESET_PAGE_SIZE,
  DEFAULT_HISTORY_TITLE_LIMIT,
  RUNTIME_LOG_PAGE_SIZE,
  RULE_GROUP_PAGE_SIZE,
  TITLE_RULE_PAGE_SIZE,
} from "./appConstants";
import {
  clampHistoryLimit,
  mergeAppHistory,
  mergePlaySourceHistory,
  normalizeAppHistory,
  normalizePlaySourceHistory,
  shouldCaptureHistoryActivity,
} from "./appHistory";
import { clampPage, clampRuleIndex, configSignature, pageForIndex, sameJsonValue } from "./appFormatting";
import { getClientCapabilities, loadAppState, saveAppState } from "../lib/api";
import { normalizeClientConfig } from "../lib/rules";
import type { JsonViewerState } from "../store/appUiStore";
import type {
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  RealtimeReporterSnapshot,
} from "../types";

interface UseAppLifecycleArgs {
  activeCustomPresetIndex: number | null;
  activeRuleIndex: number;
  appliedRuntimeConfigSignature: string | null;
  baseState: AppStatePayload;
  config: ClientConfig;
  customRulesDialogOpen: boolean;
  discardDialogOpen: boolean;
  discordSnapshot: DiscordPresenceSnapshot;
  hydrated: boolean;
  jsonViewer: JsonViewerState | null;
  reporterSnapshot: RealtimeReporterSnapshot;
  rulesDialogOpen: boolean;
  runtimeLogsLength: number;
  refreshDiscord: () => Promise<void>;
  refreshReporter: () => Promise<void>;
  runAction: (name: string, work: () => Promise<void>) => Promise<void>;
  startRuntimeSession: () => Promise<void>;
  setActiveCustomPresetIndex: (value: SetStateAction<number | null>) => void;
  setActiveRuleIndex: (value: SetStateAction<number>) => void;
  setAppliedRuntimeConfigSignature: (value: SetStateAction<string | null>) => void;
  setBaseState: (value: SetStateAction<AppStatePayload>) => void;
  setCapabilities: (value: SetStateAction<ClientCapabilities>) => void;
  setConfig: (value: SetStateAction<ClientConfig>) => void;
  setCustomPresetPage: (value: SetStateAction<number>) => void;
  setCustomRulesDialogOpen: (value: SetStateAction<boolean>) => void;
  setDiscardDialogOpen: (value: SetStateAction<boolean>) => void;
  setHydrated: (value: SetStateAction<boolean>) => void;
  setJsonViewer: (value: SetStateAction<JsonViewerState | null>) => void;
  setRuleGroupPage: (value: SetStateAction<number>) => void;
  setRulesDialogOpen: (value: SetStateAction<boolean>) => void;
  setRuntimeLogPage: (value: SetStateAction<number>) => void;
  setPresetDetailsForceCustomChoice: (value: SetStateAction<boolean>) => void;
  setPresetStateForceCustomChoice: (value: SetStateAction<boolean>) => void;
  setTitleRulePage: (value: SetStateAction<number>) => void;
}

export function useAppLifecycle({
  activeCustomPresetIndex,
  activeRuleIndex,
  appliedRuntimeConfigSignature,
  baseState,
  config,
  customRulesDialogOpen,
  discardDialogOpen,
  discordSnapshot,
  hydrated,
  jsonViewer,
  reporterSnapshot,
  rulesDialogOpen,
  runtimeLogsLength,
  refreshDiscord,
  refreshReporter,
  runAction,
  startRuntimeSession,
  setActiveCustomPresetIndex,
  setActiveRuleIndex,
  setAppliedRuntimeConfigSignature,
  setBaseState,
  setCapabilities,
  setConfig,
  setCustomPresetPage,
  setCustomRulesDialogOpen,
  setDiscardDialogOpen,
  setHydrated,
  setJsonViewer,
  setRuleGroupPage,
  setRulesDialogOpen,
  setRuntimeLogPage,
  setPresetDetailsForceCustomChoice,
  setPresetStateForceCustomChoice,
  setTitleRulePage,
}: UseAppLifecycleArgs) {
  const runtimeAutostartAttemptedRef = useRef(false);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const caps = await getClientCapabilities();
      if (!cancelled && caps.success && caps.data) setCapabilities(caps.data);
      const state = await loadAppState();
      if (cancelled) return;
      const resolvedConfig = normalizeClientConfig(state.config);
      const historyTitleLimit = clampHistoryLimit(resolvedConfig.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
      const payload = {
        config: resolvedConfig,
        appHistory: normalizeAppHistory(state.appHistory, historyTitleLimit),
        playSourceHistory: normalizePlaySourceHistory(state.playSourceHistory),
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
    setRuntimeLogPage((current) => clampPage(current, runtimeLogsLength, RUNTIME_LOG_PAGE_SIZE));
  }, [runtimeLogsLength]);

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
    const historyTitleLimit = clampHistoryLimit(config.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
    const nextAppHistory = shouldCaptureHistoryActivity(reporterSnapshot.currentActivity)
      ? mergeAppHistory(baseState.appHistory, reporterSnapshot.currentActivity, historyTitleLimit)
      : baseState.appHistory;
    const nextPlaySourceHistory = mergePlaySourceHistory(baseState.playSourceHistory, reporterSnapshot.currentActivity);
    if (sameJsonValue(nextAppHistory, baseState.appHistory) && sameJsonValue(nextPlaySourceHistory, baseState.playSourceHistory)) return;
    const payload = { ...baseState, appHistory: nextAppHistory, playSourceHistory: nextPlaySourceHistory };
    void saveAppState(payload)
      .then(() => setBaseState(payload))
      .catch(() => {});
  }, [
    hydrated,
    config.captureReportedAppsEnabled,
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
}
