import { startTransition, useEffect, useMemo, useRef, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AnimatePresence, motion } from "motion/react";

import "./App.css";
import appIcon from "./assets/app-icon-base.png";
import { ListEditor, SuggestionInput } from "./components/ListEditor";
import {
  defaultClientConfig,
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
import { exportRulesJson, normalizeClientConfig, parseRulesJson, summarizeRuleGroup } from "./lib/rules";
import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppHistoryEntry,
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordAppNameMode,
  DiscordRichPresenceButtonConfig,
  DiscordCustomPreset,
  DiscordActivityType,
  DiscordDebugPayload,
  DiscordPresenceSnapshot,
  DiscordReportMode,
  DiscordStatusDisplay,
  PlatformProbeResult,
  PlatformSelfTestResult,
  PlaySourceHistoryEntry,
  ReporterActivity,
  ReporterLogEntry,
  RealtimeReporterSnapshot,
} from "./types";

type NoticeTone = "info" | "success" | "warn" | "error";
type ViewSection = "runtime" | "settings" | "about";

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
const GITHUB_URL = "https://github.com/MoYoez/ActivityPing";
const MAX_RUNTIME_LOGS = 20;
const RUNTIME_LOG_PAGE_SIZE = 6;
const DEFAULT_HISTORY_RECORD_LIMIT = 3;
const DEFAULT_HISTORY_TITLE_LIMIT = 5;
const MIN_HISTORY_LIMIT = 1;
const MAX_HISTORY_LIMIT = 50;
const RULE_GROUP_PAGE_SIZE = 6;
const TITLE_RULE_PAGE_SIZE = 3;
const CUSTOM_PRESET_PAGE_SIZE = 5;
const DISCORD_CUSTOM_LINE_CUSTOM_VALUE = "__custom__";
const DISCORD_CUSTOM_LINE_OPTIONS = [
  { value: "", label: "Hidden", helper: "Do not send this line." },
  { value: "{activity}", label: "Current activity", helper: "Use the current activity text after rules are applied." },
  { value: "{context}", label: "Current context", helper: "Use the current secondary context chosen by the active mode." },
  { value: "{app}", label: "App name", helper: "Use the captured process or app name." },
  { value: "{title}", label: "Window title", helper: "Use the current window title when available." },
  { value: "{rule}", label: "Matched rule", helper: "Use the matched rule text when a rule hit exists." },
  { value: "{media}", label: "Music summary", helper: "Use song, artist, and album together when music is active." },
  { value: "{song}", label: "Song name", helper: "Use the current song or media title." },
  { value: "{artist}", label: "Artist", helper: "Use the current artist." },
  { value: "{album}", label: "Album", helper: "Use the current album name." },
  { value: "{source}", label: "Source app", helper: "Use the current playback source app." },
  {
    value: DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
    label: "Custom text",
    helper: "Type your own text. Template tokens like {app}, {song}, and {artist} still work.",
  },
] as const;
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
    description: "Use the matched title on line 1, optionally keep the foreground app on line 2, and move visible music to line 3.",
    details: "Foreground app",
    state: "Music line",
  },
  {
    mode: "music",
    title: "Music",
    description: "Use a music-first layout similar to inflink-rs when playback is active.",
    details: "Song / media title",
    state: "Artist",
  },
  {
    mode: "app",
    title: "App",
    description: "Use the matched title on line 1 and the foreground app on line 2 when app reporting is enabled.",
    details: "Foreground app",
    state: "Hidden",
  },
  {
    mode: "custom",
    title: "Custom",
    description: "Edit the default Discord lines directly, then keep reusable Custom presets for quick import.",
    details: "Preset or default template",
    state: "Preset or default template",
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
const DISCORD_STATUS_DISPLAY_OPTIONS: Array<{
  value: DiscordStatusDisplay;
  label: string;
  helper: string;
}> = [
  { value: "name", label: "Application name", helper: "Use the activity name for Discord's compact member-list status text." },
  { value: "state", label: "State line", helper: "Use the state field for Discord's compact member-list status text." },
  { value: "details", label: "Details line", helper: "Use the details field for Discord's compact member-list status text." },
];
const DISCORD_APP_NAME_OPTIONS: Array<{
  value: DiscordAppNameMode;
  label: string;
  helper: string;
}> = [
  { value: "default", label: "Application name", helper: "Keep Discord's application name. Smart and App mode still use the matched title first when an app is active." },
  { value: "song", label: "Song name", helper: "Use the current track title when the activity falls back to music-first output." },
  { value: "artist", label: "Artist", helper: "Use the current artist when the activity falls back to music-first output." },
  { value: "album", label: "Album", helper: "Use the current album when the activity falls back to music-first output." },
  { value: "custom", label: "Custom text", helper: "Type a custom application name for the first line." },
];
const DISCORD_TEMPLATE_TOKENS = [
  "{app}",
  "{title}",
  "{rule}",
  "{media}",
  "{song}",
  "{artist}",
  "{album}",
  "{source}",
] as const;
const VIEW_MOTION = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -6 },
};
const CARD_MOTION = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
};
const LOG_MOTION = {
  initial: { opacity: 0, x: 10 },
  animate: { opacity: 1, x: 0 },
};
const MOTION_TRANSITION = { duration: 0.18, ease: "easeOut" } as const;

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

interface JsonViewerState {
  eyebrow: string;
  title: string;
  description: string;
  value: unknown | null;
  emptyText: string;
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
  runtime: {
    kicker: "Runtime",
    title: "Live monitor",
    description: "Watch captured activity, current RPC output and the recent runtime log. Requires a saved RPC profile first.",
  },
  settings: {
    kicker: "Settings",
    title: "RPC and local rules",
    description: "Configure Discord RPC first, then tune monitor behavior and local rule clauses in one place.",
  },
  about: {
    kicker: "About",
    title: "ActivityPing",
    description: "Project links and build information.",
  },
};

const SECTION_ORDER: ViewSection[] = ["runtime", "settings", "about"];

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
  if (config.reportStoppedMedia) parts.push("Paused media");
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

