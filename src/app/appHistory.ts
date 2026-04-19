import type { AppHistoryEntry, PlaySourceHistoryEntry, ReporterActivity } from "../types";

import {
  DEFAULT_HISTORY_RECORD_LIMIT,
  DEFAULT_HISTORY_TITLE_LIMIT,
  MAX_HISTORY_LIMIT,
  MAX_HISTORY_LIMIT as APP_HISTORY_LIMIT,
  MIN_HISTORY_LIMIT,
} from "./appConstants";

export function compactOptionalText(value?: string | null) {
  const trimmed = value?.trim() ?? "";
  return trimmed || null;
}

export function clampHistoryLimit(value: unknown, fallback: number) {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(MAX_HISTORY_LIMIT, Math.max(MIN_HISTORY_LIMIT, parsed));
}

export function normalizeTitleHistory(values: unknown, fallback?: string | null, limit = DEFAULT_HISTORY_TITLE_LIMIT) {
  const result: string[] = [];
  const seen = new Set<string>();
  const candidates = [...(Array.isArray(values) ? values : []), fallback];

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

export function normalizeAppHistory(
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

export function normalizePlaySourceHistory(
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

export function uniqueHistoryValues(values: string[]) {
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

export function shouldCaptureHistoryActivity(activity?: ReporterActivity | null) {
  const processName = activity?.processName?.trim().toLowerCase() ?? "";
  return Boolean(processName && processName !== "activityping.exe");
}

export function mergeAppHistory(
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

export function mergePlaySourceHistory(
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

export function appHistoryRawTitles(entry: AppHistoryEntry) {
  return normalizeTitleHistory(entry.processTitles, entry.processTitle, APP_HISTORY_LIMIT);
}
