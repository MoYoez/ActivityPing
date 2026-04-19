export type AppTitleRuleMode = "plain" | "regex";
export type AppFilterMode = "blacklist" | "whitelist";
export type DiscordReportMode = "music" | "app" | "mixed" | "custom";
export type DiscordActivityType = "playing" | "listening" | "watching" | "competing";
export type DiscordStatusDisplay = "name" | "state" | "details";
export type DiscordAppNameMode = "default" | "song" | "artist" | "album" | "source" | "custom";

export interface ClientCapabilities {
  realtimeReporter: boolean;
  tray: boolean;
  platformSelfTest: boolean;
  discordPresence: boolean;
  autostart: boolean;
}

export interface AppMessageTitleRule {
  mode: AppTitleRuleMode;
  pattern: string;
  text: string;
}

export interface AppMessageRuleGroup {
  processMatch: string;
  defaultText: string;
  titleRules: AppMessageTitleRule[];
  buttons: DiscordRichPresenceButtonConfig[];
  partyId: string;
  partySizeCurrent?: number | null;
  partySizeMax?: number | null;
  joinSecret: string;
  spectateSecret: string;
  matchSecret: string;
}

export interface DiscordCustomPreset {
  name: string;
  activityType: DiscordActivityType;
  statusDisplay: DiscordStatusDisplay;
  appNameMode: DiscordAppNameMode;
  customAppName: string;
  detailsFormat: string;
  stateFormat: string;
  buttons: DiscordRichPresenceButtonConfig[];
  partyId: string;
  partySizeCurrent?: number | null;
  partySizeMax?: number | null;
  joinSecret: string;
  spectateSecret: string;
  matchSecret: string;
}

export interface DiscordRichPresenceButtonConfig {
  label: string;
  url: string;
}

export interface ClientConfig {
  pollIntervalMs: number;
  heartbeatIntervalMs: number;
  runtimeAutostartEnabled: boolean;
  reportForegroundApp: boolean;
  reportWindowTitle: boolean;
  reportMedia: boolean;
  reportStoppedMedia: boolean;
  reportPlaySource: boolean;
  discordApplicationId: string;
  discordReportMode: DiscordReportMode;
  discordActivityType: DiscordActivityType;
  discordSmartStatusDisplay: DiscordStatusDisplay;
  discordSmartAppNameMode: DiscordAppNameMode;
  discordSmartCustomAppName: string;
  discordMusicStatusDisplay: DiscordStatusDisplay;
  discordMusicAppNameMode: DiscordAppNameMode;
  discordMusicCustomAppName: string;
  discordAppStatusDisplay: DiscordStatusDisplay;
  discordAppAppNameMode: DiscordAppNameMode;
  discordAppCustomAppName: string;
  discordCustomModeStatusDisplay: DiscordStatusDisplay;
  discordCustomModeAppNameMode: DiscordAppNameMode;
  discordCustomModeCustomAppName: string;
  discordSmartEnableMusicCountdown: boolean;
  discordSmartShowAppName: boolean;
  discordUseAppArtwork: boolean;
  discordUseMusicArtwork: boolean;
  discordArtworkWorkerUploadUrl: string;
  discordArtworkWorkerToken: string;
  discordDetailsFormat: string;
  discordStateFormat: string;
  discordCustomButtons: DiscordRichPresenceButtonConfig[];
  discordCustomPartyId: string;
  discordCustomPartySizeCurrent?: number | null;
  discordCustomPartySizeMax?: number | null;
  discordCustomJoinSecret: string;
  discordCustomSpectateSecret: string;
  discordCustomMatchSecret: string;
  discordUseCustomAddonsOverride: boolean;
  discordCustomPresets: DiscordCustomPreset[];
  launchOnStartup: boolean;
  captureReportedAppsEnabled: boolean;
  captureHistoryRecordLimit: number;
  captureHistoryTitleLimit: number;
  appMessageRules: AppMessageRuleGroup[];
  appMessageRulesShowProcessName: boolean;
  appFilterMode: AppFilterMode;
  appBlacklist: string[];
  appWhitelist: string[];
  appNameOnlyList: string[];
  mediaPlaySourceBlocklist: string[];
}

export interface AppStatePayload {
  config: ClientConfig;
  appHistory: AppHistoryEntry[];
  playSourceHistory: PlaySourceHistoryEntry[];
  locale?: string;
}

export interface AppHistoryEntry {
  processName: string;
  processTitle?: string | null;
  processTitles?: string[];
  statusText?: string | null;
  updatedAt?: string | null;
}

export interface PlaySourceHistoryEntry {
  source: string;
  mediaTitle?: string | null;
  mediaArtist?: string | null;
  mediaAlbum?: string | null;
  mediaSummary?: string | null;
  updatedAt?: string | null;
}

export interface ApiError {
  status: number;
  message: string;
  code?: string | null;
  params?: Record<string, unknown> | null;
  details?: unknown;
}

export interface ApiResult<T> {
  success: boolean;
  status: number;
  data?: T;
  error?: ApiError;
}

export interface ReporterActivity {
  processName: string;
  processTitle?: string | null;
  rawProcessTitle?: string | null;
  mediaTitle?: string | null;
  mediaArtist?: string | null;
  mediaAlbum?: string | null;
  mediaSummary?: string | null;
  mediaDurationMs?: number | null;
  mediaPositionMs?: number | null;
  playSource?: string | null;
  statusText?: string | null;
  updatedAt?: string | null;
}

export type ReporterLogLevel = "info" | "success" | "warn" | "error";

export interface ReporterLogEntry {
  id: string;
  timestamp: string;
  level: ReporterLogLevel;
  title: string;
  detail: string;
  titleKey?: string | null;
  titleParams?: Record<string, unknown> | null;
  detailKey?: string | null;
  detailParams?: Record<string, unknown> | null;
  payload?: Record<string, unknown> | null;
}

export interface RealtimeReporterSnapshot {
  running: boolean;
  logs: ReporterLogEntry[];
  currentActivity?: ReporterActivity | null;
  lastHeartbeatAt?: string | null;
  lastError?: string | null;
}

export interface DiscordPresenceSnapshot {
  running: boolean;
  connected: boolean;
  lastSyncAt?: string | null;
  lastError?: string | null;
  currentSummary?: string | null;
  debugPayload?: DiscordDebugPayload | null;
}

export interface DiscordDebugPayload {
  activityName?: string | null;
  details: string;
  state?: string | null;
  summary: string;
  signature: string;
  reportModeApplied: string;
  activityType: string;
  statusDisplayType?: string | null;
  startedAtMillis?: number | null;
  endedAtMillis?: number | null;
  mediaDurationMs?: number | null;
  mediaPositionMs?: number | null;
  appIconUrl?: string | null;
  appIconText?: string | null;
  appIconError?: string | null;
  artworkUrl?: string | null;
  artworkHoverText?: string | null;
  artworkContentType?: string | null;
  artworkUploadError?: string | null;
}

export interface LocalizedTextEntry {
  text: string;
  key?: string | null;
  params?: Record<string, unknown> | null;
}

export interface PlatformProbeResult {
  success: boolean;
  summary: string;
  detail: string;
  guidance?: string[];
  summaryKey?: string | null;
  summaryParams?: Record<string, unknown> | null;
  detailKey?: string | null;
  detailParams?: Record<string, unknown> | null;
  guidanceEntries?: LocalizedTextEntry[] | null;
}

export interface PlatformSelfTestResult {
  platform: string;
  foreground: PlatformProbeResult;
  windowTitle: PlatformProbeResult;
  media: PlatformProbeResult;
}