function discordReportModeName(mode: DiscordReportMode) {
  switch (mode) {
    case "music":
      return "Music";
    case "app":
      return "App";
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

function localWorkingModeText(config: ClientConfig) {
  switch (config.discordReportMode) {
    case "music":
      return "Music mode";
    case "app":
      return "App mode";
    case "custom":
      return `Custom Discord Text · ${discordActivityTypeText(config.discordActivityType)}`;
    default:
      return "Smart mode";
  }
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

function compactOptionalText(value?: string | null) {
  const trimmed = value?.trim() ?? "";
  return trimmed || null;
}

function clampHistoryLimit(value: unknown, fallback: number) {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(MAX_HISTORY_LIMIT, Math.max(MIN_HISTORY_LIMIT, parsed));
}

function normalizeTitleHistory(values: unknown, fallback?: string | null, limit = DEFAULT_HISTORY_TITLE_LIMIT) {
  const result: string[] = [];
  const seen = new Set<string>();
  const candidates = [
    ...(Array.isArray(values) ? values : []),
    fallback,
  ];

  for (const value of candidates) {
    const trimmed = String(value ?? "").trim();
    if (!trimmed || seen.has(trimmed)) continue;
    seen.add(trimmed);
    result.push(trimmed);
    if (result.length >= limit) break;
  }

  return result;
}

function sameStringList(left: string[] = [], right: string[] = []) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function mergeTitleHistory(existing: AppHistoryEntry | undefined, rawTitle: string | null, limit: number) {
  const currentTitles = normalizeTitleHistory(existing?.processTitles, existing?.processTitle, limit);
  if (!rawTitle) return currentTitles.slice(0, limit);
  return normalizeTitleHistory([rawTitle, ...currentTitles], null, limit);
}

function normalizeAppHistory(
  values: unknown,
  recordLimit = DEFAULT_HISTORY_RECORD_LIMIT,
  titleLimit = DEFAULT_HISTORY_TITLE_LIMIT,
): AppHistoryEntry[] {
  if (!Array.isArray(values)) return [];
  return values
    .map((value): AppHistoryEntry | null => {
      if (typeof value === "string") {
        return { processName: value.trim(), processTitle: null, processTitles: [], statusText: null, updatedAt: null };
      }
      if (!value || typeof value !== "object") {
        return null;
      }
      const record = value as Partial<AppHistoryEntry>;
      return {
        processName: String(record.processName ?? "").trim(),
        processTitle: compactOptionalText(record.processTitle),
        processTitles: normalizeTitleHistory(record.processTitles, record.processTitle, titleLimit),
        statusText: compactOptionalText(record.statusText),
        updatedAt: compactOptionalText(record.updatedAt),
      };
    })
    .filter((value): value is AppHistoryEntry => Boolean(value?.processName))
    .slice(0, recordLimit);
}

function normalizePlaySourceHistory(
  values: unknown,
  recordLimit = DEFAULT_HISTORY_RECORD_LIMIT,
): PlaySourceHistoryEntry[] {
  if (!Array.isArray(values)) return [];
  return values
    .map((value): PlaySourceHistoryEntry | null => {
      if (typeof value === "string") {
        return {
          source: value.trim().toLowerCase(),
          mediaTitle: null,
          mediaArtist: null,
          mediaAlbum: null,
          mediaSummary: null,
          updatedAt: null,
        };
      }
      if (!value || typeof value !== "object") {
        return null;
      }
      const record = value as Partial<PlaySourceHistoryEntry>;
      return {
        source: String(record.source ?? "").trim().toLowerCase(),
        mediaTitle: compactOptionalText(record.mediaTitle),
        mediaArtist: compactOptionalText(record.mediaArtist),
        mediaAlbum: compactOptionalText(record.mediaAlbum),
        mediaSummary: compactOptionalText(record.mediaSummary),
        updatedAt: compactOptionalText(record.updatedAt),
      };
    })
    .filter((value): value is PlaySourceHistoryEntry => Boolean(value?.source))
    .slice(0, recordLimit);
}

function uniqueHistoryValues(values: string[]) {
  const seen = new Set<string>();
  return values.filter((value) => {
    const key = value.trim().toLowerCase();
    if (!key || seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function sameOptionalValue(left?: string | null, right?: string | null) {
  return (left?.trim() || "") === (right?.trim() || "");
}

function sameAppHistoryContent(left: AppHistoryEntry, right: AppHistoryEntry) {
  return (
    left.processName.trim().toLowerCase() === right.processName.trim().toLowerCase() &&
    sameOptionalValue(left.processTitle, right.processTitle) &&
    sameStringList(left.processTitles, right.processTitles) &&
    sameOptionalValue(left.statusText, right.statusText)
  );
}

function samePlaySourceHistoryContent(left: PlaySourceHistoryEntry, right: PlaySourceHistoryEntry) {
  return (
    left.source.trim().toLowerCase() === right.source.trim().toLowerCase() &&
    sameOptionalValue(left.mediaTitle, right.mediaTitle) &&
    sameOptionalValue(left.mediaArtist, right.mediaArtist) &&
    sameOptionalValue(left.mediaAlbum, right.mediaAlbum) &&
    sameOptionalValue(left.mediaSummary, right.mediaSummary)
  );
}

function shouldCaptureHistoryActivity(activity?: ReporterActivity | null) {
  const processName = activity?.processName?.trim().toLowerCase() ?? "";
  return Boolean(processName && processName !== "activityping.exe");
}

function mergeAppHistory(
  values: AppHistoryEntry[],
  activity: ReporterActivity | null | undefined,
  recordLimit: number,
  titleLimit: number,
) {
  const processName = activity?.processName?.trim() ?? "";
  if (!processName) return values;
  const key = processName.toLowerCase();
  const existingEntry = values.find((item) => item.processName.trim().toLowerCase() === key);
  const rawTitle = compactOptionalText(activity?.rawProcessTitle) ?? compactOptionalText(activity?.processTitle);
  const entry: AppHistoryEntry = {
    processName,
    processTitle: compactOptionalText(activity?.processTitle),
    processTitles: mergeTitleHistory(existingEntry, rawTitle, titleLimit),
    statusText: compactOptionalText(activity?.statusText),
    updatedAt: activity?.updatedAt ?? new Date().toISOString(),
  };
  if (values[0] && sameAppHistoryContent(values[0], entry)) {
    return values;
  }
  return [entry, ...values.filter((item) => item.processName.trim().toLowerCase() !== key)].slice(0, recordLimit);
}

function mergePlaySourceHistory(
  values: PlaySourceHistoryEntry[],
  activity: ReporterActivity | null | undefined,
  recordLimit: number,
) {
  const source = activity?.playSource?.trim().toLowerCase() ?? "";
  if (!source) return values;
  const entry: PlaySourceHistoryEntry = {
    source,
    mediaTitle: compactOptionalText(activity?.mediaTitle),
    mediaArtist: compactOptionalText(activity?.mediaArtist),
    mediaAlbum: compactOptionalText(activity?.mediaAlbum),
    mediaSummary: compactOptionalText(activity?.mediaSummary),
    updatedAt: activity?.updatedAt ?? new Date().toISOString(),
  };
  if (values[0] && samePlaySourceHistoryContent(values[0], entry)) {
    return values;
  }
  return [entry, ...values.filter((item) => item.source.trim().toLowerCase() !== source)].slice(0, recordLimit);
}

function appHistoryDisplayTitle(entry: AppHistoryEntry) {
  return entry.statusText?.trim() || entry.processTitle?.trim() || "No title captured";
}

function appHistoryRawTitles(entry: AppHistoryEntry) {
  return normalizeTitleHistory(entry.processTitles, entry.processTitle, MAX_HISTORY_LIMIT);
}

function playSourceHistoryDisplayTitle(entry: PlaySourceHistoryEntry) {
  return entry.mediaTitle?.trim() || entry.mediaSummary?.trim() || "No media title captured";
}

function playSourceHistoryMeta(entry: PlaySourceHistoryEntry) {
  return [entry.mediaArtist, entry.mediaAlbum].map((item) => item?.trim()).filter(Boolean).join(" · ");
}

function hasJsonPayload(value: unknown) {
  return typeof value === "object" && value !== null && Object.keys(value).length > 0;
}

function sameJsonValue(left: unknown, right: unknown) {
  return JSON.stringify(left) === JSON.stringify(right);
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

function createDiscordButton(): DiscordRichPresenceButtonConfig {
  return { label: "", url: "" };
}

function createAppMessageRuleGroup(): AppMessageRuleGroup {
  return {
    processMatch: "",
    defaultText: "",
    titleRules: [],
    buttons: [],
    partyId: "",
    partySizeCurrent: null,
    partySizeMax: null,
    joinSecret: "",
    spectateSecret: "",
    matchSecret: "",
  };
}

function appendDiscordTemplateToken(currentValue: string, token: string) {
  const trimmed = String(currentValue ?? "").trim();
  return trimmed ? `${trimmed} ${token}` : token;
}

function DiscordTemplateTokenRow({
  onInsert,
}: {
  onInsert: (token: string) => void;
}) {
  return (
    <div className="discord-template-token-row">
      <span>Quick insert</span>
      {DISCORD_TEMPLATE_TOKENS.map((token) => (
        <button
          key={token}
          className="btn btn-ghost btn-xs no-animation"
          type="button"
          onClick={() => onInsert(token)}
        >
          {token}
        </button>
      ))}
    </div>
  );
}

function DiscordOptionHelp({
  idPrefix,
  includeSmartModeNote = true,
}: {
  idPrefix: string;
  includeSmartModeNote?: boolean;
}) {
  const helpEntries = [
    ...DISCORD_CUSTOM_LINE_OPTIONS.map((option) => ({
      key: `${idPrefix}-line-${option.value || "hidden"}`,
      label: option.label,
      helper: option.helper,
    })),
    ...DISCORD_STATUS_DISPLAY_OPTIONS.map((option) => ({
      key: `${idPrefix}-status-${option.value}`,
      label: option.label,
      helper: option.helper,
    })),
    ...DISCORD_APP_NAME_OPTIONS.map((option) => ({
      key: `${idPrefix}-app-name-${option.value}`,
      label: option.label,
      helper: option.helper,
    })),
    ...(includeSmartModeNote
      ? [
          {
            key: `${idPrefix}-smart-mode`,
            label: "Smart mode",
            helper: "line 1 follows the matched title, line 2 can show the foreground app, and line 3 carries music when playback is active.",
          },
        ]
      : []),
  ];

  return (
    <details className="dropdown dropdown-end discord-option-help">
      <summary className="discord-option-help-trigger" aria-label="Show Discord option help">
        ?
      </summary>
      <div className="discord-option-help-panel dropdown-content z-[20] w-[min(38rem,calc(100vw-2rem))] max-w-[calc(100vw-2rem)] rounded-box border border-base-300 bg-base-100 p-4 text-sm text-base-content/70 shadow-xl">
        <div className="discord-option-help-grid">
          {helpEntries.map((entry) => (
            <p key={entry.key}>
              <strong>{entry.label}:</strong> {entry.helper}
            </p>
          ))}
        </div>
      </div>
    </details>
  );
}

function discordModeStatusDisplay(config: ClientConfig, mode: DiscordReportMode) {
  switch (mode) {
    case "music":
      return config.discordMusicStatusDisplay;
    case "app":
      return config.discordAppStatusDisplay;
    case "custom":
      return config.discordCustomModeStatusDisplay;
    default:
      return config.discordSmartStatusDisplay;
  }
}

function discordModeAppNameMode(config: ClientConfig, mode: DiscordReportMode) {
  switch (mode) {
    case "music":
      return config.discordMusicAppNameMode;
    case "app":
      return config.discordAppAppNameMode;
    case "custom":
      return config.discordCustomModeAppNameMode;
    default:
      return config.discordSmartAppNameMode;
  }
}

function discordModeCustomAppName(config: ClientConfig, mode: DiscordReportMode) {
  switch (mode) {
    case "music":
      return config.discordMusicCustomAppName;
    case "app":
      return config.discordAppCustomAppName;
    case "custom":
      return config.discordCustomModeCustomAppName;
    default:
      return config.discordSmartCustomAppName;
  }
}

function patchDiscordModeSettings(
  config: ClientConfig,
  mode: DiscordReportMode,
  patch: {
    statusDisplay?: DiscordStatusDisplay;
    appNameMode?: DiscordAppNameMode;
    customAppName?: string;
  },
): ClientConfig {
  switch (mode) {
    case "music":
      return {
        ...config,
        ...(patch.statusDisplay ? { discordMusicStatusDisplay: patch.statusDisplay } : {}),
        ...(patch.appNameMode ? { discordMusicAppNameMode: patch.appNameMode } : {}),
        ...(patch.customAppName !== undefined ? { discordMusicCustomAppName: patch.customAppName } : {}),
      };
    case "app":
      return {
        ...config,
        ...(patch.statusDisplay ? { discordAppStatusDisplay: patch.statusDisplay } : {}),
        ...(patch.appNameMode ? { discordAppAppNameMode: patch.appNameMode } : {}),
        ...(patch.customAppName !== undefined ? { discordAppCustomAppName: patch.customAppName } : {}),
      };
    case "custom":
      return {
        ...config,
        ...(patch.statusDisplay ? { discordCustomModeStatusDisplay: patch.statusDisplay } : {}),
        ...(patch.appNameMode ? { discordCustomModeAppNameMode: patch.appNameMode } : {}),
        ...(patch.customAppName !== undefined ? { discordCustomModeCustomAppName: patch.customAppName } : {}),
      };
    default:
      return {
        ...config,
        ...(patch.statusDisplay ? { discordSmartStatusDisplay: patch.statusDisplay } : {}),
        ...(patch.appNameMode ? { discordSmartAppNameMode: patch.appNameMode } : {}),
        ...(patch.customAppName !== undefined ? { discordSmartCustomAppName: patch.customAppName } : {}),
      };
  }
}

function discordLineOptionLabel(value: string) {
  const normalizedValue = normalizeDiscordLineTemplate(value);
  if (!normalizedValue) {
    return "Hidden";
  }
  const matched = DISCORD_CUSTOM_LINE_OPTIONS.find(
    (option) => option.value === normalizedValue && option.value !== DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
  )?.label;
  return matched ?? "Custom text";
}

function normalizeDiscordLineTemplate(value: unknown) {
  const trimmed = String(value ?? "").trim();
  return trimmed === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? "" : trimmed;
}

function isDiscordBuiltinLineChoice(value: string) {
  return DISCORD_CUSTOM_LINE_OPTIONS.some(
    (option) => option.value === value && option.value !== DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
  );
}

function resolveDiscordLineChoice(value: string, forceCustom = false) {
  if (forceCustom) {
    return DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
  }
  const rawValue = String(value ?? "").trim();
  if (rawValue === DISCORD_CUSTOM_LINE_CUSTOM_VALUE) {
    return DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
  }

  const normalizedValue = normalizeDiscordLineTemplate(value);
  if (!normalizedValue) {
    return "";
  }

  return isDiscordBuiltinLineChoice(normalizedValue)
    ? normalizedValue
    : DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
}

function discordLineCustomTextValue(value: string) {
  const rawValue = String(value ?? "").trim();
  if (!rawValue || rawValue === DISCORD_CUSTOM_LINE_CUSTOM_VALUE) {
    return "";
  }
  return isDiscordBuiltinLineChoice(rawValue) ? "" : rawValue;
}

function nextDiscordLineValue(currentValue: string, nextChoice: string) {
  if (nextChoice !== DISCORD_CUSTOM_LINE_CUSTOM_VALUE) {
    return nextChoice;
  }

  const currentCustomValue = discordLineCustomTextValue(currentValue);
  return currentCustomValue || DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
}

function createDiscordCustomPreset(): DiscordCustomPreset {
  return {
    name: "",
    activityType: "playing",
    statusDisplay: "name",
    appNameMode: "default",
    customAppName: "",
    detailsFormat: "{activity}",
    stateFormat: "{context}",
    buttons: [],
    partyId: "",
    partySizeCurrent: null,
    partySizeMax: null,
    joinSecret: "",
    spectateSecret: "",
    matchSecret: "",
  };
}

function createDiscordCustomPresetFromConfig(config: ClientConfig): DiscordCustomPreset {
  return {
    name: "",
    activityType: config.discordActivityType,
    statusDisplay: config.discordCustomModeStatusDisplay,
    appNameMode: config.discordCustomModeAppNameMode,
    customAppName: config.discordCustomModeCustomAppName,
    detailsFormat: normalizeDiscordLineTemplate(config.discordDetailsFormat),
    stateFormat: normalizeDiscordLineTemplate(config.discordStateFormat),
    buttons: config.discordCustomButtons.map((button) => ({ ...button })),
    partyId: config.discordCustomPartyId,
    partySizeCurrent: config.discordCustomPartySizeCurrent ?? null,
    partySizeMax: config.discordCustomPartySizeMax ?? null,
    joinSecret: config.discordCustomJoinSecret,
    spectateSecret: config.discordCustomSpectateSecret,
    matchSecret: config.discordCustomMatchSecret,
  };
}

function summarizeDiscordCustomPreset(preset: DiscordCustomPreset) {
  const extras = [];
  if (preset.buttons.length > 0) extras.push(`${preset.buttons.length} button${preset.buttons.length === 1 ? "" : "s"}`);
  if (preset.partyId.trim() || preset.partySizeCurrent || preset.partySizeMax) extras.push("party");
  if (preset.joinSecret.trim() || preset.spectateSecret.trim() || preset.matchSecret.trim()) extras.push("secrets");
  const extraText = extras.length > 0 ? ` · ${extras.join(" · ")}` : "";
  return `${discordActivityTypeText(preset.activityType)} · ${DISCORD_STATUS_DISPLAY_OPTIONS.find((option) => option.value === preset.statusDisplay)?.label ?? "Application name"} · ${discordLineOptionLabel(preset.detailsFormat.trim())} / ${discordLineOptionLabel(preset.stateFormat.trim())}${extraText}`;
}

function normalizePositiveNumberInput(value: string) {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }
  return parsed;
}

function validateRuleRegex(config: ClientConfig) {
  for (const group of config.appMessageRules) {
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

function validateArtworkPublishing(config: ClientConfig) {
  if (
    (config.discordUseAppArtwork || config.discordUseMusicArtwork) &&
    !config.discordArtworkWorkerUploadUrl.trim()
  ) {
    return "Use app artwork or Use music artwork requires an uploader service URL in Artwork publishing.";
  }
  return null;
}

function buildPayload(baseState: AppStatePayload, config: ClientConfig): AppStatePayload {
  const normalizedConfig = normalizeClientConfig(config);
  const historyRecordLimit = clampHistoryLimit(normalizedConfig.captureHistoryRecordLimit, DEFAULT_HISTORY_RECORD_LIMIT);
  const historyTitleLimit = clampHistoryLimit(normalizedConfig.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
  return {
    ...baseState,
    config: normalizedConfig,
    appHistory: normalizeAppHistory(baseState.appHistory, historyRecordLimit, historyTitleLimit),
    playSourceHistory: normalizePlaySourceHistory(baseState.playSourceHistory, historyRecordLimit),
    locale: "en-US",
  };
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
  const [activeSection, setActiveSection] = useState<ViewSection>("runtime");
  const [activeRuleIndex, setActiveRuleIndex] = useState(0);
  const [rulesImportOpen, setRulesImportOpen] = useState(false);
  const [rulesImportValue, setRulesImportValue] = useState("");
  const [blacklistInput, setBlacklistInput] = useState("");
  const [whitelistInput, setWhitelistInput] = useState("");
  const [nameOnlyInput, setNameOnlyInput] = useState("");
  const [mediaSourceInput, setMediaSourceInput] = useState("");
  const [rulesDialogOpen, setRulesDialogOpen] = useState(false);
  const [customRulesDialogOpen, setCustomRulesDialogOpen] = useState(false);
  const [customPresetPage, setCustomPresetPage] = useState(0);
  const [activeCustomPresetIndex, setActiveCustomPresetIndex] = useState<number | null>(null);
  const [discordDetailsForceCustomChoice, setDiscordDetailsForceCustomChoice] = useState(false);
  const [discordStateForceCustomChoice, setDiscordStateForceCustomChoice] = useState(false);
  const [presetDetailsForceCustomChoice, setPresetDetailsForceCustomChoice] = useState(false);
  const [presetStateForceCustomChoice, setPresetStateForceCustomChoice] = useState(false);
  const [discardDialogOpen, setDiscardDialogOpen] = useState(false);
  const [appliedRuntimeConfigSignature, setAppliedRuntimeConfigSignature] = useState<string | null>(null);
  const [ruleGroupPage, setRuleGroupPage] = useState(0);
  const [titleRulePage, setTitleRulePage] = useState(0);
  const [runtimeLogPage, setRuntimeLogPage] = useState(0);
  const [jsonViewer, setJsonViewer] = useState<JsonViewerState | null>(null);
  const runtimeAutostartAttemptedRef = useRef(false);

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
            <span>Working mode</span>
            <strong>{currentLocalModeText}</strong>
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
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Force Custom add-ons override rule add-ons</strong>
              <span>When Custom add-ons are configured, reuse them instead of the matched rule group's buttons or social metadata.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.discordUseCustomAddonsOverride} onChange={(e) => update("discordUseCustomAddonsOverride", e.currentTarget.checked)} />
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
                appMessageRules: [...current.appMessageRules, createAppMessageRuleGroup()],
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
                    discordUseCustomAddonsOverride: parsed.data.discordUseCustomAddonsOverride,
                    discordCustomPresets: parsed.data.discordCustomPresets,
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

                <details className="discord-advanced-panel rounded-box border border-base-300 bg-base-200/45 p-4">
                  <summary className="discord-advanced-summary flex cursor-pointer list-none items-center justify-between gap-3">
                    <div>
                      <strong className="block font-semibold">Discord add-ons</strong>
                      <p className="mt-1 text-sm text-base-content/70">
                        The matched rule group can publish buttons, party metadata, or social secrets outside Custom mode.
                      </p>
                    </div>
                    <div className="discord-advanced-summary-meta">
                      {activeRule.buttons.length > 0 ? <span className="badge badge-soft">{activeRule.buttons.length} / 2 buttons</span> : null}
                      {activeRuleAdvancedAddonsConfigured ? <span className="badge badge-soft">Configured</span> : null}
                      {config.discordUseCustomAddonsOverride && customAddonsConfigured ? <span className="badge badge-soft">Custom override</span> : null}
                      <span className="discord-advanced-summary-hint" aria-hidden="true">
                        <span className="discord-advanced-summary-hint-closed">Expand</span>
                        <span className="discord-advanced-summary-hint-open">Collapse</span>
                        <span className="discord-advanced-summary-caret">v</span>
                      </span>
                    </div>
                  </summary>
                  <div className="mt-4 space-y-4">
                    <div className="space-y-3">
                      {activeRule.buttons.map((button, index) => (
                        <div key={`rule-${activeRuleIndex}-discord-button-${index}`} className="rounded-box border border-base-300 bg-base-100/80 p-3 space-y-3">
                          <div className="field-grid">
                            <label className={FIELD_CLASS}>
                              <span>Button label</span>
                              <input
                                className={INPUT_CLASS}
                                value={button.label}
                                onChange={(e) => patchRuleDiscordButtonAt(activeRuleIndex, index, (current) => ({ ...current, label: e.currentTarget.value }))}
                                placeholder="Open website"
                              />
                            </label>
                            <label className={FIELD_CLASS}>
                              <span>Button URL</span>
                              <input
                                className={INPUT_CLASS}
                                value={button.url}
                                onChange={(e) => patchRuleDiscordButtonAt(activeRuleIndex, index, (current) => ({ ...current, url: e.currentTarget.value }))}
                                placeholder="https://example.com or myapp://open"
                              />
                            </label>
                          </div>
                          <div className="card-actions justify-end">
                            <button
                              className={DANGER_BUTTON_CLASS}
                              type="button"
                              onClick={() =>
                                patchRuleAt(activeRuleIndex, (rule) => ({
                                  ...rule,
                                  buttons: rule.buttons.filter((_, itemIndex) => itemIndex !== index),
                                }))
                              }
                            >
                              Remove button
                            </button>
                          </div>
                        </div>
                      ))}
                      {activeRule.buttons.length < 2 ? (
                        <button
                          className={BUTTON_CLASS}
                          type="button"
                          onClick={() =>
                            patchRuleAt(activeRuleIndex, (rule) => ({
                              ...rule,
                              buttons: [...rule.buttons, createDiscordButton()],
                            }))
                          }
                        >
                          Add button
                        </button>
                      ) : null}
                    </div>

                    <details className="discord-advanced-panel rounded-box border border-base-300 bg-base-100/70 p-4">
                      <summary className="discord-advanced-summary flex cursor-pointer list-none items-center justify-between gap-3">
                        <div>
                          <strong className="block font-semibold">Advanced</strong>
                          <p className="mt-1 text-sm text-base-content/70">Party metadata and Discord social secrets.</p>
                        </div>
                        <div className="discord-advanced-summary-meta">
                          {activeRuleAdvancedAddonsConfigured ? <span className="badge badge-soft">Configured</span> : null}
                          <span className="discord-advanced-summary-hint" aria-hidden="true">
                            <span className="discord-advanced-summary-hint-closed">Expand</span>
                            <span className="discord-advanced-summary-hint-open">Collapse</span>
                            <span className="discord-advanced-summary-caret">v</span>
                          </span>
                        </div>
                      </summary>
                      <div className="field-grid mt-4">
                        <label className={FIELD_CLASS}>
                          <span>Party ID</span>
                          <input
                            className={INPUT_CLASS}
                            value={activeRule.partyId}
                            onChange={(e) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, partyId: e.currentTarget.value }))}
                            placeholder="party-123"
                          />
                        </label>
                        <label className={FIELD_CLASS}>
                          <span>Party current size</span>
                          <input
                            className={INPUT_CLASS}
                            type="number"
                            min={1}
                            value={activeRule.partySizeCurrent ?? ""}
                            onChange={(e) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, partySizeCurrent: normalizePositiveNumberInput(e.currentTarget.value) }))}
                            placeholder="2"
                          />
                        </label>
                        <label className={FIELD_CLASS}>
                          <span>Party max size</span>
                          <input
                            className={INPUT_CLASS}
                            type="number"
                            min={1}
                            value={activeRule.partySizeMax ?? ""}
                            onChange={(e) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, partySizeMax: normalizePositiveNumberInput(e.currentTarget.value) }))}
                            placeholder="3"
                          />
                        </label>
                        <label className={FIELD_SPAN_CLASS}>
                          <span>Join secret</span>
                          <input
                            className={INPUT_CLASS}
                            value={activeRule.joinSecret}
                            onChange={(e) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, joinSecret: e.currentTarget.value }))}
                            placeholder="join-secret"
                          />
                        </label>
                        <label className={FIELD_SPAN_CLASS}>
                          <span>Spectate secret</span>
                          <input
                            className={INPUT_CLASS}
                            value={activeRule.spectateSecret}
                            onChange={(e) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, spectateSecret: e.currentTarget.value }))}
                            placeholder="spectate-secret"
                          />
                        </label>
                        <label className={FIELD_SPAN_CLASS}>
                          <span>Match secret</span>
                          <input
                            className={INPUT_CLASS}
                            value={activeRule.matchSecret}
                            onChange={(e) => patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, matchSecret: e.currentTarget.value }))}
                            placeholder="match-secret"
                          />
                        </label>
                      </div>
                    </details>
                  </div>
                </details>
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
          <span className={BADGE_CLASS}>
            {baseState.appHistory.length + baseState.playSourceHistory.length} records · {appRawTitleCount} raw titles
          </span>
        </div>

        <div className="toggle-grid compact-toggles rules-toggles">
          <label className={TOGGLE_TILE_CLASS}>
            <div>
              <strong>Capture reported apps</strong>
              <span>Save recent app and play-source records for local suggestions and export.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={config.captureReportedAppsEnabled} onChange={(e) => update("captureReportedAppsEnabled", e.currentTarget.checked)} />
          </label>
        </div>
        <div className="field-grid compact-fields">
          <label className={FIELD_CLASS}>
            <span>Records per list</span>
            <input
              className={INPUT_CLASS}
              type="number"
              min={MIN_HISTORY_LIMIT}
              max={MAX_HISTORY_LIMIT}
              value={historyRecordLimit}
              onChange={(e) => update("captureHistoryRecordLimit", clampHistoryLimit(e.currentTarget.value, DEFAULT_HISTORY_RECORD_LIMIT))}
            />
          </label>
          <label className={FIELD_CLASS}>
            <span>Raw titles per app</span>
            <input
              className={INPUT_CLASS}
              type="number"
              min={MIN_HISTORY_LIMIT}
              max={MAX_HISTORY_LIMIT}
              value={historyTitleLimit}
              onChange={(e) => update("captureHistoryTitleLimit", clampHistoryLimit(e.currentTarget.value, DEFAULT_HISTORY_TITLE_LIMIT))}
            />
          </label>
        </div>

        <div className="history-record-grid">
          <div className="history-record-panel">
            <div className="history-record-head">
              <strong>Apps</strong>
              <span>{baseState.appHistory.length} / {historyRecordLimit}</span>
            </div>
            {baseState.appHistory.length === 0 ? (
              <div className="empty-state compact-empty">No app records yet.</div>
            ) : (
              <div className="history-record-list">
                {baseState.appHistory.map((entry) => {
                  const rawTitles = appHistoryRawTitles(entry).slice(0, historyTitleLimit);
                  return (
                    <article key={`${entry.processName}-${entry.updatedAt ?? ""}`} className="history-record-item">
                      <strong>{entry.processName}</strong>
                      <span>{appHistoryDisplayTitle(entry)}</span>
                      {rawTitles.length > 0 ? (
                        <div className="history-title-list">
                          <small>Raw titles</small>
                          {rawTitles.map((title) => (
                            <code key={title}>{title}</code>
                          ))}
                        </div>
                      ) : null}
                      <small>{formatDate(entry.updatedAt)}</small>
                    </article>
                  );
                })}
              </div>
            )}
          </div>
          <div className="history-record-panel">
            <div className="history-record-head">
              <strong>Play sources</strong>
              <span>{baseState.playSourceHistory.length} / {historyRecordLimit}</span>
            </div>
            {baseState.playSourceHistory.length === 0 ? (
              <div className="empty-state compact-empty">No play-source records yet.</div>
            ) : (
              <div className="history-record-list">
                {baseState.playSourceHistory.map((entry) => (
                  <article key={`${entry.source}-${entry.updatedAt ?? ""}`} className="history-record-item history-source-item">
                    <span className="history-source-label">{entry.source}</span>
                    <strong className="history-source-title">{playSourceHistoryDisplayTitle(entry)}</strong>
                    <span className="history-source-meta">{playSourceHistoryMeta(entry)}</span>
                    <small className="history-record-time">{formatDate(entry.updatedAt)}</small>
                  </article>
                ))}
              </div>
            )}
          </div>
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
                  <span>
                    Details:{" "}
                    {option.details}
                  </span>
                  <span>
                    State / Summary: {option.state}
                  </span>
                </div>
              </div>
            </label>
          ))}
        </div>
        <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
          <div className="list-editor-summary">
            <div className="list-editor-copy">
              <strong className="block font-semibold">Compact status</strong>
              <p>Controls Discord&apos;s compact member-list text only. This setting is saved separately for the current mode.</p>
            </div>
            <span className="badge badge-soft">{activeDiscordModeName} mode</span>
          </div>
          <div className="field-grid">
            <label className={FIELD_CLASS}>
              <span>Compact status</span>
              <select
                className={SELECT_CLASS}
                value={activeDiscordStatusDisplay}
                onChange={(e) => updateDiscordModeSettings({ statusDisplay: e.currentTarget.value as DiscordStatusDisplay })}
              >
                {DISCORD_STATUS_DISPLAY_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
        </div>
        <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
          <div className="list-editor-summary">
            <div className="list-editor-copy">
              <strong className="block font-semibold">Custom application name</strong>
              <p>This setting is saved separately for the current mode. Smart and App still keep their fixed app-first output while an app is active.</p>
            </div>
            <span className="badge badge-soft">{activeDiscordModeName} mode</span>
          </div>
          <div className="field-grid">
            <label className={FIELD_CLASS}>
              <span>Application name source</span>
              <select
                className={SELECT_CLASS}
                value={activeDiscordAppNameMode}
                onChange={(e) => updateDiscordModeSettings({ appNameMode: e.currentTarget.value as DiscordAppNameMode })}
              >
                {DISCORD_APP_NAME_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            {customAppNameEnabled ? (
              <label className={FIELD_SPAN_CLASS}>
                <span>Custom application text</span>
                <input
                  className={INPUT_CLASS}
                  value={activeDiscordCustomAppName}
                  onChange={(e) => updateDiscordModeSettings({ customAppName: e.currentTarget.value })}
                  placeholder="Your custom application name"
                />
              </label>
            ) : null}
          </div>
        </div>
        {customDiscordMode ? (
          <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
            <div className="list-editor-summary">
              <div className="list-editor-copy">
                <strong className="block font-semibold">Custom mode</strong>
                <p>Choose the activity label and pick each Discord line from a short list of built-in options.</p>
              </div>
              <div className="card-actions gap-2">
                <span className="badge badge-soft">{discordActivityTypeText(config.discordActivityType)}</span>
                <DiscordOptionHelp idPrefix="custom-mode-help" />
              </div>
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
                <span>Line 2</span>
                <select
                  className={SELECT_CLASS}
                  value={resolveDiscordLineChoice(config.discordDetailsFormat, discordDetailsForceCustomChoice)}
                  onChange={(e) => {
                    const value = e.currentTarget.value;
                    setDiscordDetailsForceCustomChoice(value === DISCORD_CUSTOM_LINE_CUSTOM_VALUE);
                    update("discordDetailsFormat", nextDiscordLineValue(config.discordDetailsFormat, value));
                  }}
                >
                  {DISCORD_CUSTOM_LINE_OPTIONS.map((option) => (
                    <option key={`details-${option.value || "hidden"}`} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
              {resolveDiscordLineChoice(config.discordDetailsFormat, discordDetailsForceCustomChoice) === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? (
                <label className={FIELD_SPAN_CLASS}>
                  <span>Line 2 custom text</span>
                  <input
                    className={INPUT_CLASS}
                    value={discordLineCustomTextValue(config.discordDetailsFormat)}
                    onChange={(e) => {
                      setDiscordDetailsForceCustomChoice(true);
                      update(
                        "discordDetailsFormat",
                        e.currentTarget.value.trim() || DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
                      );
                    }}
                    placeholder="Coding in {app}"
                  />
                  <DiscordTemplateTokenRow
                    onInsert={(token) => {
                      setDiscordDetailsForceCustomChoice(true);
                      update(
                        "discordDetailsFormat",
                        appendDiscordTemplateToken(discordLineCustomTextValue(config.discordDetailsFormat), token),
                      );
                    }}
                  />
                </label>
              ) : null}
              <label className={FIELD_SPAN_CLASS}>
                <span>Line 3</span>
                <select
                  className={SELECT_CLASS}
                  value={resolveDiscordLineChoice(config.discordStateFormat, discordStateForceCustomChoice)}
                  onChange={(e) => {
                    const value = e.currentTarget.value;
                    setDiscordStateForceCustomChoice(value === DISCORD_CUSTOM_LINE_CUSTOM_VALUE);
                    update("discordStateFormat", nextDiscordLineValue(config.discordStateFormat, value));
                  }}
                >
                  {DISCORD_CUSTOM_LINE_OPTIONS.map((option) => (
                    <option key={`state-${option.value || "hidden"}`} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
              {resolveDiscordLineChoice(config.discordStateFormat, discordStateForceCustomChoice) === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? (
                <label className={FIELD_SPAN_CLASS}>
                  <span>Line 3 custom text</span>
                  <input
                    className={INPUT_CLASS}
                    value={discordLineCustomTextValue(config.discordStateFormat)}
                    onChange={(e) => {
                      setDiscordStateForceCustomChoice(true);
                      update(
                        "discordStateFormat",
                        e.currentTarget.value.trim() || DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
                      );
                    }}
                    placeholder="With {artist}"
                  />
                  <DiscordTemplateTokenRow
                    onInsert={(token) => {
                      setDiscordStateForceCustomChoice(true);
                      update(
                        "discordStateFormat",
                        appendDiscordTemplateToken(discordLineCustomTextValue(config.discordStateFormat), token),
                      );
                    }}
                  />
                </label>
              ) : null}
            </div>
            <div className="rounded-box border border-base-300 bg-base-100 p-4 space-y-4">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <strong className="block font-semibold">Custom add-ons</strong>
                  <p className="mt-1 text-sm text-base-content/70">URL buttons stay visible here. Party and secrets live under Advanced.</p>
                </div>
                <span className="badge badge-soft">{config.discordCustomButtons.length} / 2 buttons</span>
              </div>
              <div className="space-y-3">
                {config.discordCustomButtons.map((button, index) => (
                  <div key={`discord-button-${index}`} className="rounded-box border border-base-300 bg-base-200/50 p-3 space-y-3">
                    <div className="field-grid">
                      <label className={FIELD_CLASS}>
                        <span>Button label</span>
                        <input
                          className={INPUT_CLASS}
                          value={button.label}
                          onChange={(e) => patchDiscordButtonAt(index, (current) => ({ ...current, label: e.currentTarget.value }))}
                          placeholder="Open website"
                        />
                      </label>
                      <label className={FIELD_CLASS}>
                        <span>Button URL</span>
                        <input
                          className={INPUT_CLASS}
                          value={button.url}
                          onChange={(e) => patchDiscordButtonAt(index, (current) => ({ ...current, url: e.currentTarget.value }))}
                          placeholder="https://example.com or myapp://open"
                        />
                      </label>
                    </div>
                    <div className="card-actions justify-end">
                      <button
                        className={DANGER_BUTTON_CLASS}
                        type="button"
                        onClick={() => setConfig((current) => ({
                          ...current,
                          discordCustomButtons: current.discordCustomButtons.filter((_, itemIndex) => itemIndex !== index),
                        }))}
                      >
                        Remove button
                      </button>
                    </div>
                  </div>
                ))}
                {config.discordCustomButtons.length < 2 ? (
                  <button
                    className={BUTTON_CLASS}
                    type="button"
                    onClick={() => setConfig((current) => ({
                      ...current,
                      discordCustomButtons: [...current.discordCustomButtons, createDiscordButton()],
                    }))}
                  >
                    Add button
                  </button>
                ) : null}
              </div>
              <details className="discord-advanced-panel rounded-box border border-base-300 bg-base-200/45 p-4">
                <summary className="discord-advanced-summary flex cursor-pointer list-none items-center justify-between gap-3">
                  <div>
                    <strong className="block font-semibold">Advanced</strong>
                    <p className="mt-1 text-sm text-base-content/70">Party metadata and Discord social secrets.</p>
                  </div>
                  <div className="discord-advanced-summary-meta">
                    {customAdvancedAddonsConfigured ? <span className="badge badge-soft">Configured</span> : null}
                    <span className="discord-advanced-summary-hint" aria-hidden="true">
                      <span className="discord-advanced-summary-hint-closed">Expand</span>
                      <span className="discord-advanced-summary-hint-open">Collapse</span>
                      <span className="discord-advanced-summary-caret">v</span>
                    </span>
                  </div>
                </summary>
                <div className="field-grid mt-4">
                  <label className={FIELD_CLASS}>
                    <span>Party ID</span>
                    <input
                      className={INPUT_CLASS}
                      value={config.discordCustomPartyId}
                      onChange={(e) => update("discordCustomPartyId", e.currentTarget.value)}
                      placeholder="party-123"
                    />
                  </label>
                  <label className={FIELD_CLASS}>
                    <span>Party current size</span>
                    <input
                      className={INPUT_CLASS}
                      type="number"
                      min={1}
                      value={config.discordCustomPartySizeCurrent ?? ""}
                      onChange={(e) => update("discordCustomPartySizeCurrent", normalizePositiveNumberInput(e.currentTarget.value))}
                      placeholder="2"
                    />
                  </label>
                  <label className={FIELD_CLASS}>
                    <span>Party max size</span>
                    <input
                      className={INPUT_CLASS}
                      type="number"
                      min={1}
                      value={config.discordCustomPartySizeMax ?? ""}
                      onChange={(e) => update("discordCustomPartySizeMax", normalizePositiveNumberInput(e.currentTarget.value))}
                      placeholder="3"
                    />
                  </label>
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Join secret</span>
                    <input
                      className={INPUT_CLASS}
                      value={config.discordCustomJoinSecret}
                      onChange={(e) => update("discordCustomJoinSecret", e.currentTarget.value)}
                      placeholder="join-secret"
                    />
                  </label>
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Spectate secret</span>
                    <input
                      className={INPUT_CLASS}
                      value={config.discordCustomSpectateSecret}
                      onChange={(e) => update("discordCustomSpectateSecret", e.currentTarget.value)}
                      placeholder="spectate-secret"
                    />
                  </label>
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Match secret</span>
                    <input
                      className={INPUT_CLASS}
                      value={config.discordCustomMatchSecret}
                      onChange={(e) => update("discordCustomMatchSecret", e.currentTarget.value)}
                      placeholder="match-secret"
                    />
                  </label>
                </div>
              </details>
            </div>
            <div className="rounded-box border border-base-300 bg-base-100 p-4">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <strong className="block font-semibold">Custom presets</strong>
                  <p className="mt-1 text-sm text-base-content/70">
                    Save ready-to-use Custom profiles for one-click selection and import.
                  </p>
                </div>
                <div className="card-actions gap-2">
                  <span className="badge badge-soft">{config.discordCustomPresets.length} presets</span>
                  <button className={PRIMARY_BUTTON_CLASS} type="button" onClick={saveCurrentCustomSettingsAsPreset}>
                    Save current as preset
                  </button>
                  <button className={BUTTON_CLASS} type="button" onClick={() => setCustomRulesDialogOpen(true)}>
                    Open presets
                  </button>
                </div>
              </div>
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
                <span>Show the current foreground app on line 2 in Smart mode when an app is active.</span>
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
              <strong>Report paused media</strong>
              <span>Keep the latest media visible after playback pauses or stops, with a frozen Discord song timer.</span>
            </div>
            <input
              className="toggle toggle-primary"
              type="checkbox"
              checked={config.reportStoppedMedia}
              onChange={(e) => update("reportStoppedMedia", e.currentTarget.checked)}
            />
          </label>
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
                <p>App icons are uploaded as PNG with transparency preserved, while music artwork is uploaded as JPEG. ActivityPing normalizes both to a 256px target and keeps each uploaded image within a 30 KB budget before sending it to your uploader service.</p>
              </div>
              <span className="badge badge-soft">Uploader service</span>
            </div>
            {artworkPublishingMissing ? (
              <div className="alert alert-warning alert-soft text-sm">
                <span>Artwork publishing needs an uploader service URL before these artwork settings can be saved.</span>
              </div>
            ) : null}
            <div className="field-grid">
              <label className={FIELD_SPAN_CLASS}>
                <span>Uploader service URL</span>
                <input
                  className={INPUT_CLASS}
                  aria-invalid={artworkPublishingMissing}
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

      <motion.section className={`${PANEL_CLASS} runtime-card runtime-main-card`} {...CARD_MOTION} transition={MOTION_TRANSITION}>
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
            disabled={busy.startRuntime || busy.restartRuntime || runtimeRunning || !runtimeReady}
            onClick={() => runAction("startRuntime", startRuntimeSession)}
          >
            {busy.startRuntime ? "Starting..." : runtimeReady ? "Start runtime" : "RPC required"}
          </button>
          <button
            className={BUTTON_CLASS}
            disabled={busy.stopRuntime || busy.restartRuntime || !runtimeRunning}
            onClick={() => runAction("stopRuntime", stopRuntimeSession)}
          >
            Stop runtime
          </button>
          <button className={BUTTON_CLASS} disabled={busy.refreshRuntime || busy.restartRuntime || !runtimeReady} onClick={() => runAction("refreshRuntime", async () => { await refreshReporter(); await refreshDiscord(); notify("info", "Runtime refreshed", "The live status panels were updated."); })}>
            {busy.refreshRuntime ? "Refreshing..." : runtimeReady ? "Refresh" : "RPC required"}
          </button>
        </div>
      </motion.section>

      <motion.section className={`${PANEL_CLASS} runtime-card runtime-log-card`} {...CARD_MOTION} transition={{ ...MOTION_TRANSITION, delay: 0.03 }}>
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
            visibleRuntimeLogs.map((entry) => (
              <motion.article
                key={entry.id}
                layout
                {...LOG_MOTION}
                transition={MOTION_TRANSITION}
                className={`${logEntryClass(entry.level)} ${hasJsonPayload(entry.payload) ? "log-entry-clickable" : ""}`}
                role={hasJsonPayload(entry.payload) ? "button" : undefined}
                tabIndex={hasJsonPayload(entry.payload) ? 0 : undefined}
                onClick={hasJsonPayload(entry.payload) ? () => openLogPayloadJson(entry) : undefined}
                onKeyDown={
                  hasJsonPayload(entry.payload)
                    ? (event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          event.preventDefault();
                          openLogPayloadJson(entry);
                        }
                      }
                    : undefined
                }
                title={hasJsonPayload(entry.payload) ? "Open reported JSON" : undefined}
              >
                <div className="card-body p-3">
                  <div className="log-entry-head">
                    <strong>{entry.title}</strong>
                    <div className="log-entry-actions">
                      <span>{formatDate(entry.timestamp)}</span>
                    </div>
                  </div>
                  <p>{entry.detail}</p>
                </div>
              </motion.article>
            ))
          )}
        </div>
        {runtimeLogs.length > 0 ? (
          <div className="pagination-row runtime-log-pagination">
            <span className="pagination-copy">
              {runtimeLogPageStart}-{runtimeLogPageEnd} of {runtimeLogs.length}
            </span>
            <div className="join">
              <button
                className="btn btn-outline btn-xs join-item"
                type="button"
                disabled={safeRuntimeLogPage <= 0}
                onClick={() => setRuntimeLogPage((current) => clampPage(current - 1, runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE))}
              >
                Prev
              </button>
              <span className="btn btn-ghost btn-xs join-item no-animation">
                Page {safeRuntimeLogPage + 1} / {runtimeLogPageCount}
              </span>
              <button
                className="btn btn-outline btn-xs join-item"
                type="button"
                disabled={safeRuntimeLogPage >= runtimeLogPageCount - 1}
                onClick={() => setRuntimeLogPage((current) => clampPage(current + 1, runtimeLogs.length, RUNTIME_LOG_PAGE_SIZE))}
              >
                Next
              </button>
            </div>
          </div>
        ) : null}
      </motion.section>

      <motion.section className={`${PANEL_CLASS} runtime-card runtime-debug-card`} {...CARD_MOTION} transition={{ ...MOTION_TRANSITION, delay: 0.06 }}>
        <div className={PANEL_HEAD_CLASS}>
          <div>
            <p className="eyebrow">Debug</p>
            <h3>Discord payload JSON</h3>
          </div>
          <button className={BUTTON_CLASS} type="button" onClick={openDiscordPayloadJson}>
            Open payload JSON
          </button>
        </div>
        {discordDebugPayload ? (
          <div className="empty-state">
            Open payload JSON to inspect the current data being pushed into Discord.
          </div>
        ) : (
          <div className="empty-state">
            No Discord payload has been published yet. Start runtime and wait for a captured activity to pass the current rules.
          </div>
        )}
      </motion.section>
    </div>
  );

  const aboutView = (
    <motion.section className="about-page" {...CARD_MOTION} transition={MOTION_TRANSITION}>
      <div className="about-identity">
        <img className="about-icon" src={appIcon} alt="ActivityPing icon" />
        <div className="about-copy">
          <p className="eyebrow">About</p>
          <h3>ActivityPing</h3>
          <p>Desktop activity monitor for Discord Rich Presence and webhook reporting.</p>
        </div>
      </div>

      <div className="about-repo">
        <span className="eyebrow">Repository</span>
        <strong>MoYoez/ActivityPing</strong>
        <span>{GITHUB_URL}</span>
      </div>

      <div className="about-actions">
        <button
          className={PRIMARY_BUTTON_CLASS}
          type="button"
          disabled={busy.openGithub}
          onClick={() =>
            runAction("openGithub", async () => {
              await openUrl(GITHUB_URL);
            })
          }
        >
          {busy.openGithub ? "Opening..." : "Open GitHub"}
        </button>
      </div>
    </motion.section>
  );

  function renderSection() {
    switch (activeSection) {
      case "runtime":
        return runtimeView;
      case "settings":
        return settingsView;
      case "about":
        return aboutView;
      default:
        return runtimeView;
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
          <div className="content-body">
            <AnimatePresence mode="wait">
              <motion.div
                key={activeSection}
                className="section-motion"
                {...VIEW_MOTION}
                transition={MOTION_TRANSITION}
              >
                {renderSection()}
              </motion.div>
            </AnimatePresence>
          </div>
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

      {customRulesDialogOpen ? (
        <section
          className="modal modal-open"
          onClick={() => {
            setActiveCustomPresetIndex(null);
            setCustomRulesDialogOpen(false);
          }}
        >
          <div className="modal-box w-11/12 max-w-5xl p-0" role="dialog" aria-modal="true" aria-labelledby="custom-presets-dialog-title" onClick={(event) => event.stopPropagation()}>
            <div className="card-body">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <p className="eyebrow">Custom</p>
                  <h3 id="custom-presets-dialog-title" className="card-title">Custom presets</h3>
                  <p>Browse saved Custom profiles page by page. Click one to open its editor.</p>
                </div>
                <div className="card-actions gap-2">
                  <button
                    className={PRIMARY_BUTTON_CLASS}
                    type="button"
                    onClick={saveCurrentCustomSettingsAsPreset}
                  >
                    Save current as preset
                  </button>
                  <button
                    className={BUTTON_CLASS}
                    type="button"
                    onClick={() => {
                      const nextIndex = config.discordCustomPresets.length;
                      setConfig((current) => ({
                        ...current,
                        discordCustomPresets: [...current.discordCustomPresets, createDiscordCustomPreset()],
                      }));
                      setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
                      setActiveCustomPresetIndex(nextIndex);
                    }}
                  >
                    Add preset
                  </button>
                  <button
                    className={BUTTON_CLASS}
                    type="button"
                    onClick={() => {
                      setActiveCustomPresetIndex(null);
                      setCustomRulesDialogOpen(false);
                    }}
                  >
                    Close
                  </button>
                </div>
              </div>
              <div className="rule-modal-body">
                {config.discordCustomPresets.length === 0 ? (
                  <div className="empty-state">No Custom presets yet.</div>
                ) : (
                  <>
                    <div className="grid gap-2">
                      {pagedCustomPresets.map((rule, offset) => {
                        const index = customPresetPageStart + offset;
                        return (
                          <article
                            key={`custom-preset-${index}`}
                            className={`${SUBRULE_CARD_CLASS} log-entry-clickable p-4`}
                            role="button"
                            tabIndex={0}
                            onClick={() => setActiveCustomPresetIndex(index)}
                            onKeyDown={(event) => {
                              if (event.key === "Enter" || event.key === " ") {
                                event.preventDefault();
                                setActiveCustomPresetIndex(index);
                              }
                            }}
                          >
                        <div className={PANEL_HEAD_CLASS}>
                          <div>
                            <strong className="block font-semibold">
                              {rule.name.trim() || `Custom preset ${index + 1}`}
                            </strong>
                            <p className="mt-1 text-sm text-base-content/70">{summarizeDiscordCustomPreset(rule)}</p>
                          </div>
                          <div className="card-actions gap-2">
                            <span className="badge badge-soft">Open</span>
                            <button
                              className={BUTTON_CLASS}
                              type="button"
                              disabled={index <= 0}
                              onClick={(event) => {
                                event.stopPropagation();
                                const nextIndex = index - 1;
                                setConfig((current) => ({
                                  ...current,
                                  discordCustomPresets: moveItem(current.discordCustomPresets, index, nextIndex),
                                }));
                                setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
                              }}
                            >
                              Up
                            </button>
                            <button
                              className={BUTTON_CLASS}
                              type="button"
                              disabled={index >= config.discordCustomPresets.length - 1}
                              onClick={(event) => {
                                event.stopPropagation();
                                const nextIndex = index + 1;
                                setConfig((current) => ({
                                  ...current,
                                  discordCustomPresets: moveItem(current.discordCustomPresets, index, nextIndex),
                                }));
                                setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
                              }}
                            >
                              Down
                            </button>
                            <button
                              className={DANGER_BUTTON_CLASS}
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                setConfig((current) => ({
                                  ...current,
                                  discordCustomPresets: current.discordCustomPresets.filter((_, itemIndex) => itemIndex !== index),
                                }));
                                if (activeCustomPresetIndex === index) {
                                  setActiveCustomPresetIndex(null);
                                }
                              }}
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      </article>
                        );
                      })}
                    </div>
                    {customPresetTotalPages > 1 ? (
                      <div className="pagination-row">
                        <span className="pagination-copy">
                          {customPresetPageStart + 1}-{Math.min(customPresetPageStart + CUSTOM_PRESET_PAGE_SIZE, config.discordCustomPresets.length)} of {config.discordCustomPresets.length}
                        </span>
                        <div className="join">
                          <button
                            className="btn btn-outline btn-xs join-item"
                            type="button"
                            disabled={safeCustomPresetPage <= 0}
                            onClick={() => setCustomPresetPage((current) => clampPage(current - 1, config.discordCustomPresets.length, CUSTOM_PRESET_PAGE_SIZE))}
                          >
                            Prev
                          </button>
                          <span className="btn btn-ghost btn-xs join-item no-animation">
                            Page {safeCustomPresetPage + 1} / {customPresetTotalPages}
                          </span>
                          <button
                            className="btn btn-outline btn-xs join-item"
                            type="button"
                            disabled={safeCustomPresetPage >= customPresetTotalPages - 1}
                            onClick={() => setCustomPresetPage((current) => clampPage(current + 1, config.discordCustomPresets.length, CUSTOM_PRESET_PAGE_SIZE))}
                          >
                            Next
                          </button>
                        </div>
                      </div>
                    ) : null}
                  </>
                )}
              </div>
            </div>
          </div>
        </section>
      ) : null}

      {activeCustomPreset && activeCustomPresetIndex !== null ? (
        <section className="modal modal-open" onClick={() => setActiveCustomPresetIndex(null)}>
          <div className="modal-box w-11/12 max-w-4xl p-0" role="dialog" aria-modal="true" aria-labelledby="custom-preset-editor-title" onClick={(event) => event.stopPropagation()}>
            <div className="card-body">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <p className="eyebrow">Custom preset</p>
                  <h3 id="custom-preset-editor-title" className="card-title">
                    {activeCustomPreset.name.trim() || `Custom preset ${activeCustomPresetIndex + 1}`}
                  </h3>
                  <p>Edit the preset fields that will be imported into Custom mode output.</p>
                </div>
                <div className="card-actions gap-2">
                  <DiscordOptionHelp idPrefix="custom-preset-help" includeSmartModeNote={false} />
                  <button
                    className={PRIMARY_BUTTON_CLASS}
                    type="button"
                    onClick={() => {
                      applyDiscordCustomPreset(activeCustomPreset);
                      setActiveCustomPresetIndex(null);
                      setCustomRulesDialogOpen(false);
                    }}
                  >
                    Use preset
                  </button>
                  <button className={BUTTON_CLASS} type="button" onClick={() => setActiveCustomPresetIndex(null)}>
                    Close
                  </button>
                </div>
              </div>

              <div className="field-grid compact-fields">
                <label className={FIELD_CLASS}>
                  <span>Name</span>
                  <input
                    className={INPUT_CLASS}
                    value={activeCustomPreset.name}
                    onChange={(e) => {
                      const value = e.currentTarget.value;
                      patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, name: value }));
                    }}
                    placeholder="Work profile"
                  />
                </label>
                <label className={FIELD_CLASS}>
                  <span>Activity label</span>
                  <select
                    className={SELECT_CLASS}
                    value={activeCustomPreset.activityType}
                    onChange={(e) => {
                      const activityType = e.currentTarget.value as DiscordActivityType;
                      patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, activityType }));
                    }}
                  >
                    {DISCORD_ACTIVITY_TYPE_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label className={FIELD_CLASS}>
                  <span>Compact status</span>
                  <select
                    className={SELECT_CLASS}
                    value={activeCustomPreset.statusDisplay}
                    onChange={(e) => {
                      const statusDisplay = e.currentTarget.value as DiscordStatusDisplay;
                      patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, statusDisplay }));
                    }}
                  >
                    {DISCORD_STATUS_DISPLAY_OPTIONS.map((option) => (
                      <option key={`preset-status-${option.value}`} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label className={FIELD_CLASS}>
                  <span>Application name source</span>
                  <select
                    className={SELECT_CLASS}
                    value={activeCustomPreset.appNameMode}
                    onChange={(e) => {
                      const appNameMode = e.currentTarget.value as DiscordAppNameMode;
                      patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, appNameMode }));
                    }}
                  >
                    {DISCORD_APP_NAME_OPTIONS.map((option) => (
                      <option key={`preset-app-name-${option.value}`} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                {activeCustomPreset.appNameMode === "custom" ? (
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Custom application text</span>
                    <input
                      className={INPUT_CLASS}
                      value={activeCustomPreset.customAppName}
                      onChange={(e) => {
                        const customAppName = e.currentTarget.value;
                        patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, customAppName }));
                      }}
                      placeholder="Your custom application name"
                    />
                  </label>
                ) : null}
                <label className={FIELD_SPAN_CLASS}>
                  <span>Preset Line 2</span>
                  <select
                    className={SELECT_CLASS}
                    value={resolveDiscordLineChoice(activeCustomPreset.detailsFormat ?? "", presetDetailsForceCustomChoice)}
                    onChange={(e) => {
                      const value = e.currentTarget.value;
                      setPresetDetailsForceCustomChoice(value === DISCORD_CUSTOM_LINE_CUSTOM_VALUE);
                      patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({
                        ...item,
                        detailsFormat: nextDiscordLineValue(item.detailsFormat ?? "", value),
                      }));
                    }}
                  >
                    {DISCORD_CUSTOM_LINE_OPTIONS.map((option) => (
                      <option key={`preset-details-${option.value || "hidden"}`} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                {resolveDiscordLineChoice(activeCustomPreset.detailsFormat ?? "", presetDetailsForceCustomChoice) === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? (
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Preset Line 2 custom text</span>
                    <input
                      className={INPUT_CLASS}
                      value={discordLineCustomTextValue(activeCustomPreset.detailsFormat ?? "")}
                      onChange={(e) => {
                        setPresetDetailsForceCustomChoice(true);
                        const detailsFormat = e.currentTarget.value.trim() || DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
                        patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, detailsFormat }));
                      }}
                      placeholder="Coding in {app}"
                    />
                    <DiscordTemplateTokenRow
                      onInsert={(token) => {
                        setPresetDetailsForceCustomChoice(true);
                        patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({
                          ...item,
                          detailsFormat: appendDiscordTemplateToken(
                            discordLineCustomTextValue(item.detailsFormat ?? ""),
                            token,
                          ),
                        }));
                      }}
                    />
                  </label>
                ) : null}
                <label className={FIELD_SPAN_CLASS}>
                  <span>Preset Line 3</span>
                  <select
                    className={SELECT_CLASS}
                    value={resolveDiscordLineChoice(activeCustomPreset.stateFormat ?? "", presetStateForceCustomChoice)}
                    onChange={(e) => {
                      const value = e.currentTarget.value;
                      setPresetStateForceCustomChoice(value === DISCORD_CUSTOM_LINE_CUSTOM_VALUE);
                      patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({
                        ...item,
                        stateFormat: nextDiscordLineValue(item.stateFormat ?? "", value),
                      }));
                    }}
                  >
                    {DISCORD_CUSTOM_LINE_OPTIONS.map((option) => (
                      <option key={`preset-state-${option.value || "hidden"}`} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                {resolveDiscordLineChoice(activeCustomPreset.stateFormat ?? "", presetStateForceCustomChoice) === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? (
                  <label className={FIELD_SPAN_CLASS}>
                    <span>Preset Line 3 custom text</span>
                    <input
                      className={INPUT_CLASS}
                      value={discordLineCustomTextValue(activeCustomPreset.stateFormat ?? "")}
                      onChange={(e) => {
                        setPresetStateForceCustomChoice(true);
                        const stateFormat = e.currentTarget.value.trim() || DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
                        patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({ ...item, stateFormat }));
                      }}
                      placeholder="With {artist}"
                    />
                    <DiscordTemplateTokenRow
                      onInsert={(token) => {
                        setPresetStateForceCustomChoice(true);
                        patchDiscordCustomPresetAt(activeCustomPresetIndex, (item) => ({
                          ...item,
                          stateFormat: appendDiscordTemplateToken(
                            discordLineCustomTextValue(item.stateFormat ?? ""),
                            token,
                          ),
                        }));
                      }}
                    />
                  </label>
                ) : null}
              </div>
              <div className="rounded-box border border-base-300 bg-base-100 p-4 text-sm text-base-content/70">
                <p>
                  <strong>Saved add-ons:</strong>{" "}
                  {[
                    `${activeCustomPreset.buttons.length} button${activeCustomPreset.buttons.length === 1 ? "" : "s"}`,
                    activeCustomPreset.partyId.trim() || activeCustomPreset.partySizeCurrent || activeCustomPreset.partySizeMax ? "party" : null,
                    activeCustomPreset.joinSecret.trim() || activeCustomPreset.spectateSecret.trim() || activeCustomPreset.matchSecret.trim() ? "secrets" : null,
                  ].filter(Boolean).join(" · ") || "none"}
                </p>
              </div>
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

      {jsonViewer ? (
        <section className="modal modal-open" onClick={() => setJsonViewer(null)}>
          <div
            className="modal-box w-11/12 max-w-4xl p-0"
            role="dialog"
            aria-modal="true"
            aria-labelledby="json-viewer-dialog-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="card-body">
              <div className={PANEL_HEAD_CLASS}>
                <div>
                  <p className="eyebrow">{jsonViewer.eyebrow}</p>
                  <h3 id="json-viewer-dialog-title" className="card-title">{jsonViewer.title}</h3>
                  <p>{jsonViewer.description}</p>
                </div>
                <button className={BUTTON_CLASS} type="button" onClick={() => setJsonViewer(null)}>
                  Close
                </button>
              </div>
              {jsonViewer.value ? (
                <pre className="debug-json">{jsonViewerJson}</pre>
              ) : (
                <div className="empty-state">{jsonViewer.emptyText}</div>
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
              <div className="save-reminder-actions">
                <button
                  className={PRIMARY_BUTTON_CLASS}
                  type="button"
                  disabled={busy.restartRuntime || busy.startRuntime || busy.stopRuntime}
                  onClick={() => runAction("restartRuntime", restartRuntimeSession)}
                >
                  {busy.restartRuntime ? "Restarting..." : "Restart now"}
                </button>
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
