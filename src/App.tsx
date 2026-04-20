import { startTransition, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";

import "./App.css";
import "./styles/layout.css";
import "./styles/forms-rules.css";
import "./styles/runtime.css";
import "./styles/about.css";
import "./styles/overlays.css";
import "./styles/responsive.css";
import {
  ACTIVE_RADIO_CARD_CLASS,
  BADGE_CLASS,
  BUTTON_CLASS,
  CARD_CLASS,
  DANGER_BUTTON_CLASS,
  FIELD_CLASS,
  FIELD_SPAN_CLASS,
  GITHUB_URL,
  GOOD_BADGE_CLASS,
  INPUT_CLASS,
  PANEL_CLASS,
  PANEL_HEAD_CLASS,
  PRIMARY_BUTTON_CLASS,
  RADIO_CARD_CLASS,
  RUNTIME_LOG_PAGE_SIZE,
  SELECT_CLASS,
  STAT_CARD_CLASS,
  SUBRULE_CARD_CLASS,
  TEXTAREA_CLASS,
  TOGGLE_TILE_CLASS,
} from "./app/appConstants";
import {
  normalizePositiveNumberInput,
  clearDiscordCustomAssetReferences,
  replaceDiscordCustomAssets,
  summarizeDiscordCustomPreset,
} from "./app/appConfig";
import { createConfigEditorActions } from "./app/createConfigEditorActions";
import { createOverlayProps } from "./app/createOverlayProps";
import { uniqueHistoryValues } from "./app/appHistory";
import {
  activityMeta,
  activityText,
  alertClass,
  captureModeText,
  clampPage,
  discordActivityTypeText,
  discordReportModeText,
  formatDate,
  logEntryClass,
} from "./app/appFormatting";
import { createAppDerivedState } from "./app/createAppDerivedState";
import { createAppPaginationState } from "./app/createAppPaginationState";
import { createSettingsViewProps } from "./app/createSettingsViewProps";
import { createRuntimeViewProps } from "./app/createRuntimeViewProps";
import { useAppLifecycle } from "./app/useAppLifecycle";
import { useProfileActions } from "./app/useProfileActions";
import { useRuntimeActions } from "./app/useRuntimeActions";
import appIcon from "./assets/app-icon-base.png";
import { AppOverlays } from "./components/app/AppOverlays";
import { AboutPage } from "./components/pages/AboutPage";
import { ResourcesPage } from "./components/pages/ResourcesPage";
import { RuntimePage } from "./components/pages/RuntimePage";
import { SECTION_COPY, SECTION_ORDER, type ViewSection } from "./components/pages/pageSections";
import { SettingsPage } from "./components/pages/SettingsPage";
import { AppShellLayout } from "./components/layout/AppShellLayout";
import { RulesDialogContent } from "./components/rules/RulesDialogContent";
import {
  deleteDiscordCustomAsset,
  hideToTray,
  importDiscordCustomAsset,
  loadAppState,
} from "./lib/api";
import { normalizeClientConfig } from "./lib/rules";
import { type NoticeTone } from "./store/appUiStore";
import { useAppUiState } from "./store/useAppUiState";
import type { DiscordDebugPayload, ReporterLogEntry } from "./types";

const TRAY_QUICK_SWITCH_EVENT = "tray-quick-switch-applied";

interface TrayQuickSwitchNotice {
  tone: NoticeTone;
  title: string;
  detail: string;
  reloadState: boolean;
  refreshRuntime?: boolean;
}

async function fileToBase64(file: File) {
  const buffer = new Uint8Array(await file.arrayBuffer());
  let binary = "";
  const chunkSize = 0x8000;
  for (let index = 0; index < buffer.length; index += chunkSize) {
    binary += String.fromCharCode(...buffer.subarray(index, index + chunkSize));
  }
  return window.btoa(binary);
}

function inferAssetName(fileName: string) {
  const trimmed = fileName.trim();
  return trimmed.replace(/\.[^.]+$/, "").trim() || "Custom asset";
}

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
  } = useAppUiState();
  const {
    update,
    updateDiscordModeSettings,
    updateRuntimeAutostart,
    patchRuleAt,
    patchTitleRuleAt,
    patchDiscordCustomPresetAt,
    patchDiscordButtonAt,
    patchRuleDiscordButtonAt,
  } = createConfigEditorActions(setConfig);

  function notify(tone: NoticeTone, title: string, detail: string) {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    addNotice({ id, tone, title, detail });
    window.setTimeout(() => {
      startTransition(() => removeNotice(id));
    }, 4200);
  }

  const {
    dirty,
    activeRule,
    activeCopy,
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
    runtimeBlockReason,
    reportPreviewText,
    sidebarStatus,
  } = createAppDerivedState({
    config,
    baseState,
    activeSection,
    activeRuleIndex,
    reporterSnapshot,
    discordSnapshot,
    appliedRuntimeConfigSignature,
  });
  const {
    refreshReporter,
    refreshDiscord,
    refreshPlatformSelfTest,
    requestPlatformAccessibilityPermission,
    startRuntimeSession,
    stopRuntimeSession,
    restartRuntimeSession,
  } = useRuntimeActions({
    config,
    reporterSnapshot,
    discordSnapshot,
    runtimeReady,
    runtimeBlockReason,
    notify,
    setReporterSnapshot,
    setDiscordSnapshot,
    setPlatformSelfTest,
    setAppliedRuntimeConfigSignature,
  });
  const {
    persistPayload,
    saveProfile,
    discardDraftChanges,
    saveCurrentCustomSettingsAsPreset,
    applyDiscordCustomPreset,
  } = useProfileActions({
    capabilities,
    baseState,
    config,
    notify,
    setActiveSection,
    setBaseState,
    setConfig,
    setCustomPresetPage,
    setActiveCustomPresetIndex,
    setCustomRulesDialogOpen,
  });
  const appSuggestions = useMemo(
    () => uniqueHistoryValues(baseState.appHistory.map((entry) => entry.processName)),
    [baseState.appHistory],
  );
  const playSourceSuggestions = useMemo(
    () => uniqueHistoryValues(baseState.playSourceHistory.map((entry) => entry.source)),
    [baseState.playSourceHistory],
  );
  const {
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
  } = createAppPaginationState({
    config,
    reporterSnapshot,
    activeRule,
    runtimeLogPage,
    ruleGroupPage,
    titleRulePage,
    customPresetPage,
    activeCustomPresetIndex,
  });
  useAppLifecycle({
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
    runtimeLogsLength: runtimeLogs.length,
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
  });
  const discordDebugPayload = useMemo<DiscordDebugPayload | null>(
    () => discordSnapshot.debugPayload ?? null,
    [discordSnapshot.debugPayload],
  );
  const jsonViewerJson = useMemo(() => (jsonViewer?.value ? JSON.stringify(jsonViewer.value, null, 2) : ""), [jsonViewer]);
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

  function syncDiscordCustomAssets(assets: Parameters<typeof replaceDiscordCustomAssets>[1]) {
    setBaseState((current) => ({
      ...current,
      config: replaceDiscordCustomAssets(current.config, assets),
    }));
    setConfig((current) => replaceDiscordCustomAssets(current, assets));
  }

  function syncDiscordCustomAssetsAfterDelete(
    assetId: string,
    assets: Parameters<typeof replaceDiscordCustomAssets>[1],
  ) {
    setBaseState((current) => ({
      ...current,
      config: replaceDiscordCustomAssets(clearDiscordCustomAssetReferences(current.config, assetId), assets),
    }));
    setConfig((current) => replaceDiscordCustomAssets(clearDiscordCustomAssetReferences(current, assetId), assets));
  }

  async function handleImportDiscordCustomAssetFiles(files: File[]) {
    let latestAssets = null;
    for (const file of files) {
      const contentType = file.type.trim().toLowerCase();
      if (contentType !== "image/png" && contentType !== "image/jpeg" && contentType !== "image/jpg") {
        throw new Error(`"${file.name}" is not a PNG or JPEG image.`);
      }
      latestAssets = await importDiscordCustomAsset({
        name: inferAssetName(file.name),
        fileName: file.name,
        contentType: contentType === "image/jpg" ? "image/jpeg" : contentType,
        base64Data: await fileToBase64(file),
      });
      syncDiscordCustomAssets(latestAssets);
    }

    if (latestAssets) {
      notify(
        "success",
        files.length === 1 ? "Image added to gallery" : "Images added to gallery",
        files.length === 1
          ? "The image is now ready in the Gallery and Custom artwork pickers."
          : `${files.length} images are now ready in the Gallery and Custom artwork pickers.`,
      );
    }
  }

  async function handleDeleteDiscordCustomAsset(assetId: string) {
    const assets = await deleteDiscordCustomAsset(assetId);
    syncDiscordCustomAssetsAfterDelete(assetId, assets);
    notify("info", "Image removed", "The local image was removed from the Gallery.");
  }

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen<TrayQuickSwitchNotice>(TRAY_QUICK_SWITCH_EVENT, (event) => {
      void (async () => {
        if (disposed) return;

        if (event.payload.reloadState) {
          const state = await loadAppState();
          if (disposed) return;
          const resolvedConfig = normalizeClientConfig(state.config);
          setBaseState({ ...state, config: resolvedConfig });
          setConfig(resolvedConfig);
          setAppliedRuntimeConfigSignature(null);
          await refreshReporter();
          if (disposed) return;
          await refreshDiscord();
          if (disposed) return;
        } else if (event.payload.refreshRuntime) {
          await refreshReporter();
          if (disposed) return;
          await refreshDiscord();
          if (disposed) return;
        }

        notify(event.payload.tone, event.payload.title, event.payload.detail);
      })();
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [
    refreshDiscord,
    refreshReporter,
    setAppliedRuntimeConfigSignature,
    setBaseState,
    setConfig,
  ]);

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
    onImportDiscordCustomAssetFiles: (files) =>
      void runAction("importDiscordCustomAsset", async () => handleImportDiscordCustomAssetFiles(files)),
    onDeleteDiscordCustomAsset: (assetId) =>
      void runAction("deleteDiscordCustomAsset", async () => handleDeleteDiscordCustomAsset(assetId)),
    setConfig,
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

  const resourcesView = (
      <ResourcesPage
        assetLibraryProps={{
          assets: config.discordCustomAssets,
          panelHeadClass: PANEL_HEAD_CLASS,
          buttonClass: BUTTON_CLASS,
          dangerButtonClass: DANGER_BUTTON_CLASS,
          onImportFiles: (files) =>
            void runAction("importDiscordCustomAsset", async () => handleImportDiscordCustomAssetFiles(files)),
          onDeleteAsset: (assetId) =>
            void runAction("deleteDiscordCustomAsset", async () => handleDeleteDiscordCustomAsset(assetId)),
        }}
    />
  );

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

  const activeSectionView =
    activeSection === "settings"
      ? settingsView
      : activeSection === "resources"
        ? resourcesView
        : activeSection === "about"
          ? aboutView
          : runtimeView;

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
