import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppTitleRuleMode,
  DiscordActivityType,
  DiscordAppNameMode,
  DiscordCustomPreset,
  DiscordReportMode,
  DiscordRichPresenceButtonConfig,
  DiscordSmartArtworkPreference,
  DiscordStatusDisplay,
} from "../../types";

const MIN_HISTORY_LIMIT = 1;
const MAX_HISTORY_LIMIT = 50;
export const DISCORD_CUSTOM_LINE_CUSTOM_VALUE = "__custom__";

export function normalizeHistoryLimit(value: unknown, fallback: number) {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(MAX_HISTORY_LIMIT, Math.max(MIN_HISTORY_LIMIT, parsed));
}

export function normalizeStringList(values: string[], lowercase: boolean) {
  const result: string[] = [];
  const seen = new Set<string>();

  for (const value of values) {
    const trimmed = String(value ?? "").trim();
    if (!trimmed) {
      continue;
    }
    const key = trimmed.toLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    result.push(lowercase ? key : trimmed);
  }

  return result;
}

export function normalizeTitleRule(rule: AppMessageTitleRule): AppMessageTitleRule | null {
  const mode: AppTitleRuleMode = String(rule.mode ?? "").trim().toLowerCase() === "regex" ? "regex" : "plain";
  const pattern = String(rule.pattern ?? "").trim();
  const text = String(rule.text ?? "").trim();
  const buttons = (Array.isArray(rule.buttons) ? rule.buttons : [])
    .map((button) => normalizeDiscordButton(button))
    .filter((button): button is DiscordRichPresenceButtonConfig => button !== null)
    .slice(0, 2);
  if (!pattern || !text) {
    return null;
  }
  return { mode, pattern, text, buttons };
}

export function normalizeRuleGroup(rule: AppMessageRuleGroup): AppMessageRuleGroup | null {
  const processMatch = String(rule.processMatch ?? "").trim();
  if (!processMatch) {
    return null;
  }

  const defaultText = String(rule.defaultText ?? "").trim();
  const titleRules = (Array.isArray(rule.titleRules) ? rule.titleRules : [])
    .map((item) => normalizeTitleRule(item))
    .filter((item): item is AppMessageTitleRule => item !== null);
  const buttons = (Array.isArray(rule.buttons) ? rule.buttons : [])
    .map((button) => normalizeDiscordButton(button))
    .filter((button): button is DiscordRichPresenceButtonConfig => button !== null)
    .slice(0, 2);
  const partyId = String(rule.partyId ?? "").trim();
  const partySizeCurrent = normalizePartySize(rule.partySizeCurrent);
  const partySizeMax = normalizePartySize(rule.partySizeMax);
  const joinSecret = String(rule.joinSecret ?? "").trim();
  const spectateSecret = String(rule.spectateSecret ?? "").trim();
  const matchSecret = String(rule.matchSecret ?? "").trim();

  if (
    !defaultText &&
    titleRules.length === 0 &&
    buttons.length === 0 &&
    !partyId &&
    partySizeCurrent === null &&
    partySizeMax === null &&
    !joinSecret &&
    !spectateSecret &&
    !matchSecret
  ) {
    return null;
  }

  return {
    processMatch,
    defaultText,
    titleRules,
    buttons,
    partyId,
    partySizeCurrent,
    partySizeMax,
    joinSecret,
    spectateSecret,
    matchSecret,
  };
}

export function normalizeDiscordCustomPreset(rule: Partial<DiscordCustomPreset>): DiscordCustomPreset | null {
  const name = String(rule.name ?? "").trim();
  const activityType = normalizeDiscordActivityType(rule.activityType);
  const statusDisplay = normalizeDiscordStatusDisplay(rule.statusDisplay);
  const appNameMode = normalizeDiscordAppNameMode(rule.appNameMode);
  const customAppName = String(rule.customAppName ?? "").trim();
  const detailsFormat = normalizeDiscordLineFormat(rule.detailsFormat);
  const stateFormat = normalizeDiscordLineFormat(rule.stateFormat);
  const buttons = (Array.isArray(rule.buttons) ? rule.buttons : [])
    .map((button) => normalizeDiscordButton(button))
    .filter((button): button is DiscordRichPresenceButtonConfig => button !== null)
    .slice(0, 2);
  const partyId = String(rule.partyId ?? "").trim();
  const partySizeCurrent = normalizePartySize(rule.partySizeCurrent);
  const partySizeMax = normalizePartySize(rule.partySizeMax);
  const joinSecret = String(rule.joinSecret ?? "").trim();
  const spectateSecret = String(rule.spectateSecret ?? "").trim();
  const matchSecret = String(rule.matchSecret ?? "").trim();

  if (
    !name &&
    !detailsFormat &&
    !stateFormat &&
    !customAppName &&
    buttons.length === 0 &&
    !partyId &&
    partySizeCurrent === null &&
    partySizeMax === null &&
    !joinSecret &&
    !spectateSecret &&
    !matchSecret
  ) {
    return null;
  }

  return {
    name,
    activityType,
    statusDisplay,
    appNameMode,
    customAppName,
    detailsFormat,
    stateFormat,
    buttons,
    partyId,
    partySizeCurrent,
    partySizeMax,
    joinSecret,
    spectateSecret,
    matchSecret,
  };
}

export function normalizeDiscordButton(
  button: Partial<DiscordRichPresenceButtonConfig>,
): DiscordRichPresenceButtonConfig | null {
  const label = String(button.label ?? "").trim();
  const url = String(button.url ?? "").trim();
  if (!label || !url) {
    return null;
  }
  return { label, url };
}

export function normalizePartySize(value: unknown): number | null {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }
  return parsed;
}

export function normalizeDiscordReportMode(value: unknown): DiscordReportMode {
  const mode = String(value ?? "").trim().toLowerCase();
  if (mode === "music" || mode === "app" || mode === "custom") {
    return mode;
  }
  return "mixed";
}

export function normalizeDiscordActivityType(value: unknown): DiscordActivityType {
  const mode = String(value ?? "").trim().toLowerCase();
  if (mode === "listening" || mode === "watching" || mode === "competing") {
    return mode;
  }
  return "playing";
}

export function normalizeDiscordStatusDisplay(
  value: unknown,
): DiscordStatusDisplay {
  const mode = String(value ?? "").trim().toLowerCase();
  if (mode === "state" || mode === "details") {
    return mode;
  }
  return "name";
}

export function normalizeDiscordSmartArtworkPreference(
  value: unknown,
): DiscordSmartArtworkPreference {
  return String(value ?? "").trim().toLowerCase() === "app" ? "app" : "music";
}

export function normalizeDiscordAppNameMode(
  value: unknown,
): DiscordAppNameMode {
  const mode = String(value ?? "").trim().toLowerCase();
  if (
    mode === "song" ||
    mode === "artist" ||
    mode === "album" ||
    mode === "source" ||
    mode === "media_source" ||
    mode === "custom"
  ) {
    return mode === "media_source" ? "source" : mode;
  }
  return "default";
}

export function normalizeDiscordLineFormat(value: unknown, fallback = ""): string {
  if (value === undefined || value === null) {
    return fallback;
  }

  const trimmed = String(value).trim();
  return trimmed === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? "" : trimmed;
}
