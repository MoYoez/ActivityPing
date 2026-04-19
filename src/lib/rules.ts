import type {
  AppFilterMode,
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppTitleRuleMode,
  ClientConfig,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
  DiscordAppNameMode,
  DiscordActivityType,
  DiscordReportMode,
  DiscordStatusDisplay,
} from "../types";

const MIN_HISTORY_LIMIT = 1;
const MAX_HISTORY_LIMIT = 50;
const DISCORD_CUSTOM_LINE_CUSTOM_VALUE = "__custom__";

type ParsedRulesPayload =
  | {
      ok: true;
      data: {
        appMessageRules: AppMessageRuleGroup[];
        appMessageRulesShowProcessName: boolean;
    discordCustomPresets: DiscordCustomPreset[];
    discordUseCustomAddonsOverride: boolean;
        appFilterMode: AppFilterMode;
        appBlacklist: string[];
        appWhitelist: string[];
        appNameOnlyList: string[];
        mediaPlaySourceBlocklist: string[];
      };
    }
  | { ok: false; error: string };

function normalizeHistoryLimit(value: unknown, fallback: number) {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(MAX_HISTORY_LIMIT, Math.max(MIN_HISTORY_LIMIT, parsed));
}

function normalizeStringList(values: string[], lowercase: boolean) {
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

function normalizeTitleRule(rule: AppMessageTitleRule): AppMessageTitleRule | null {
  const mode: AppTitleRuleMode = String(rule.mode ?? "").trim().toLowerCase() === "regex" ? "regex" : "plain";
  const pattern = String(rule.pattern ?? "").trim();
  const text = String(rule.text ?? "").trim();
  if (!pattern || !text) {
    return null;
  }
  return { mode, pattern, text };
}

function normalizeRuleGroup(rule: AppMessageRuleGroup): AppMessageRuleGroup | null {
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

function normalizeDiscordCustomPreset(rule: Partial<DiscordCustomPreset>): DiscordCustomPreset | null {
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

function normalizeDiscordButton(
  button: Partial<DiscordRichPresenceButtonConfig>,
): DiscordRichPresenceButtonConfig | null {
  const label = String(button.label ?? "").trim();
  const url = String(button.url ?? "").trim();
  if (!label || !url) {
    return null;
  }
  return { label, url };
}

function normalizePartySize(value: unknown): number | null {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }
  return parsed;
}

function normalizeDiscordReportMode(value: unknown): DiscordReportMode {
  const mode = String(value ?? "").trim().toLowerCase();
  if (mode === "music" || mode === "app" || mode === "custom") {
    return mode;
  }
  return "mixed";
}

function normalizeDiscordActivityType(value: unknown): DiscordActivityType {
  const mode = String(value ?? "").trim().toLowerCase();
  if (mode === "listening" || mode === "watching" || mode === "competing") {
    return mode;
  }
  return "playing";
}

function normalizeDiscordStatusDisplay(
  value: unknown,
): DiscordStatusDisplay {
  const mode = String(value ?? "").trim().toLowerCase();
  if (mode === "state" || mode === "details") {
    return mode;
  }
  return "name";
}

function normalizeDiscordAppNameMode(
  value: unknown,
): DiscordAppNameMode {
  const mode = String(value ?? "").trim().toLowerCase();
  if (
    mode === "song" ||
    mode === "artist" ||
    mode === "album" ||
    mode === "custom"
  ) {
    return mode;
  }
  return "default";
}

function normalizeDiscordLineFormat(value: unknown, fallback = ""): string {
  if (value === undefined || value === null) {
    return fallback;
  }

  const trimmed = String(value).trim();
  return trimmed === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? "" : trimmed;
}

export function normalizeClientConfig(config: ClientConfig): ClientConfig {
  const discordDetailsFormat = normalizeDiscordLineFormat(config.discordDetailsFormat, "{activity}");
  const discordStateFormat = normalizeDiscordLineFormat(config.discordStateFormat, "{context}");
  const legacyConfig = config as ClientConfig & {
    reporterEnabled?: unknown;
    discordEnabled?: unknown;
    discordUseMediaArtwork?: unknown;
    discordCustomRules?: unknown;
    discordStatusDisplay?: unknown;
    discordAppNameMode?: unknown;
    discordCustomAppName?: unknown;
    discordMusicStatusDisplay?: unknown;
    discordMusicAppNameMode?: unknown;
    discordMusicCustomAppName?: unknown;
    discordSmartStatusDisplay?: unknown;
    discordSmartAppNameMode?: unknown;
    discordSmartCustomAppName?: unknown;
    discordAppStatusDisplay?: unknown;
    discordAppAppNameMode?: unknown;
    discordAppCustomAppName?: unknown;
    discordCustomModeStatusDisplay?: unknown;
    discordCustomModeAppNameMode?: unknown;
    discordCustomModeCustomAppName?: unknown;
  };
  const runtimeAutostartEnabled = Boolean(
    config.runtimeAutostartEnabled || legacyConfig.reporterEnabled || legacyConfig.discordEnabled,
  );
  const legacyArtworkEnabled = Boolean(legacyConfig.discordUseMediaArtwork);
  const legacyGlobalStatusDisplay = legacyConfig.discordStatusDisplay;
  const legacyGlobalAppNameMode = legacyConfig.discordAppNameMode;
  const legacyGlobalCustomAppName = legacyConfig.discordCustomAppName;
  return {
    pollIntervalMs: Math.max(1000, Number(config.pollIntervalMs) || 1000),
    heartbeatIntervalMs: Math.max(0, Number(config.heartbeatIntervalMs) || 0),
    runtimeAutostartEnabled,
    reportForegroundApp: config.reportForegroundApp !== false,
    reportWindowTitle: config.reportWindowTitle !== false,
    reportMedia: config.reportMedia !== false,
    reportStoppedMedia: Boolean(config.reportStoppedMedia),
    reportPlaySource: config.reportPlaySource !== false,
    discordApplicationId: String(config.discordApplicationId ?? "").trim(),
    discordReportMode: normalizeDiscordReportMode(config.discordReportMode),
    discordActivityType: normalizeDiscordActivityType(config.discordActivityType),
    discordSmartStatusDisplay: normalizeDiscordStatusDisplay(
      config.discordSmartStatusDisplay ?? legacyConfig.discordSmartStatusDisplay ?? legacyGlobalStatusDisplay,
    ),
    discordSmartAppNameMode: normalizeDiscordAppNameMode(
      config.discordSmartAppNameMode ?? legacyConfig.discordSmartAppNameMode ?? legacyGlobalAppNameMode,
    ),
    discordSmartCustomAppName: String(
      config.discordSmartCustomAppName ?? legacyConfig.discordSmartCustomAppName ?? legacyGlobalCustomAppName ?? "",
    ).trim(),
    discordMusicStatusDisplay: normalizeDiscordStatusDisplay(
      config.discordMusicStatusDisplay ?? legacyConfig.discordMusicStatusDisplay ?? legacyGlobalStatusDisplay,
    ),
    discordMusicAppNameMode: normalizeDiscordAppNameMode(
      config.discordMusicAppNameMode ?? legacyConfig.discordMusicAppNameMode ?? legacyGlobalAppNameMode,
    ),
    discordMusicCustomAppName: String(
      config.discordMusicCustomAppName ?? legacyConfig.discordMusicCustomAppName ?? legacyGlobalCustomAppName ?? "",
    ).trim(),
    discordAppStatusDisplay: normalizeDiscordStatusDisplay(
      config.discordAppStatusDisplay ?? legacyConfig.discordAppStatusDisplay ?? legacyGlobalStatusDisplay,
    ),
    discordAppAppNameMode: normalizeDiscordAppNameMode(
      config.discordAppAppNameMode ?? legacyConfig.discordAppAppNameMode ?? legacyGlobalAppNameMode,
    ),
    discordAppCustomAppName: String(
      config.discordAppCustomAppName ?? legacyConfig.discordAppCustomAppName ?? legacyGlobalCustomAppName ?? "",
    ).trim(),
    discordCustomModeStatusDisplay: normalizeDiscordStatusDisplay(
      config.discordCustomModeStatusDisplay ?? legacyConfig.discordCustomModeStatusDisplay ?? legacyGlobalStatusDisplay,
    ),
    discordCustomModeAppNameMode: normalizeDiscordAppNameMode(
      config.discordCustomModeAppNameMode ?? legacyConfig.discordCustomModeAppNameMode ?? legacyGlobalAppNameMode,
    ),
    discordCustomModeCustomAppName: String(
      config.discordCustomModeCustomAppName ??
        legacyConfig.discordCustomModeCustomAppName ??
        legacyGlobalCustomAppName ??
        "",
    ).trim(),
    discordSmartEnableMusicCountdown: config.discordSmartEnableMusicCountdown !== false,
    discordSmartShowAppName: Boolean(config.discordSmartShowAppName),
    discordUseAppArtwork: Boolean(config.discordUseAppArtwork || legacyArtworkEnabled),
    discordUseMusicArtwork: Boolean(config.discordUseMusicArtwork || legacyArtworkEnabled),
    discordArtworkWorkerUploadUrl: String(config.discordArtworkWorkerUploadUrl ?? "").trim(),
    discordArtworkWorkerToken: String(config.discordArtworkWorkerToken ?? "").trim(),
    discordDetailsFormat,
    discordStateFormat,
    discordCustomButtons: (Array.isArray(config.discordCustomButtons) ? config.discordCustomButtons : [])
      .map((button) => normalizeDiscordButton(button))
      .filter((button): button is DiscordRichPresenceButtonConfig => button !== null)
      .slice(0, 2),
    discordCustomPartyId: String(config.discordCustomPartyId ?? "").trim(),
    discordCustomPartySizeCurrent: normalizePartySize(config.discordCustomPartySizeCurrent),
    discordCustomPartySizeMax: normalizePartySize(config.discordCustomPartySizeMax),
    discordCustomJoinSecret: String(config.discordCustomJoinSecret ?? "").trim(),
    discordCustomSpectateSecret: String(config.discordCustomSpectateSecret ?? "").trim(),
    discordCustomMatchSecret: String(config.discordCustomMatchSecret ?? "").trim(),
    discordUseCustomAddonsOverride: Boolean(config.discordUseCustomAddonsOverride),
    discordCustomPresets: (
      Array.isArray(config.discordCustomPresets)
        ? config.discordCustomPresets
        : Array.isArray(legacyConfig.discordCustomRules)
          ? legacyConfig.discordCustomRules
          : []
    )
      .map((rule) => normalizeDiscordCustomPreset(rule as Partial<DiscordCustomPreset>))
      .filter((rule): rule is DiscordCustomPreset => rule !== null),
    launchOnStartup: Boolean(config.launchOnStartup),
    captureReportedAppsEnabled: config.captureReportedAppsEnabled !== false,
    captureHistoryRecordLimit: normalizeHistoryLimit(config.captureHistoryRecordLimit, 3),
    captureHistoryTitleLimit: normalizeHistoryLimit(config.captureHistoryTitleLimit, 5),
    appMessageRules: (Array.isArray(config.appMessageRules) ? config.appMessageRules : [])
      .map((rule) => normalizeRuleGroup(rule))
      .filter((rule): rule is AppMessageRuleGroup => rule !== null),
    appMessageRulesShowProcessName: Boolean(config.appMessageRulesShowProcessName),
    appBlacklist: normalizeStringList(Array.isArray(config.appBlacklist) ? config.appBlacklist : [], false),
    appWhitelist: normalizeStringList(Array.isArray(config.appWhitelist) ? config.appWhitelist : [], false),
    appNameOnlyList: normalizeStringList(Array.isArray(config.appNameOnlyList) ? config.appNameOnlyList : [], false),
    mediaPlaySourceBlocklist: normalizeStringList(
      Array.isArray(config.mediaPlaySourceBlocklist) ? config.mediaPlaySourceBlocklist : [],
      true,
    ),
    appFilterMode: config.appFilterMode === "whitelist" ? "whitelist" : "blacklist",
  };
}

export function exportRulesJson(config: ClientConfig) {
  const normalized = normalizeClientConfig(config);
  return JSON.stringify(
    {
      version: 4,
      exportedAt: new Date().toISOString(),
      rules: {
        appMessageRules: normalized.appMessageRules,
        appMessageRulesShowProcessName: normalized.appMessageRulesShowProcessName,
        discordUseCustomAddonsOverride: normalized.discordUseCustomAddonsOverride,
        discordCustomPresets: normalized.discordCustomPresets,
        appFilterMode: normalized.appFilterMode,
        appBlacklist: normalized.appBlacklist,
        appWhitelist: normalized.appWhitelist,
        appNameOnlyList: normalized.appNameOnlyList,
        mediaPlaySourceBlocklist: normalized.mediaPlaySourceBlocklist,
      },
    },
    null,
    2,
  );
}

function normalizeImportedRuleGroup(raw: unknown): AppMessageRuleGroup | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return null;
  }

  const group = raw as {
    processMatch?: unknown;
    defaultText?: unknown;
    titleRules?: unknown;
    buttons?: unknown;
    partyId?: unknown;
    partySizeCurrent?: unknown;
    partySizeMax?: unknown;
    joinSecret?: unknown;
    spectateSecret?: unknown;
    matchSecret?: unknown;
    match?: unknown;
    text?: unknown;
  };
  const processMatch = String(group.processMatch ?? group.match ?? "").trim();
  const defaultText = String(group.defaultText ?? group.text ?? "").trim();
  const titleRules = (Array.isArray(group.titleRules) ? group.titleRules : [])
    .map((item) => {
      if (!item || typeof item !== "object" || Array.isArray(item)) {
        return null;
      }
      const titleRule = item as { mode?: unknown; pattern?: unknown; text?: unknown };
      return normalizeTitleRule({
        mode: String(titleRule.mode ?? "plain") as AppTitleRuleMode,
        pattern: String(titleRule.pattern ?? ""),
        text: String(titleRule.text ?? ""),
      });
    })
    .filter((item): item is AppMessageTitleRule => item !== null);
  const buttons = (Array.isArray(group.buttons) ? group.buttons : [])
    .map((button) => normalizeDiscordButton(button as Partial<DiscordRichPresenceButtonConfig>))
    .filter((button): button is DiscordRichPresenceButtonConfig => button !== null)
    .slice(0, 2);
  const partyId = String(group.partyId ?? "").trim();
  const partySizeCurrent = normalizePartySize(group.partySizeCurrent);
  const partySizeMax = normalizePartySize(group.partySizeMax);
  const joinSecret = String(group.joinSecret ?? "").trim();
  const spectateSecret = String(group.spectateSecret ?? "").trim();
  const matchSecret = String(group.matchSecret ?? "").trim();

  if (
    !processMatch ||
    (!defaultText &&
      titleRules.length === 0 &&
      buttons.length === 0 &&
      !partyId &&
      partySizeCurrent === null &&
      partySizeMax === null &&
      !joinSecret &&
      !spectateSecret &&
      !matchSecret)
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

function normalizeImportedDiscordCustomPreset(raw: unknown): DiscordCustomPreset | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return null;
  }

  const rule = raw as {
    name?: unknown;
    activityType?: unknown;
    statusDisplay?: unknown;
    appNameMode?: unknown;
    customAppName?: unknown;
    detailsFormat?: unknown;
    stateFormat?: unknown;
    buttons?: unknown;
    partyId?: unknown;
    partySizeCurrent?: unknown;
    partySizeMax?: unknown;
    joinSecret?: unknown;
    spectateSecret?: unknown;
    matchSecret?: unknown;
  };

  return normalizeDiscordCustomPreset({
    name: String(rule.name ?? ""),
    activityType: normalizeDiscordActivityType(rule.activityType),
    statusDisplay: normalizeDiscordStatusDisplay(rule.statusDisplay),
    appNameMode: normalizeDiscordAppNameMode(rule.appNameMode),
    customAppName: String(rule.customAppName ?? ""),
    detailsFormat: String(rule.detailsFormat ?? ""),
    stateFormat: String(rule.stateFormat ?? ""),
    buttons: Array.isArray(rule.buttons) ? (rule.buttons as DiscordRichPresenceButtonConfig[]) : [],
    partyId: String(rule.partyId ?? ""),
    partySizeCurrent: rule.partySizeCurrent as number | null | undefined,
    partySizeMax: rule.partySizeMax as number | null | undefined,
    joinSecret: String(rule.joinSecret ?? ""),
    spectateSecret: String(rule.spectateSecret ?? ""),
    matchSecret: String(rule.matchSecret ?? ""),
  });
}

