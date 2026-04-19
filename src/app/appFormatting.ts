import { normalizeClientConfig } from "../lib/rules";
import type { ClientConfig, DiscordActivityType, DiscordReportMode, RealtimeReporterSnapshot } from "../types";

import { MAX_RUNTIME_LOGS } from "./appConstants";

export function alertClass(tone: "info" | "success" | "warn" | "error") {
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

export function logEntryClass(level: string) {
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

export function limitReporterSnapshotLogs(snapshot: RealtimeReporterSnapshot) {
  if (snapshot.logs.length <= MAX_RUNTIME_LOGS) {
    return snapshot;
  }
  return { ...snapshot, logs: snapshot.logs.slice(0, MAX_RUNTIME_LOGS) };
}

export function clampRuleIndex(index: number, total: number) {
  if (total <= 0) {
    return -1;
  }
  return Math.min(Math.max(index, 0), total - 1);
}

export function pageCount(total: number, pageSize: number) {
  return Math.max(1, Math.ceil(total / pageSize));
}

export function clampPage(page: number, total: number, pageSize: number) {
  return Math.min(Math.max(page, 0), pageCount(total, pageSize) - 1);
}

export function pageForIndex(index: number, pageSize: number) {
  return index < 0 ? 0 : Math.floor(index / pageSize);
}

export function configSignature(value: ClientConfig) {
  return JSON.stringify(normalizeClientConfig(value));
}

export function formatDate(value?: string | null) {
  if (!value) {
    return "Not available";
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("en-US");
}

export function formatDurationClock(valueMs?: number | null) {
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

export function mediaTimelineText(positionMs?: number | null, durationMs?: number | null) {
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

export function activityText(snapshot: RealtimeReporterSnapshot) {
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

export function activityMeta(snapshot: RealtimeReporterSnapshot) {
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

export function captureModeText(config: ClientConfig) {
  const parts = [];
  if (config.reportForegroundApp) parts.push("App");
  if (config.reportWindowTitle) parts.push("Title");
  if (config.reportMedia) parts.push("Media");
  if (config.reportStoppedMedia) parts.push("Paused media");
  if (config.reportPlaySource) parts.push("Source");
  return parts.length > 0 ? parts.join(" + ") : "Capture disabled";
}

export function discordReportModeText(config: ClientConfig) {
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

export function discordReportModeName(mode: DiscordReportMode) {
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

export function discordActivityTypeText(value: DiscordActivityType) {
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

export function localWorkingModeText(config: ClientConfig) {
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

export function sameJsonValue(left: unknown, right: unknown) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function moveItem<T>(items: T[], from: number, to: number) {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) {
    return items;
  }
  const next = [...items];
  const [picked] = next.splice(from, 1);
  next.splice(to, 0, picked);
  return next;
}
