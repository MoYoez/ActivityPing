import type {
  AppFilterMode,
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppTitleRuleMode,
  ClientConfig,
  DiscordActivityType,
  DiscordReportMode,
} from "../types";

type ParsedRulesPayload =
  | {
      ok: true;
      data: {
        appMessageRules: AppMessageRuleGroup[];
        appMessageRulesShowProcessName: boolean;
        appFilterMode: AppFilterMode;
        appBlacklist: string[];
        appWhitelist: string[];
        appNameOnlyList: string[];
        mediaPlaySourceBlocklist: string[];
      };
    }
  | { ok: false; error: string };

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

  if (!defaultText && titleRules.length === 0) {
    return null;
  }

  return {
    processMatch,
    defaultText,
    titleRules,
  };
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

export function normalizeClientConfig(config: ClientConfig): ClientConfig {
  const discordDetailsFormat = String(config.discordDetailsFormat ?? "").trim();
  const legacyConfig = config as ClientConfig & {
    reporterEnabled?: unknown;
    discordEnabled?: unknown;
    discordUseMediaArtwork?: unknown;
  };
  const runtimeAutostartEnabled = Boolean(
    config.runtimeAutostartEnabled || legacyConfig.reporterEnabled || legacyConfig.discordEnabled,
  );
  const legacyArtworkEnabled = Boolean(legacyConfig.discordUseMediaArtwork);
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
    discordSmartEnableMusicCountdown: config.discordSmartEnableMusicCountdown !== false,
    discordSmartShowAppName: Boolean(config.discordSmartShowAppName),
    discordUseAppArtwork: Boolean(config.discordUseAppArtwork || legacyArtworkEnabled),
    discordUseMusicArtwork: Boolean(config.discordUseMusicArtwork || legacyArtworkEnabled),
    discordArtworkWorkerUploadUrl: String(config.discordArtworkWorkerUploadUrl ?? "").trim(),
    discordArtworkWorkerToken: String(config.discordArtworkWorkerToken ?? "").trim(),
    discordDetailsFormat: discordDetailsFormat || "{activity}",
    discordStateFormat: String(config.discordStateFormat ?? "{context}").trim(),
    launchOnStartup: Boolean(config.launchOnStartup),
    captureReportedAppsEnabled: config.captureReportedAppsEnabled !== false,
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
      version: 2,
      exportedAt: new Date().toISOString(),
      rules: {
        appMessageRules: normalized.appMessageRules,
        appMessageRulesShowProcessName: normalized.appMessageRulesShowProcessName,
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

  const group = raw as { processMatch?: unknown; defaultText?: unknown; titleRules?: unknown; match?: unknown; text?: unknown };
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

  if (!processMatch || (!defaultText && titleRules.length === 0)) {
    return null;
  }

  return {
    processMatch,
    defaultText,
    titleRules,
  };
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
  if (typeof root.version === "number" && root.version !== 1 && root.version !== 2) {
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
  if (rule.titleRules.length > 0) {
    return `${rule.defaultText || "Custom text"} · ${rule.titleRules.length} title rule${rule.titleRules.length === 1 ? "" : "s"}`;
  }
  return rule.defaultText || "No default text";
}