export function parseRulesJson(raw: string): ParsedRulesPayload {
  let json: unknown;
  try {
    json = JSON.parse(raw);
  } catch {
    return { ok: false, error: "JSON parse failed." };
  }

  if (!json || typeof json !== "object" || Array.isArray(json)) {
    return { ok: false, error: "The JSON top level must be an object." };
  }

  const root = json as Record<string, unknown>;
  if (typeof root.version === "number" && root.version !== 1 && root.version !== 2 && root.version !== 3 && root.version !== 4) {
    return { ok: false, error: "Unsupported rules JSON version." };
  }

  const rules = root.rules;
  if (!rules || typeof rules !== "object" || Array.isArray(rules)) {
    return { ok: false, error: "Missing rules object." };
  }

  const payload = rules as Record<string, unknown>;
  return {
    ok: true,
    data: {
      appMessageRules: (Array.isArray(payload.appMessageRules) ? payload.appMessageRules : [])
        .map((rule) => normalizeImportedRuleGroup(rule))
        .filter((rule): rule is AppMessageRuleGroup => rule !== null),
      appMessageRulesShowProcessName:
        typeof payload.appMessageRulesShowProcessName === "boolean"
          ? payload.appMessageRulesShowProcessName
          : false,
      discordUseCustomAddonsOverride:
        typeof payload.discordUseCustomAddonsOverride === "boolean"
          ? payload.discordUseCustomAddonsOverride
          : false,
      discordCustomPresets: (
        Array.isArray(payload.discordCustomPresets)
          ? payload.discordCustomPresets
          : Array.isArray(payload.discordCustomRules)
            ? payload.discordCustomRules
            : []
      )
        .map((rule) => normalizeImportedDiscordCustomPreset(rule))
        .filter((rule): rule is DiscordCustomPreset => rule !== null),
      appFilterMode: String(payload.appFilterMode ?? "").trim().toLowerCase() === "whitelist" ? "whitelist" : "blacklist",
      appBlacklist: normalizeStringList(
        Array.isArray(payload.appBlacklist) ? payload.appBlacklist.map((item) => String(item ?? "")) : [],
        false,
      ),
      appWhitelist: normalizeStringList(
        Array.isArray(payload.appWhitelist) ? payload.appWhitelist.map((item) => String(item ?? "")) : [],
        false,
      ),
      appNameOnlyList: normalizeStringList(
        Array.isArray(payload.appNameOnlyList) ? payload.appNameOnlyList.map((item) => String(item ?? "")) : [],
        false,
      ),
      mediaPlaySourceBlocklist: normalizeStringList(
        Array.isArray(payload.mediaPlaySourceBlocklist)
          ? payload.mediaPlaySourceBlocklist.map((item) => String(item ?? ""))
          : [],
        true,
      ),
    },
  };
}

export function summarizeRuleGroup(rule: AppMessageRuleGroup) {
  const parts = [];
  parts.push(rule.defaultText || "No default text");
  if (rule.titleRules.length > 0) {
    parts.push(`${rule.titleRules.length} title rule${rule.titleRules.length === 1 ? "" : "s"}`);
  }
  if (rule.buttons.length > 0) {
    parts.push(`${rule.buttons.length} button${rule.buttons.length === 1 ? "" : "s"}`);
  }
  if (rule.partyId.trim() || rule.partySizeCurrent || rule.partySizeMax) {
    parts.push("party");
  }
  if (rule.joinSecret.trim() || rule.spectateSecret.trim() || rule.matchSecret.trim()) {
    parts.push("secrets");
  }
  return parts.join(" · ");
}
