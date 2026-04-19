import type {
  AppMessageRuleGroup,
  ClientConfig,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
} from "../../types";

import {
  normalizeDiscordActivityType,
  normalizeDiscordAppNameMode,
  normalizeDiscordButton,
  normalizeDiscordCustomPreset,
  normalizeDiscordLineFormat,
  normalizeDiscordReportMode,
  normalizeDiscordStatusDisplay,
  normalizeHistoryLimit,
  normalizePartySize,
  normalizeRuleGroup,
  normalizeStringList,
} from "./helpers";
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
      config.discordMusicAppNameMode ?? legacyConfig.discordMusicAppNameMode ?? legacyGlobalAppNameMode ?? "source",
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
