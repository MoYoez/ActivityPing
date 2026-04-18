import { startTransition, useEffect, useMemo, useRef, useState } from "react";

import "./App.css";
import appIcon from "./assets/app-icon-base.png";
import { ListEditor, SuggestionInput } from "./components/ListEditor";
import {
  defaultClientConfig,
  getClientCapabilities,
  getDiscordPresenceSnapshot,
  getRealtimeReporterSnapshot,
  hideToTray,
  isAutostartEnabled,
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
import { exportRulesJson, normalizeClientConfig, parseRulesJson, summarizeRuleGroup } from "./lib/rules";
import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordActivityType,
  DiscordDebugPayload,
  DiscordPresenceSnapshot,
  DiscordReportMode,
  PlatformProbeResult,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "./types";

type NoticeTone = "info" | "success" | "warn" | "error";
type ViewSection = "settings" | "runtime";

const CARD_CLASS = "card border border-base-300 bg-base-100 shadow-sm";
const PANEL_CLASS = `${CARD_CLASS} space-y-4 p-4`;
const PANEL_HEAD_CLASS = "flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between";
const FIELD_CLASS = "flex min-w-0 flex-col gap-2";
const FIELD_SPAN_CLASS = `${FIELD_CLASS} field-span-2`;
const INPUT_CLASS = "input input-bordered w-full";
const TEXTAREA_CLASS = "textarea textarea-bordered w-full";
const SELECT_CLASS = "select select-bordered w-full";
const BUTTON_CLASS = "btn btn-outline btn-sm";
const PRIMARY_BUTTON_CLASS = "btn btn-primary btn-sm";
const DANGER_BUTTON_CLASS = "btn btn-error btn-outline btn-sm";
const BADGE_CLASS = "badge badge-soft";
const GOOD_BADGE_CLASS = "badge badge-success badge-soft";
const STAT_CARD_CLASS = "min-h-[92px] rounded-box border border-base-300 bg-base-200/70 p-4";
const TOGGLE_TILE_CLASS = "flex min-h-[88px] items-center justify-between gap-3 rounded-box border border-base-300 bg-base-200/70 p-4 text-left";
const RADIO_CARD_CLASS = "flex items-start gap-3 rounded-box border border-base-300 bg-base-100 p-4 text-left";
const ACTIVE_RADIO_CARD_CLASS = "flex items-start gap-3 rounded-box border border-primary bg-primary/10 p-4 text-left";
const SUBRULE_CARD_CLASS = "card border border-base-300 bg-base-100 shadow-sm";
const MAX_RUNTIME_LOGS = 20;
const RULE_GROUP_PAGE_SIZE = 6;
const TITLE_RULE_PAGE_SIZE = 3;
const DISCORD_TEMPLATE_TOKENS = [
  "{activity}",
  "{context}",
  "{app}",
  "{title}",
  "{rule}",
  "{media}",
  "{song}",
  "{artist}",
  "{album}",
  "{source}",
] as const;
const DISCORD_TEMPLATE_TOKEN_HINTS: Record<(typeof DISCORD_TEMPLATE_TOKENS)[number], string> = {
  "{activity}": "The main activity text after the current mode and matching rules are applied.",
  "{context}": "The secondary line for the current activity, usually the app name or artist.",
  "{app}": "The captured app or process name.",
  "{title}": "The current window title.",
  "{rule}": "The text produced by the matched rule, when a rule hit exists.",
  "{media}": "A combined media summary, such as song, artist, and album when media is active.",
  "{song}": "The current media title or song name.",
  "{artist}": "The current media artist.",
  "{album}": "The current media album.",
  "{source}": "The active media source app id when source reporting is enabled.",
};
const DISCORD_REPORT_MODE_OPTIONS: Array<{
  mode: DiscordReportMode;
  title: string;
  description: string;
  details: string;
  state: string;
}> = [
  {
    mode: "mixed",
    title: "Smart",
    description: "Keep the current title on the first line, the app name on the second line, and reserve artwork and timer space for active media.",
    details: "Window title or rule text",
    state: "App name",
  },
  {
    mode: "music",
    title: "Music",
    description: "Always format the activity around now-playing metadata.",
    details: "Song / media title",
    state: "Artist",
  },
  {
    mode: "app",
    title: "App",
    description: "Keep the activity centered on the current app, title, or matched rule text.",
    details: "Rule text or window title",
    state: "App name",
  },
  {
    mode: "custom",
    title: "Custom",
    description: "Edit the Discord lines directly and choose the Discord activity label.",
    details: "Custom template",
    state: "Custom template",
  },
];
const DISCORD_ACTIVITY_TYPE_OPTIONS: Array<{
  value: DiscordActivityType;
  label: string;
  helper: string;
}> = [
  { value: "playing", label: "Playing", helper: "Default game-like activity label." },
  { value: "listening", label: "Listening", helper: "Discord shows a listening-style label." },
  { value: "watching", label: "Watching", helper: "Discord shows a watching-style label." },
  { value: "competing", label: "Competing", helper: "Discord shows a competing-style label." },
];

function alertClass(tone: NoticeTone) {
  switch (tone) {
    case "success":
      return "alert alert-success";
    case "warn":
      return "alert alert-warning";
    case "error":
      return "alert alert-error";
    default:
      return "alert alert-info";
  }
}

function logEntryClass(level: string) {
  switch (level) {
    case "success":
      return "log-entry card card-compact border-success bg-base-100 shadow-sm";
    case "warn":
      return "log-entry card card-compact border-warning bg-base-100 shadow-sm";
    case "error":
      return "log-entry card card-compact border-error bg-base-100 shadow-sm";
    default:
      return "log-entry card card-compact border-info bg-base-100 shadow-sm";
  }
}

function limitReporterSnapshotLogs(snapshot: RealtimeReporterSnapshot) {
  if (snapshot.logs.length <= MAX_RUNTIME_LOGS) {
    return snapshot;
  }
  return { ...snapshot, logs: snapshot.logs.slice(0, MAX_RUNTIME_LOGS) };
}

interface Notice {
  id: number;
  tone: NoticeTone;
  title: string;
  detail: string;
}

const DEFAULT_CAPABILITIES: ClientCapabilities = {
  realtimeReporter: true,
  tray: true,
  platformSelfTest: true,
  discordPresence: true,
  autostart: true,
};

const EMPTY_REPORTER: RealtimeReporterSnapshot = {
  running: false,
  logs: [],
  currentActivity: null,
  lastHeartbeatAt: null,
  lastError: null,
};

const EMPTY_DISCORD: DiscordPresenceSnapshot = {
  running: false,
  connected: false,
  lastSyncAt: null,
  lastError: null,
  currentSummary: null,
  debugPayload: null,
};

function probeBadgeClass(probe: PlatformProbeResult) {
  return probe.success ? GOOD_BADGE_CLASS : "badge badge-error badge-soft";
}

const SECTION_COPY: Record<ViewSection, { kicker: string; title: string; description: string }> = {
  settings: {
    kicker: "Settings",
    title: "RPC and local rules",
    description: "Configure Discord RPC first, then tune monitor behavior and local rule clauses in one place.",
  },
  runtime: {
    kicker: "Runtime",
    title: "Live monitor",
    description: "Watch captured activity, current RPC output and the recent runtime log. Requires a saved RPC profile first.",
  },
};

const SECTION_ORDER: ViewSection[] = ["runtime", "settings"];

function clampRuleIndex(index: number, total: number) {
  if (total <= 0) {
    return -1;
  }
  return Math.min(Math.max(index, 0), total - 1);
}

function pageCount(total: number, pageSize: number) {
  return Math.max(1, Math.ceil(total / pageSize));
}

function clampPage(page: number, total: number, pageSize: number) {
  return Math.min(Math.max(page, 0), pageCount(total, pageSize) - 1);
}

function pageForIndex(index: number, pageSize: number) {
  return index < 0 ? 0 : Math.floor(index / pageSize);
}

function configSignature(value: ClientConfig) {
  return JSON.stringify(normalizeClientConfig(value));
}

function formatDate(value?: string | null) {
  if (!value) {
    return "Not available";
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("en-US");
}

function formatDurationClock(valueMs?: number | null) {
  if (typeof valueMs !== "number" || !Number.isFinite(valueMs) || valueMs < 0) {
    return null;
  }

  const totalSeconds = Math.floor(valueMs / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function mediaTimelineText(positionMs?: number | null, durationMs?: number | null) {
  const positionText = formatDurationClock(positionMs);
  const durationText = formatDurationClock(durationMs);

  if (positionText && durationText) {
    return `${positionText} / ${durationText}`;
  }
  if (durationText) {
    return `Length: ${durationText}`;
  }
  if (positionText) {
    return `Position: ${positionText}`;
  }
  return null;
}

function activityText(snapshot: RealtimeReporterSnapshot) {
  const activity = snapshot.currentActivity;
  if (!activity) {
    return "No activity captured yet.";
  }
  if (activity.statusText?.trim()) {
    return activity.statusText;
  }
  if (activity.processName && activity.processTitle?.trim()) {
    return `${activity.processName} / ${activity.processTitle}`;
  }
  if (activity.processName?.trim()) {
    return activity.processName;
  }
  if (activity.mediaSummary?.trim()) {
    return activity.mediaSummary;
  }
  return "No activity captured yet.";
}

function activityMeta(snapshot: RealtimeReporterSnapshot) {
  const activity = snapshot.currentActivity;
  if (!activity) {
    return "Waiting for the next local capture sample.";
  }
  const parts = [];
  if (activity.mediaSummary?.trim()) {
    parts.push(activity.mediaSummary);
  }
  if (activity.playSource?.trim()) {
    parts.push(`Source: ${activity.playSource}`);
  }
  const timelineText = mediaTimelineText(activity.mediaPositionMs, activity.mediaDurationMs);
  if (timelineText) {
    parts.push(timelineText);
  }
  return parts.length > 0 ? parts.join(" · ") : "No media metadata attached.";
}

function captureModeText(config: ClientConfig) {
  const parts = [];
  if (config.reportForegroundApp) parts.push("App");
  if (config.reportWindowTitle) parts.push("Title");
  if (config.reportMedia) parts.push("Media");
  if (config.reportPlaySource) parts.push("Source");
  return parts.length > 0 ? parts.join(" + ") : "Capture disabled";
}

function discordReportModeText(config: ClientConfig) {
  switch (config.discordReportMode) {
    case "music":
      return "Music only";
    case "app":
      return "App only";
    case "custom":
      return "Custom";
    default:
      return "Smart";
  }
}

function discordActivityTypeText(value: DiscordActivityType) {
  switch (value) {
    case "listening":
      return "Listening";
    case "watching":
      return "Watching";
    case "competing":
      return "Competing";
    default:
      return "Playing";
  }
}

function mergeHistoryList(values: string[], value: string, lowercase: boolean) {
  const trimmed = value.trim();
  if (!trimmed) return values;
  const nextValue = lowercase ? trimmed.toLowerCase() : trimmed;
  const key = nextValue.toLowerCase();
  if (values.some((item) => item.trim().toLowerCase() === key)) {
    return values;
  }
  return [nextValue, ...values].slice(0, 240);
}

function appendUniqueListValue(values: string[], value: string, lowercase: boolean) {
  const trimmed = value.trim();
  if (!trimmed) {
    return values;
  }
  const nextValue = lowercase ? trimmed.toLowerCase() : trimmed;
  const key = nextValue.toLowerCase();
  if (values.some((item) => item.trim().toLowerCase() === key)) {
    return values;
  }
  return [...values, nextValue];
}

function moveItem<T>(items: T[], from: number, to: number) {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) {
    return items;
  }
  const next = [...items];
  const [picked] = next.splice(from, 1);
  next.splice(to, 0, picked);
  return next;
}

function validateRuleRegex(groups: AppMessageRuleGroup[]) {
  for (const group of groups) {
    for (const titleRule of group.titleRules) {
      if (titleRule.mode !== "regex") continue;
      try {
        new RegExp(titleRule.pattern, "i");
      } catch (error) {
        return error instanceof Error ? error.message : "Invalid regular expression.";
      }
    }
  }
  return null;
}

function buildPayload(baseState: AppStatePayload, config: ClientConfig): AppStatePayload {
  return { ...baseState, config: normalizeClientConfig(config), locale: "en-US" };
}

function App() {
  const [capabilities, setCapabilities] = useState(DEFAULT_CAPABILITIES);
  const [baseState, setBaseState] = useState<AppStatePayload>({
    config: defaultClientConfig(),
    appHistory: [],
    playSourceHistory: [],
    locale: "en-US",
  });
  const [config, setConfig] = useState(defaultClientConfig());
  const [reporterSnapshot, setReporterSnapshot] = useState(EMPTY_REPORTER);
  const [discordSnapshot, setDiscordSnapshot] = useState(EMPTY_DISCORD);
  const [platformSelfTest, setPlatformSelfTest] = useState<PlatformSelfTestResult | null>(null);
  const [notices, setNotices] = useState<Notice[]>([]);
  const [hydrated, setHydrated] = useState(false);
  const [busy, setBusy] = useState<Record<string, boolean>>({});
  const [activeSection, setActiveSection] = useState<ViewSection>("settings");
  const [activeRuleIndex, setActiveRuleIndex] = useState(0);
  const [rulesImportOpen, setRulesImportOpen] = useState(false);
  const [rulesImportValue, setRulesImportValue] = useState("");
  const [blacklistInput, setBlacklistInput] = useState("");
  const [whitelistInput, setWhitelistInput] = useState("");
  const [nameOnlyInput, setNameOnlyInput] = useState("");
  const [mediaSourceInput, setMediaSourceInput] = useState("");
  const [activeDiscordFormatField, setActiveDiscordFormatField] = useState<"details" | "state">("details");
  const [rulesDialogOpen, setRulesDialogOpen] = useState(false);
  const [discardDialogOpen, setDiscardDialogOpen] = useState(false);
  const [appliedRuntimeConfigSignature, setAppliedRuntimeConfigSignature] = useState<string | null>(null);
  const [ruleGroupPage, setRuleGroupPage] = useState(0);
  const [titleRulePage, setTitleRulePage] = useState(0);
  const [discordDebugOpen, setDiscordDebugOpen] = useState(false);
  const discordDetailsInputRef = useRef<HTMLInputElement | null>(null);
  const discordStateInputRef = useRef<HTMLInputElement | null>(null);

  function notify(tone: NoticeTone, title: string, detail: string) {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    setNotices((items) => [...items, { id, tone, title, detail }]);
    window.setTimeout(() => {
      startTransition(() => setNotices((items) => items.filter((item) => item.id !== id)));
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
    const regexError = validateRuleRegex(normalized.appMessageRules);
    if (regexError) {
      notify("error", "Rules not saved", regexError);
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

  function insertDiscordToken(token: string) {
    const targetField = activeDiscordFormatField;
    const input = targetField === "state" ? discordStateInputRef.current : discordDetailsInputRef.current;
    const currentValue = targetField === "state" ? config.discordStateFormat : config.discordDetailsFormat;
    const selectionStart = input?.selectionStart ?? currentValue.length;
    const selectionEnd = input?.selectionEnd ?? currentValue.length;
    const nextValue = `${currentValue.slice(0, selectionStart)}${token}${currentValue.slice(selectionEnd)}`;
    const nextCursor = selectionStart + token.length;

    if (targetField === "state") {
      update("discordStateFormat", nextValue);
    } else {
      update("discordDetailsFormat", nextValue);
    }

    window.requestAnimationFrame(() => {
      const targetInput = targetField === "state" ? discordStateInputRef.current : discordDetailsInputRef.current;
      targetInput?.focus();
      targetInput?.setSelectionRange(nextCursor, nextCursor);
    });
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

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const caps = await getClientCapabilities();
      if (!cancelled && caps.success && caps.data) setCapabilities(caps.data);
      const state = await loadAppState();
      if (cancelled) return;
      let resolvedConfig = normalizeClientConfig(state.config);
      try {
        resolvedConfig = { ...resolvedConfig, launchOnStartup: await isAutostartEnabled() };
      } catch {}
      const payload = {
        config: resolvedConfig,
        appHistory: Array.isArray(state.appHistory) ? state.appHistory : [],
        playSourceHistory: Array.isArray(state.playSourceHistory) ? state.playSourceHistory : [],
        locale: "en-US",
      };
      setBaseState(payload);
      setConfig(payload.config);
      setHydrated(true);
      await refreshReporter();
      await refreshDiscord();
      if (!cancelled && caps.success && caps.data?.platformSelfTest) {
        const selfTest = await runPlatformSelfTest();
        if (!cancelled && selfTest.success && selfTest.data) {
          setPlatformSelfTest(selfTest.data);
        }
      }
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
    if (!rulesDialogOpen && !discardDialogOpen && !discordDebugOpen) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (discordDebugOpen) {
          setDiscordDebugOpen(false);
          return;
        }
        if (discardDialogOpen) {
          setDiscardDialogOpen(false);
          return;
        }
        setRulesDialogOpen(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [rulesDialogOpen, discardDialogOpen, discordDebugOpen]);

  useEffect(() => {
    if (!hydrated || !config.captureReportedAppsEnabled) return;
    const processName = reporterSnapshot.currentActivity?.processName?.trim() ?? "";
    const playSource = reporterSnapshot.currentActivity?.playSource?.trim() ?? "";
    const nextAppHistory = processName ? mergeHistoryList(baseState.appHistory, processName, false) : baseState.appHistory;
    const nextPlaySourceHistory = playSource ? mergeHistoryList(baseState.playSourceHistory, playSource, true) : baseState.playSourceHistory;
    if (nextAppHistory === baseState.appHistory && nextPlaySourceHistory === baseState.playSourceHistory) return;
    const payload = { ...baseState, appHistory: nextAppHistory, playSourceHistory: nextPlaySourceHistory };
    void saveAppState(payload).then(() => setBaseState(payload)).catch(() => {});
  }, [
    hydrated,
    config.captureReportedAppsEnabled,
    reporterSnapshot.currentActivity?.processName,
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
  const runtimeAutostartEnabled = config.runtimeAutostartEnabled;
  const runtimeReady = baseState.config.discordApplicationId.trim().length > 0;
  const runtimeRunning = reporterSnapshot.running || discordSnapshot.running;
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
  const appSuggestions = useMemo(() => baseState.appHistory, [baseState.appHistory]);
  const playSourceSuggestions = useMemo(() => baseState.playSourceHistory, [baseState.playSourceHistory]);
  const runtimeLogs = useMemo(() => reporterSnapshot.logs.slice(0, MAX_RUNTIME_LOGS), [reporterSnapshot.logs]);
  const discordDebugPayload = useMemo<DiscordDebugPayload | null>(
    () => discordSnapshot.debugPayload ?? null,
    [discordSnapshot.debugPayload],
  );
  const discordDebugJson = useMemo(
    () => (discordDebugPayload ? JSON.stringify(discordDebugPayload, null, 2) : ""),
    [discordDebugPayload],
  );
  const ruleGroupTotalPages = pageCount(config.appMessageRules.length, RULE_GROUP_PAGE_SIZE);
  const safeRuleGroupPage = clampPage(ruleGroupPage, config.appMessageRules.length, RULE_GROUP_PAGE_SIZE);
  const ruleGroupPageStart = safeRuleGroupPage * RULE_GROUP_PAGE_SIZE;
  const pagedRuleGroups = config.appMessageRules.slice(ruleGroupPageStart, ruleGroupPageStart + RULE_GROUP_PAGE_SIZE);
  const activeTitleRuleCount = activeRule?.titleRules.length ?? 0;
  const titleRuleTotalPages = pageCount(activeTitleRuleCount, TITLE_RULE_PAGE_SIZE);
  const safeTitleRulePage = clampPage(titleRulePage, activeTitleRuleCount, TITLE_RULE_PAGE_SIZE);
  const titleRulePageStart = safeTitleRulePage * TITLE_RULE_PAGE_SIZE;
  const pagedTitleRules = activeRule?.titleRules.slice(titleRulePageStart, titleRulePageStart + TITLE_RULE_PAGE_SIZE) ?? [];

  function update<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
    setConfig((current) => ({ ...current, [key]: value }));
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

  const generalView = (
    <>
      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Behavior</p>
            <h3>Startup and timing</h3>
          </div>
        </div>
        <div className="toggle-grid">
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Auto-start runtime</strong>
              <span>Start local capture and Discord RPC together when the app launches.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={runtimeAutostartEnabled} onChange={(e) => updateRuntimeAutostart(e.currentTarget.checked)} />
          </label>
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Launch with system</strong>
              <span>Register the app in the OS startup list.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.launchOnStartup} onChange={(e) => update("launchOnStartup", e.currentTarget.checked)} />
          </label>
        </div>
        <div className="field-grid compact-fields">
          <label className={FIELD_CLASS}>
            <span>Poll interval (ms)</span>
            <input className={INPUT_CLASS} type="number" min={1000} value={config.pollIntervalMs} onChange={(e) => update("pollIntervalMs", Number(e.currentTarget.value) || 1000)} />
          </label>
          <label className={FIELD_CLASS}>
            <span>Heartbeat interval (ms)</span>
            <input className={INPUT_CLASS} type="number" min={0} value={config.heartbeatIntervalMs} onChange={(e) => update("heartbeatIntervalMs", Number(e.currentTarget.value) || 0)} />
          </label>
        </div>
        <div className="card-actions gap-2">
          {capabilities.tray ? (
            <button
              className={BUTTON_CLASS}
              onClick={() =>
                void hideToTray().catch((error: unknown) =>
                  notify("warn", "Tray hide failed", error instanceof Error ? error.message : "The window could not be hidden."),
                )
              }
            >
              Hide to tray
            </button>
          ) : null}
        </div>
      </section>

      {capabilities.platformSelfTest ? (
        <section className={PANEL_CLASS}>
          <div className={PANEL_HEAD_CLASS}>
            <div>
              <p className="eyebrow">Platform</p>
              <h3>Permissions and self-test</h3>
            </div>
          </div>
          <div className="card-actions gap-2">
            <button
              className={BUTTON_CLASS}
              type="button"
              disabled={busy.platformSelfTest}
              onClick={() => runAction("platformSelfTest", async () => refreshPlatformSelfTest(true))}
            >
              {busy.platformSelfTest ? "Running..." : "Run self-test"}
            </button>
            <button
              className={BUTTON_CLASS}
              type="button"
              disabled={busy.accessibilityPermission}
              onClick={() => runAction("accessibilityPermission", requestPlatformAccessibilityPermission)}
            >
              {busy.accessibilityPermission ? "Requesting..." : "Request Accessibility Permission"}
            </button>
          </div>
          {platformSelfTest ? (
            <>
              <div className="stat-grid">
                {[
                  { label: "Foreground app", probe: platformSelfTest.foreground },
                  { label: "Window title", probe: platformSelfTest.windowTitle },
                  { label: "Media capture", probe: platformSelfTest.media },
                ].map(({ label, probe }) => (
                  <div key={label} className={STAT_CARD_CLASS}>
                    <span>{label}</span>
                    <strong>{probe.summary}</strong>
                    <div className="platform-probe-badge-row">
                      <span className={probeBadgeClass(probe)}>{probe.success ? "OK" : "Needs attention"}</span>
                    </div>
                  </div>
                ))}
              </div>
              <div className="platform-probe-list">
                {[
                  { label: "Foreground app", probe: platformSelfTest.foreground },
                  { label: "Window title", probe: platformSelfTest.windowTitle },
                  { label: "Media capture", probe: platformSelfTest.media },
                ].map(({ label, probe }) => (
                  <div key={label} className="empty-state platform-probe-card">
                    <strong>{label}</strong>
                    <p>{probe.detail}</p>
                    {probe.guidance?.length ? (
                      <ul>
                        {probe.guidance.map((item) => (
                          <li key={`${label}-${item}`}>{item}</li>
                        ))}
                      </ul>
                    ) : null}
                  </div>
                ))}
              </div>
            </>
          ) : (
            <div className="empty-state">
              Run the self-test to check foreground app capture, window titles, and media capture on this machine.
            </div>
          )}
        </section>
      ) : null}

      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Summary</p>
            <h3>Current local mode</h3>
          </div>
        </div>
        <div className="stat-grid">
          <div className={STAT_CARD_CLASS}>
            <span>Capture mode</span>
            <strong>{captureModeText(config)}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Rule groups</span>
            <strong>{config.appMessageRules.length}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Saved apps</span>
            <strong>{baseState.appHistory.length}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Media sources</span>
            <strong>{baseState.playSourceHistory.length}</strong>
          </div>
        </div>
      </section>
    </>
  );

  const rulesView = (
    <>
      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">App rules</p>
            <h3>Message rule groups</h3>
          </div>
          <span className={BADGE_CLASS}>{config.appMessageRules.length} groups</span>
        </div>

        <div className="toggle-grid compact-toggles rules-toggles">
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Show process name on rule hit</strong>
              <span>Append the executable name after a matching rule text. In Smart mode without media, the process name moves to the last line.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.appMessageRulesShowProcessName} onChange={(e) => update("appMessageRulesShowProcessName", e.currentTarget.checked)} />
          </label>
        </div>

        <div className="card-actions gap-2">
          <button
            className={PRIMARY_BUTTON_CLASS}
            type="button"
            onClick={() => {
              const nextIndex = config.appMessageRules.length;
              setConfig((current) => ({
                ...current,
                appMessageRules: [...current.appMessageRules, { processMatch: "", defaultText: "", titleRules: [] }],
              }));
              setActiveRuleIndex(nextIndex);
              setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
              setTitleRulePage(0);
            }}
          >
            Add rule group
          </button>
          <button
            className={BUTTON_CLASS}
            type="button"
            onClick={() =>
              void navigator.clipboard
                .writeText(exportRulesJson(config))
                .then(() => notify("success", "Rules copied", "The local rule JSON was copied to the clipboard."))
                .catch(() => notify("error", "Copy failed", "Clipboard access was not available."))
            }
          >
            Copy rules JSON
          </button>
          <button className={BUTTON_CLASS} type="button" onClick={() => setRulesImportOpen((current) => !current)}>
            {rulesImportOpen ? "Hide import" : "Import rules JSON"}
          </button>
        </div>

        {rulesImportOpen ? (
          <div className="import-panel">
            <label className={FIELD_CLASS}>
              <span>Rules JSON</span>
              <textarea className={TEXTAREA_CLASS} value={rulesImportValue} onChange={(e) => setRulesImportValue(e.currentTarget.value)} placeholder='{"version":2,"rules":{"appMessageRules":[]}}' />
            </label>
            <div className="card-actions gap-2">
              <button
                className={PRIMARY_BUTTON_CLASS}
                type="button"
                onClick={() => {
                  const parsed = parseRulesJson(rulesImportValue);
                  if (!parsed.ok) {
                    notify("error", "Import failed", parsed.error);
                    return;
                  }
                  setConfig((current) => ({
                    ...current,
                    appMessageRules: parsed.data.appMessageRules,
                    appMessageRulesShowProcessName: parsed.data.appMessageRulesShowProcessName,
                    appFilterMode: parsed.data.appFilterMode,
                    appBlacklist: parsed.data.appBlacklist,
                    appWhitelist: parsed.data.appWhitelist,
                    appNameOnlyList: parsed.data.appNameOnlyList,
                    mediaPlaySourceBlocklist: parsed.data.mediaPlaySourceBlocklist,
                  }));
                  setRulesImportOpen(false);
                  setRulesImportValue("");
                  setActiveRuleIndex(0);
                  setRuleGroupPage(0);
                  setTitleRulePage(0);
                  notify("success", "Rules imported", "The rule JSON was written into the current form.");
                }}
              >
                Apply imported rules
              </button>
            </div>
          </div>
        ) : null}

        <div className="rules-shell">
          <div className="rule-list-panel rounded-box border border-base-300 bg-base-200/45 p-4">
            <div className="list-editor-summary">
              <div className="list-editor-copy">
                <strong className="block font-semibold">Process groups</strong>
                <p>Rules run from top to bottom.</p>
              </div>
            </div>
            {config.appMessageRules.length === 0 ? (
              <div className="empty-state compact-empty">No app rule groups yet.</div>
            ) : (
              <>
                <div className="grid gap-2">
                  {pagedRuleGroups.map((rule, offset) => {
                    const index = ruleGroupPageStart + offset;
                    return (
                      <button
                        key={`${rule.processMatch || "rule"}-${index}`}
                        className={`btn btn-ghost h-auto min-h-16 w-full flex-col items-start justify-start gap-1 text-left normal-case ${activeRuleIndex === index ? "btn-active" : ""}`}
                        type="button"
                        onClick={() => {
                          setActiveRuleIndex(index);
                          setTitleRulePage(0);
                        }}
                      >
                        <strong className="block break-words">{rule.processMatch || `Rule group ${index + 1}`}</strong>
                        <span className="mt-1 block text-sm text-base-content/70">{summarizeRuleGroup(rule)}</span>
                      </button>
                    );
                  })}
                </div>
                {ruleGroupTotalPages > 1 ? (
                  <div className="pagination-row">
                    <span className="pagination-copy">
                      {ruleGroupPageStart + 1}-{Math.min(ruleGroupPageStart + RULE_GROUP_PAGE_SIZE, config.appMessageRules.length)} of {config.appMessageRules.length}
                    </span>
                    <div className="join">
                      <button className="btn btn-outline btn-xs join-item" type="button" disabled={safeRuleGroupPage <= 0} onClick={() => setRuleGroupPage((current) => clampPage(current - 1, config.appMessageRules.length, RULE_GROUP_PAGE_SIZE))}>
                        Prev
                      </button>
                      <span className="btn btn-ghost btn-xs join-item no-animation">Page {safeRuleGroupPage + 1} / {ruleGroupTotalPages}</span>
                      <button className="btn btn-outline btn-xs join-item" type="button" disabled={safeRuleGroupPage >= ruleGroupTotalPages - 1} onClick={() => setRuleGroupPage((current) => clampPage(current + 1, config.appMessageRules.length, RULE_GROUP_PAGE_SIZE))}>
                        Next
                      </button>
                    </div>
                  </div>
                ) : null}
              </>
            )}
          </div>

          <div className={`${CARD_CLASS} space-y-4 p-4 rules-editor`}>
            {activeRule ? (
              <>
                <div className={PANEL_HEAD_CLASS}>
                  <div>
                    <strong className="block font-semibold">Group editor</strong>
                    <p className="mt-1 text-sm text-base-content/70">Match the process first, then the title subrules. Use {"{process}"} and {"{title}"} in text.</p>
                  </div>
                  <div className="card-actions gap-2">
                    <button
                      className={BUTTON_CLASS}
                      type="button"
                      disabled={activeRuleIndex <= 0}
                      onClick={() => {
                        const nextIndex = activeRuleIndex - 1;
                        setConfig((current) => ({
                          ...current,
                          appMessageRules: moveItem(current.appMessageRules, activeRuleIndex, nextIndex),
                        }));
                        setActiveRuleIndex(nextIndex);
                        setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
                      }}
                    >
                      Move up
                    </button>
                    <button
                      className={BUTTON_CLASS}
                      type="button"
                      disabled={activeRuleIndex >= config.appMessageRules.length - 1}
                      onClick={() => {
                        const nextIndex = activeRuleIndex + 1;
                        setConfig((current) => ({
                          ...current,
                          appMessageRules: moveItem(current.appMessageRules, activeRuleIndex, nextIndex),
                        }));
                        setActiveRuleIndex(nextIndex);
                        setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
                      }}
                    >
                      Move down
                    </button>
                    <button
                      className={DANGER_BUTTON_CLASS}
                      type="button"
                      onClick={() => {
                        const nextIndex = clampRuleIndex(activeRuleIndex, config.appMessageRules.length - 1);
                        setConfig((current) => ({
                          ...current,
                          appMessageRules: current.appMessageRules.filter((_, index) => index !== activeRuleIndex),
                        }));
                        setActiveRuleIndex(nextIndex);
                        setRuleGroupPage(pageForIndex(nextIndex, RULE_GROUP_PAGE_SIZE));
                        setTitleRulePage(0);
                      }}
                    >
                      Delete group
                    </button>
                  </div>
                </div>

                <div className="field-grid">
                  <label className={FIELD_CLASS}>
                    <span>Process match</span>
                    <SuggestionInput
                      value={activeRule.processMatch}
                      onChange={(value) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, processMatch: value }))}
                      suggestions={appSuggestions}
                      placeholder="code.exe"
                    />
                  </label>
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Default text</span>
                    <textarea
                      className={TEXTAREA_CLASS}
                      value={activeRule.defaultText}
                      onChange={(e) => {
                        const value = e.currentTarget.value;
                        patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, defaultText: value }));
                      }}
                      placeholder="Coding"
                    />
                  </label>
                </div>

                <div className="rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
                  <div className={PANEL_HEAD_CLASS}>
                    <div>
                      <strong className="block font-semibold">Title subrules</strong>
                      <p className="mt-1 text-sm text-base-content/70">Choose plain contains or regex matching for the window title.</p>
                    </div>
                    <button
                      className={BUTTON_CLASS}
                      type="button"
                      onClick={() => {
                        const nextIndex = activeRule.titleRules.length;
                        patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, titleRules: [...rule.titleRules, { mode: "plain", pattern: "", text: "" }] }));
                        setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
                      }}
                    >
                      Add title rule
                    </button>
                  </div>

                  {activeRule.titleRules.length === 0 ? (
                    <div className="empty-state compact-empty">No title subrules yet.</div>
                  ) : (
                    <>
                      <div className="grid gap-2">
                        {pagedTitleRules.map((titleRule, offset) => {
                          const titleRuleIndex = titleRulePageStart + offset;
                          return (
                            <article key={`${activeRuleIndex}-${titleRuleIndex}`} className={`${SUBRULE_CARD_CLASS} p-4`}>
                              <div className={PANEL_HEAD_CLASS}>
                                <strong className="block font-semibold">Title rule {titleRuleIndex + 1} / {activeRule.titleRules.length}</strong>
                                <div className="card-actions gap-2">
                                  <button
                                    className={BUTTON_CLASS}
                                    type="button"
                                    disabled={titleRuleIndex <= 0}
                                    onClick={() => {
                                      const nextIndex = titleRuleIndex - 1;
                                      patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, titleRules: moveItem(rule.titleRules, titleRuleIndex, nextIndex) }));
                                      setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
                                    }}
                                  >
                                    Up
                                  </button>
                                  <button
                                    className={BUTTON_CLASS}
                                    type="button"
                                    disabled={titleRuleIndex >= activeRule.titleRules.length - 1}
                                    onClick={() => {
                                      const nextIndex = titleRuleIndex + 1;
                                      patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, titleRules: moveItem(rule.titleRules, titleRuleIndex, nextIndex) }));
                                      setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
                                    }}
                                  >
                                    Down
                                  </button>
                                  <button
                                    className={DANGER_BUTTON_CLASS}
                                    type="button"
                                    onClick={() => {
                                      const nextIndex = clampRuleIndex(titleRuleIndex, activeRule.titleRules.length - 1);
                                      patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, titleRules: rule.titleRules.filter((_, index) => index !== titleRuleIndex) }));
                                      setTitleRulePage(pageForIndex(nextIndex, TITLE_RULE_PAGE_SIZE));
                                    }}
                                  >
                                    Remove
                                  </button>
                                </div>
                              </div>

                              <div className="field-grid compact-fields">
                                <label className={FIELD_CLASS}>
                                  <span>Mode</span>
                                  <select
                                    className={SELECT_CLASS}
                                    value={titleRule.mode}
                                    onChange={(e) => {
                                      const mode = e.currentTarget.value === "regex" ? "regex" : "plain";
                                      patchTitleRuleAt(activeRuleIndex, titleRuleIndex, (rule) => ({ ...rule, mode }));
                                    }}
                                  >
                                    <option value="plain">Plain contains</option>
                                    <option value="regex">Regex</option>
                                  </select>
                                </label>
                                <label className={FIELD_SPAN_CLASS}>
                                  <span>Pattern</span>
                                  <textarea
                                    className={TEXTAREA_CLASS}
                                    value={titleRule.pattern}
                                    onChange={(e) => {
                                      const value = e.currentTarget.value;
                                      patchTitleRuleAt(activeRuleIndex, titleRuleIndex, (rule) => ({ ...rule, pattern: value }));
                                    }}
                                    placeholder={titleRule.mode === "regex" ? "\\.tsx$" : "Visual Studio Code"}
                                  />
                                </label>
                                <label className={FIELD_SPAN_CLASS}>
                                  <span>Text</span>
                                  <textarea
                                    className={TEXTAREA_CLASS}
                                    value={titleRule.text}
                                    onChange={(e) => {
                                      const value = e.currentTarget.value;
                                      patchTitleRuleAt(activeRuleIndex, titleRuleIndex, (rule) => ({ ...rule, text: value }));
                                    }}
                                    placeholder="Writing frontend: {title}"
                                  />
                                </label>
                              </div>
                            </article>
                          );
                        })}
                      </div>
                      {titleRuleTotalPages > 1 ? (
                        <div className="pagination-row">
                          <span className="pagination-copy">
                            {titleRulePageStart + 1}-{Math.min(titleRulePageStart + TITLE_RULE_PAGE_SIZE, activeTitleRuleCount)} of {activeTitleRuleCount}
                          </span>
                          <div className="join">
                            <button className="btn btn-outline btn-xs join-item" type="button" disabled={safeTitleRulePage <= 0} onClick={() => setTitleRulePage((current) => clampPage(current - 1, activeTitleRuleCount, TITLE_RULE_PAGE_SIZE))}>
                              Prev
                            </button>
                            <span className="btn btn-ghost btn-xs join-item no-animation">Page {safeTitleRulePage + 1} / {titleRuleTotalPages}</span>
                            <button className="btn btn-outline btn-xs join-item" type="button" disabled={safeTitleRulePage >= titleRuleTotalPages - 1} onClick={() => setTitleRulePage((current) => clampPage(current + 1, activeTitleRuleCount, TITLE_RULE_PAGE_SIZE))}>
                              Next
                            </button>
                          </div>
                        </div>
                      ) : null}
                    </>
                  )}
                </div>
              </>
            ) : (
              <div className="empty-state rules-empty">Create a process group to start editing local app message rules.</div>
            )}
          </div>
        </div>
      </section>

      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Filter</p>
            <h3>App filter mode</h3>
          </div>
          <span className={BADGE_CLASS}>{config.appFilterMode}</span>
        </div>

        <div className="radio-grid">
          <label className={config.appFilterMode === "blacklist" ? ACTIVE_RADIO_CARD_CLASS : RADIO_CARD_CLASS}>
            <input className="radio radio-primary mt-1" type="radio" name="appFilterMode" checked={config.appFilterMode === "blacklist"} onChange={() => update("appFilterMode", "blacklist")} />
            <div>
              <strong>Blacklist</strong>
              <span>Hide any app whose process name exactly matches the blocked list.</span>
            </div>
          </label>
          <label className={config.appFilterMode === "whitelist" ? ACTIVE_RADIO_CARD_CLASS : RADIO_CARD_CLASS}>
            <input className="radio radio-primary mt-1" type="radio" name="appFilterMode" checked={config.appFilterMode === "whitelist"} onChange={() => update("appFilterMode", "whitelist")} />
            <div>
              <strong>Whitelist</strong>
              <span>Only allow apps that exist in the allowed list.</span>
            </div>
          </label>
        </div>

        {config.appFilterMode === "blacklist" ? (
          <ListEditor
            title="Blocked apps"
            description="Blocked process names are dropped from local status output."
            placeholder="wechat.exe"
            value={config.appBlacklist}
            inputValue={blacklistInput}
            onInputValueChange={setBlacklistInput}
            onAdd={() => {
              const value = blacklistInput.trim();
              if (!value) return;
              setConfig((current) => ({ ...current, appBlacklist: appendUniqueListValue(current.appBlacklist, value, false) }));
              setBlacklistInput("");
            }}
            onRemove={(index) => setConfig((current) => ({ ...current, appBlacklist: current.appBlacklist.filter((_, itemIndex) => itemIndex !== index) }))}
            suggestions={appSuggestions}
          />
        ) : (
          <ListEditor
            title="Allowed apps"
            description="Only these process names are allowed to reach the local feed and Discord RPC."
            placeholder="code.exe"
            value={config.appWhitelist}
            inputValue={whitelistInput}
            onInputValueChange={setWhitelistInput}
            onAdd={() => {
              const value = whitelistInput.trim();
              if (!value) return;
              setConfig((current) => ({ ...current, appWhitelist: appendUniqueListValue(current.appWhitelist, value, false) }));
              setWhitelistInput("");
            }}
            onRemove={(index) => setConfig((current) => ({ ...current, appWhitelist: current.appWhitelist.filter((_, itemIndex) => itemIndex !== index) }))}
            suggestions={appSuggestions}
          />
        )}
      </section>

      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Clauses</p>
            <h3>Name-only and media-source rules</h3>
          </div>
        </div>
        <div className="rules-utility-grid">
          <ListEditor
            title="App-name-only list"
            description="These apps keep only the app name. Their window title is masked before local display."
            placeholder="chrome.exe"
            value={config.appNameOnlyList}
            inputValue={nameOnlyInput}
            onInputValueChange={setNameOnlyInput}
            onAdd={() => {
              const value = nameOnlyInput.trim();
              if (!value) return;
              setConfig((current) => ({ ...current, appNameOnlyList: appendUniqueListValue(current.appNameOnlyList, value, false) }));
              setNameOnlyInput("");
            }}
            onRemove={(index) => setConfig((current) => ({ ...current, appNameOnlyList: current.appNameOnlyList.filter((_, itemIndex) => itemIndex !== index) }))}
            suggestions={appSuggestions}
          />
          <ListEditor
            title="Media-source blocklist"
            description="When a play_source hits this list, the media metadata is hidden locally."
            placeholder="system_media"
            value={config.mediaPlaySourceBlocklist}
            inputValue={mediaSourceInput}
            onInputValueChange={setMediaSourceInput}
            onAdd={() => {
              const value = mediaSourceInput.trim().toLowerCase();
              if (!value) return;
              setConfig((current) => ({
                ...current,
                mediaPlaySourceBlocklist: appendUniqueListValue(current.mediaPlaySourceBlocklist, value, true),
              }));
              setMediaSourceInput("");
            }}
            onRemove={(index) => setConfig((current) => ({ ...current, mediaPlaySourceBlocklist: current.mediaPlaySourceBlocklist.filter((_, itemIndex) => itemIndex !== index) }))}
            suggestions={playSourceSuggestions}
            lowercase
          />
        </div>
      </section>

      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">History</p>
            <h3>Saved local records</h3>
          </div>
          <span className={BADGE_CLASS}>{baseState.appHistory.length + baseState.playSourceHistory.length} records</span>
        </div>

        <div className="toggle-grid compact-toggles rules-toggles">
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Capture reported apps</strong>
              <span>Save app names and play sources for local rule suggestions and export.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.captureReportedAppsEnabled} onChange={(e) => update("captureReportedAppsEnabled", e.currentTarget.checked)} />
          </label>
        </div>

        <div className="card-actions gap-2">
          <button
            className={BUTTON_CLASS}
            type="button"
            onClick={() =>
              void navigator.clipboard
                .writeText(JSON.stringify({ appHistory: baseState.appHistory, playSourceHistory: baseState.playSourceHistory }, null, 2))
                .then(() => notify("success", "History copied", "Local history records were copied to the clipboard."))
                .catch(() => notify("error", "Copy failed", "Clipboard access was not available."))
            }
          >
            Copy history JSON
          </button>
          <button
            className={DANGER_BUTTON_CLASS}
            type="button"
            onClick={() => {
              const payload = { ...baseState, appHistory: [], playSourceHistory: [] };
              void persistPayload(payload, false).then(() => notify("info", "History cleared", "Local rule suggestion history was cleared."));
            }}
          >
            Clear history
          </button>
        </div>
      </section>
    </>
  );

  const rulesLauncherView = (
    <section className={PANEL_CLASS}>
      <div className={PANEL_HEAD_CLASS}>
        <div>
          <p className="eyebrow">Rule center</p>
          <h3>Open rule dialog</h3>
        </div>
        <span className={BADGE_CLASS}>
          {config.appMessageRules.length + config.appNameOnlyList.length + config.mediaPlaySourceBlocklist.length} items
        </span>
      </div>
      <div className="rule-entry-grid">
        <div className="rule-entry-copy">
          <strong>Rule clauses, app filter, name-only and media-source lists</strong>
          <p>Detailed rule editing now lives in a secondary dialog so the main settings page stays compact.</p>
          <div className="rule-entry-meta">
            <span className="badge badge-soft">{config.appMessageRules.length} rule groups</span>
            <span className="badge badge-soft">{config.appFilterMode} filter</span>
            <span className="badge badge-soft">{config.appNameOnlyList.length} name-only apps</span>
            <span className="badge badge-soft">{config.mediaPlaySourceBlocklist.length} media blocks</span>
          </div>
        </div>
        <div className="card-actions gap-2">
          <button className={PRIMARY_BUTTON_CLASS} type="button" onClick={() => setRulesDialogOpen(true)}>
            Open rules
          </button>
        </div>
      </div>
    </section>
  );

  const discordView = (
    <>
      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Bridge</p>
            <h3>Discord RPC settings</h3>
          </div>
          <span className={discordSnapshot.connected ? GOOD_BADGE_CLASS : BADGE_CLASS}>
            {discordSnapshot.connected ? "Connected" : "Idle"}
          </span>
        </div>
        <div className="field-grid">
          <label className={FIELD_SPAN_CLASS}>
            <span>Discord application ID</span>
            <input className={INPUT_CLASS} value={config.discordApplicationId} onChange={(e) => update("discordApplicationId", e.currentTarget.value)} placeholder="Your Discord application ID" />
          </label>
        </div>
        <div className="radio-grid discord-mode-grid">
          {DISCORD_REPORT_MODE_OPTIONS.map((option) => (
            <label key={option.mode} className={config.discordReportMode === option.mode ? ACTIVE_RADIO_CARD_CLASS : RADIO_CARD_CLASS}>
              <input
                className="radio radio-primary mt-1"
                type="radio"
                name="discordReportMode"
                checked={config.discordReportMode === option.mode}
                onChange={() => update("discordReportMode", option.mode)}
              />
              <div>
                <strong>{option.title}</strong>
                <span>{option.description}</span>
                <div className="discord-mode-layout">
                  <span>Details: {option.details}</span>
                  <span>State: {option.state}</span>
                </div>
              </div>
            </label>
          ))}
        </div>
        {customDiscordMode ? (
          <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
            <div className="list-editor-summary">
              <div className="list-editor-copy">
                <strong className="block font-semibold">Custom Discord text</strong>
                <p>Edit the two Discord lines directly and choose the Discord activity label.</p>
              </div>
              <span className="badge badge-soft">{discordActivityTypeText(config.discordActivityType)}</span>
            </div>
            <div className="radio-grid discord-activity-grid">
              {DISCORD_ACTIVITY_TYPE_OPTIONS.map((option) => (
                <label key={option.value} className={config.discordActivityType === option.value ? ACTIVE_RADIO_CARD_CLASS : RADIO_CARD_CLASS}>
                  <input
                    className="radio radio-primary mt-1"
                    type="radio"
                    name="discordActivityType"
                    checked={config.discordActivityType === option.value}
                    onChange={() => update("discordActivityType", option.value)}
                  />
                  <div>
                    <strong>{option.label}</strong>
                    <span>{option.helper}</span>
                  </div>
                </label>
              ))}
            </div>
            <div className="field-grid">
              <label className={FIELD_SPAN_CLASS}>
                <span>Details format</span>
                <input
                  ref={discordDetailsInputRef}
                  className={`${INPUT_CLASS} ${activeDiscordFormatField === "details" ? "discord-format-input-active" : ""}`}
                  value={config.discordDetailsFormat}
                  onChange={(e) => update("discordDetailsFormat", e.currentTarget.value)}
                  onFocus={() => setActiveDiscordFormatField("details")}
                  placeholder="{activity}"
                />
              </label>
              <label className={FIELD_SPAN_CLASS}>
                <span>State format</span>
                <input
                  ref={discordStateInputRef}
                  className={`${INPUT_CLASS} ${activeDiscordFormatField === "state" ? "discord-format-input-active" : ""}`}
                  value={config.discordStateFormat}
                  onChange={(e) => update("discordStateFormat", e.currentTarget.value)}
                  onFocus={() => setActiveDiscordFormatField("state")}
                  placeholder="{context}"
                />
              </label>
            </div>
            <div className="token-toolbar-copy">
              <strong>Template tokens</strong>
              <p>Click a token to insert it into {activeDiscordFormatField === "state" ? "State format" : "Details format"}.</p>
            </div>
            <div className="template-token-list">
              {DISCORD_TEMPLATE_TOKENS.map((token) => (
                <button
                  key={token}
                  className="btn btn-outline btn-xs rounded-md normal-case"
                  type="button"
                  title={DISCORD_TEMPLATE_TOKEN_HINTS[token]}
                  aria-label={`${token}: ${DISCORD_TEMPLATE_TOKEN_HINTS[token]}`}
                  onClick={() => insertDiscordToken(token)}
                >
                  {token}
                </button>
              ))}
            </div>
          </div>
        ) : null}
        <div className="toggle-grid compact-toggles">
          {config.discordReportMode === "mixed" ? (
            <label className={TOGGLE_TILE_CLASS}>
              <div>
                <strong>Enable music countdown in Smart mode</strong>
                <span>Keep Discord's song timer in sync while Smart mode is tracking active media.</span>
              </div>
              <input
                className="toggle toggle-primary"
                type="checkbox"
                checked={config.discordSmartEnableMusicCountdown}
                onChange={(e) => update("discordSmartEnableMusicCountdown", e.currentTarget.checked)}
              />
            </label>
          ) : null}
          {config.discordReportMode === "mixed" ? (
            <label className={TOGGLE_TILE_CLASS}>
              <div>
                <strong>Show app name in Smart mode</strong>
                <span>Show the current app name in Smart mode. With music it becomes `title | app` on the first line, and without music it shows only the app name on the last line.</span>
              </div>
              <input
                className="toggle toggle-primary"
                type="checkbox"
                checked={config.discordSmartShowAppName}
                onChange={(e) => update("discordSmartShowAppName", e.currentTarget.checked)}
              />
            </label>
          ) : null}
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Use app artwork</strong>
              <span>Upload the current foreground app icon for Rich Presence. When no music artwork is active, the app icon becomes the main image.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.discordUseAppArtwork} onChange={(e) => update("discordUseAppArtwork", e.currentTarget.checked)} />
          </label>
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Use music artwork</strong>
              <span>Upload media cover art when available and keep the playback source icon attached while music is active.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.discordUseMusicArtwork} onChange={(e) => update("discordUseMusicArtwork", e.currentTarget.checked)} />
          </label>
        </div>
        {config.discordUseAppArtwork || config.discordUseMusicArtwork ? (
          <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
            <div className="list-editor-summary">
              <div className="list-editor-copy">
                <strong className="block font-semibold">Artwork publishing</strong>
                <p>Selected app icons and media cover art are converted to 128x128 JPEG and uploaded through your uploader service, which returns signed public image URLs for Discord with a 3600 second lifetime.</p>
              </div>
              <span className="badge badge-soft">Uploader service</span>
            </div>
            <div className="field-grid">
              <label className={FIELD_SPAN_CLASS}>
                <span>Uploader service URL</span>
                <input
                  className={INPUT_CLASS}
                  value={config.discordArtworkWorkerUploadUrl}
                  onChange={(e) => update("discordArtworkWorkerUploadUrl", e.currentTarget.value)}
                  placeholder="https://your-uploader.example.com/upload"
                />
              </label>
              <label className={FIELD_SPAN_CLASS}>
                <span>Uploader service token</span>
                <input
                  className={INPUT_CLASS}
                  type="password"
                  value={config.discordArtworkWorkerToken}
                  onChange={(e) => update("discordArtworkWorkerToken", e.currentTarget.value)}
                  placeholder="Optional bearer token"
                />
              </label>
            </div>
          </div>
        ) : null}
      </section>

      <section className={PANEL_CLASS}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">State</p>
            <h3>Current bridge status</h3>
          </div>
        </div>
        <div className="stat-grid">
          <div className={STAT_CARD_CLASS}>
            <span>Link state</span>
            <strong>{discordSnapshot.running ? (discordSnapshot.connected ? "Connected" : "Waiting for Discord") : "Stopped"}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Current summary</span>
            <strong>{discordSnapshot.currentSummary || "No local activity is being mirrored yet."}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Report mode</span>
            <strong>{customDiscordMode ? `${discordReportModeText(config)} · ${discordActivityTypeText(config.discordActivityType)}` : discordReportModeText(config)}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Last error</span>
            <strong>{discordSnapshot.lastError || "No Discord runtime error recorded."}</strong>
          </div>
        </div>
      </section>
    </>
  );

  const settingsView = (
    <div className="settings-flow">
      <section className="settings-group">
        <div className="settings-group-head">
          <p className="eyebrow">Step 1</p>
          <h3>RPC setup</h3>
          <p>Save the Discord RPC profile first. Runtime depends on this step.</p>
        </div>
        <div className="compact-stack">{discordView}</div>
      </section>

      <section className="settings-group">
        <div className="settings-group-head">
          <p className="eyebrow">Step 2</p>
          <h3>Rule clauses</h3>
          <p>Open the secondary rule dialog for message clauses, app filter, name-only rules and media blocking.</p>
        </div>
        <div className="compact-stack">{rulesLauncherView}</div>
      </section>

      <section className="settings-group">
        <div className="settings-group-head">
          <p className="eyebrow">Step 3</p>
          <h3>App behavior</h3>
          <p>Keep startup, polling and local monitor behavior at the bottom of the settings page.</p>
        </div>
        <div className="compact-stack">{generalView}</div>
      </section>
    </div>
  );

  const runtimeView = (
    <div className="runtime-layout">
      {!runtimeReady ? (
        <section className={`${PANEL_CLASS} runtime-prerequisite-card`}>
          <div className={PANEL_HEAD_CLASS}>
            <div>
              <p className="eyebrow">Prerequisite</p>
              <h3>RPC setup required</h3>
            </div>
            <span className="badge badge-warning badge-soft">Locked</span>
          </div>
          <div className="empty-state">
            {runtimeBlockReason || "Save the RPC settings first to unlock runtime."}
          </div>
          <div className="card-actions gap-2">
            <button className={PRIMARY_BUTTON_CLASS} type="button" onClick={() => setActiveSection("settings")}>
              Open settings
            </button>
          </div>
        </section>
      ) : null}

      <section className={`${PANEL_CLASS} runtime-card runtime-main-card`}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Monitor</p>
            <h3>Live runtime controls</h3>
          </div>
          <span className={runtimeRunning ? GOOD_BADGE_CLASS : BADGE_CLASS}>
            {runtimeRunning ? "Running" : "Stopped"}
          </span>
        </div>
        <div className="stat-grid">
          <div className={STAT_CARD_CLASS}>
            <span>Current activity</span>
            <strong>{activityText(reporterSnapshot)}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Attached meta</span>
            <strong>{activityMeta(reporterSnapshot)}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Capture mode</span>
            <strong>{captureModeText(config)}</strong>
          </div>
          <div className={STAT_CARD_CLASS}>
            <span>Last heartbeat</span>
            <strong>{formatDate(reporterSnapshot.lastHeartbeatAt)}</strong>
          </div>
        </div>
        <div className="card-actions gap-2">
          <button
            className={PRIMARY_BUTTON_CLASS}
            disabled={busy.startRuntime || runtimeRunning || !runtimeReady}
            onClick={() => runAction("startRuntime", startRuntimeSession)}
          >
            {busy.startRuntime ? "Starting..." : runtimeReady ? "Start runtime" : "RPC required"}
          </button>
          <button
            className={BUTTON_CLASS}
            disabled={busy.stopRuntime || !runtimeRunning}
            onClick={() => runAction("stopRuntime", stopRuntimeSession)}
          >
            Stop runtime
          </button>
          <button className={BUTTON_CLASS} disabled={busy.refreshRuntime || !runtimeReady} onClick={() => runAction("refreshRuntime", async () => { await refreshReporter(); await refreshDiscord(); notify("info", "Runtime refreshed", "The live status panels were updated."); })}>
            {busy.refreshRuntime ? "Refreshing..." : runtimeReady ? "Refresh" : "RPC required"}
          </button>
        </div>
      </section>

      <section className={`${PANEL_CLASS} runtime-card runtime-log-card`}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Log</p>
            <h3>Recent runtime events</h3>
          </div>
        </div>
        <div className="log-feed compact-log-feed runtime-log-feed">
          {runtimeLogs.length === 0 ? (
            <div className="empty-state">No runtime entries yet.</div>
          ) : (
            runtimeLogs.map((entry) => (
              <article key={entry.id} className={logEntryClass(entry.level)}>
                <div className="card-body p-3">
                  <div className="log-entry-head">
                    <strong>{entry.title}</strong>
                    <span>{formatDate(entry.timestamp)}</span>
                  </div>
                  <p>{entry.detail}</p>
                </div>
              </article>
            ))
          )}
        </div>
      </section>

      <section className={`${PANEL_CLASS} runtime-card runtime-debug-card`}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Debug</p>
            <h3>Discord payload JSON</h3>
          </div>
          <button className={BUTTON_CLASS} type="button" onClick={() => setDiscordDebugOpen(true)}>
            Open payload JSON
          </button>
        </div>
        {discordDebugPayload ? (
          <div className="empty-state">
            Open payload JSON to inspect the exact data being pushed into Discord.
          </div>
        ) : (
          <div className="empty-state">
            No Discord payload has been published yet. Start runtime and wait for a captured activity to pass the current rules.
          </div>
        )}
      </section>
    </div>
  );

  function renderSection() {
    switch (activeSection) {
      case "settings":
        return settingsView;
      case "runtime":
        return runtimeView;
      default:
        return settingsView;
    }
  }

  return (
    <main className="shell" data-theme="activityping">
      <section className="app-frame home-shell card border border-base-300 bg-base-100 shadow-xl">
        <aside className="sidebar">
          <div className="sidebar-brand">
            <div className="sidebar-brand-head">
              <img className="sidebar-brand-icon" src={appIcon} alt="ActivityPing icon" />
              <div>
                <p className="eyebrow">ActivityPing</p>
                <h1>Activity relay</h1>
              </div>
            </div>
          </div>

          <nav aria-label="Primary navigation">
            <ul className="menu menu-sm sidebar-menu rounded-box bg-base-200/70 p-2">
              {SECTION_ORDER.map((section) => (
                <li key={section}>
                  <button className={activeSection === section ? "menu-active" : ""} onClick={() => setActiveSection(section)} type="button">
                    <span className="sidebar-menu-copy">
                      <span>{SECTION_COPY[section].kicker}</span>
                      <strong>{SECTION_COPY[section].title}</strong>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </nav>

          <div className="sidebar-guide card border border-base-300 bg-base-200/70 p-3 shadow-none">
            <div className="sidebar-guide-head">
              <span className="eyebrow">Status</span>
              <strong>{sidebarStatus.label}</strong>
            </div>
            <p className="sidebar-guide-note">{sidebarStatus.detail}</p>
            <div className="sidebar-guide-report">
              <span className="eyebrow">Outgoing report</span>
              <p className="sidebar-guide-preview">{reportPreviewText}</p>
            </div>
          </div>
        </aside>

        <section className="content">
          <header className="content-header">
            <div>
              <p className="eyebrow">{activeCopy.kicker}</p>
              <h2>{activeCopy.title}</h2>
              <p>{activeCopy.description}</p>
            </div>
          </header>
          <div className="content-body">{renderSection()}</div>
        </section>
      </section>

      {rulesDialogOpen ? (
        <section className="modal modal-open" onClick={() => setRulesDialogOpen(false)}>
          <div className="modal-box w-11/12 max-w-6xl p-0" role="dialog" aria-modal="true" aria-labelledby="rules-dialog-title" onClick={(event) => event.stopPropagation()}>
            <div className="card-body">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <p className="eyebrow">Rules</p>
                  <h3 id="rules-dialog-title" className="card-title">Detailed rule editor</h3>
                  <p>Edit rule clauses, app filter, name-only and media-source lists here, then save from the bottom-right notice.</p>
                </div>
                <button className={BUTTON_CLASS} type="button" onClick={() => setRulesDialogOpen(false)}>
                  Close
                </button>
              </div>
              <div className="rule-modal-body">{rulesView}</div>
            </div>
          </div>
        </section>
      ) : null}

      {discardDialogOpen ? (
        <section className="modal modal-open" onClick={() => setDiscardDialogOpen(false)}>
          <div className="modal-box w-11/12 max-w-xl p-0" role="dialog" aria-modal="true" aria-labelledby="discard-dialog-title" onClick={(event) => event.stopPropagation()}>
            <div className="card-body">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <p className="eyebrow">Draft</p>
                  <h3 id="discard-dialog-title" className="card-title">Revert unsaved changes?</h3>
                  <p>This resets the current form back to the last saved settings.</p>
                </div>
              </div>
              <div className="card-actions justify-end gap-2">
                <button className={BUTTON_CLASS} type="button" onClick={() => setDiscardDialogOpen(false)}>
                  Cancel
                </button>
                <button
                  className={DANGER_BUTTON_CLASS}
                  type="button"
                  onClick={() => {
                    discardDraftChanges();
                    setDiscardDialogOpen(false);
                  }}
                >
                  Revert changes
                </button>
              </div>
            </div>
          </div>
        </section>
      ) : null}

      {discordDebugOpen ? (
        <section className="modal modal-open" onClick={() => setDiscordDebugOpen(false)}>
          <div
            className="modal-box w-11/12 max-w-4xl p-0"
            role="dialog"
            aria-modal="true"
            aria-labelledby="discord-debug-dialog-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="card-body">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <p className="eyebrow">Debug</p>
                  <h3 id="discord-debug-dialog-title" className="card-title">Discord payload JSON</h3>
                  <p>Inspect the exact payload being pushed into Discord.</p>
                </div>
                <button className={BUTTON_CLASS} type="button" onClick={() => setDiscordDebugOpen(false)}>
                  Close
                </button>
              </div>
              {discordDebugPayload ? (
                <pre className="debug-json">{discordDebugJson}</pre>
              ) : (
                <div className="empty-state">
                  No Discord payload has been published yet. Start runtime and wait for a captured activity to pass the current rules.
                </div>
              )}
            </div>
          </div>
        </section>
      ) : null}

      <section className="toast toast-end toast-bottom">
        {runtimeNeedsRestart ? (
          <article className="save-reminder card card-compact border border-warning bg-base-100 shadow-lg">
            <div className="card-body">
              <div className="save-reminder-copy">
                <span className="badge badge-warning badge-soft">Restart</span>
                <div className="notice-copy">
                  <strong>Runtime restart required</strong>
                  <span>Restart runtime to apply the saved configuration changes.</span>
                </div>
              </div>
            </div>
          </article>
        ) : null}
        {dirty ? (
          <article className="save-reminder card card-compact border border-base-300 bg-base-100 shadow-lg">
            <div className="card-body">
              <div className="save-reminder-copy">
                <span className="badge badge-warning badge-soft">Draft</span>
                <div className="notice-copy">
                  <strong>Unsaved draft</strong>
                  <span>Changes in the current form are not saved yet.</span>
                </div>
              </div>
              <div className="save-reminder-actions">
                <button
                  className={BUTTON_CLASS}
                  type="button"
                  disabled={busy.saveDraft}
                  onClick={() => setDiscardDialogOpen(true)}
                >
                  Revert changes
                </button>
                <button
                  className={PRIMARY_BUTTON_CLASS}
                  type="button"
                  disabled={busy.saveDraft}
                  onClick={() => runAction("saveDraft", async () => saveProfile("Changes saved", "The current settings were saved."))}
                >
                  {busy.saveDraft ? "Saving..." : "Save changes"}
                </button>
              </div>
            </div>
          </article>
        ) : null}
        {notices.map((item) => (
          <article key={item.id} className={alertClass(item.tone)}>
            <div className="notice-copy">
              <strong>{item.title}</strong>
              <span>{item.detail}</span>
            </div>
          </article>
        ))}
      </section>
    </main>
  );
}

export default App;
