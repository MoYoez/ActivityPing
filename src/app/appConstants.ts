import type { ClientCapabilities, DiscordPresenceSnapshot, RealtimeReporterSnapshot } from "../types";

export const CARD_CLASS = "card border border-base-300 bg-base-100 shadow-sm";
export const PANEL_CLASS = `${CARD_CLASS} space-y-4 p-4`;
export const PANEL_HEAD_CLASS = "flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between";
export const FIELD_CLASS = "flex min-w-0 flex-col gap-2";
export const FIELD_SPAN_CLASS = `${FIELD_CLASS} field-span-2`;
export const INPUT_CLASS = "input input-bordered w-full";
export const TEXTAREA_CLASS = "textarea textarea-bordered w-full";
export const SELECT_CLASS = "select select-bordered w-full";
export const BUTTON_CLASS = "btn btn-outline btn-sm";
export const PRIMARY_BUTTON_CLASS = "btn btn-primary btn-sm";
export const DANGER_BUTTON_CLASS = "btn btn-error btn-outline btn-sm";
export const BADGE_CLASS = "badge badge-soft";
export const GOOD_BADGE_CLASS = "badge badge-success badge-soft";
export const STAT_CARD_CLASS = "min-h-[92px] rounded-box border border-base-300 bg-base-200/70 p-4";
export const TOGGLE_TILE_CLASS =
  "flex min-h-[88px] items-center justify-between gap-3 rounded-box border border-base-300 bg-base-200/70 p-4 text-left";
export const RADIO_CARD_CLASS = "flex items-start gap-3 rounded-box border border-base-300 bg-base-100 p-4 text-left";
export const ACTIVE_RADIO_CARD_CLASS =
  "flex items-start gap-3 rounded-box border border-primary bg-primary/10 p-4 text-left";
export const SUBRULE_CARD_CLASS = "card border border-base-300 bg-base-100 shadow-sm";
export const GITHUB_URL = "https://github.com/MoYoez/ActivityPing";
export const MAX_RUNTIME_LOGS = 20;
export const RUNTIME_LOG_PAGE_SIZE = 6;
export const DEFAULT_HISTORY_RECORD_LIMIT = 3;
export const DEFAULT_HISTORY_TITLE_LIMIT = 5;
export const MIN_HISTORY_LIMIT = 1;
export const MAX_HISTORY_LIMIT = 50;
export const RULE_GROUP_PAGE_SIZE = 6;
export const TITLE_RULE_PAGE_SIZE = 3;
export const CUSTOM_PRESET_PAGE_SIZE = 5;

export const DEFAULT_CAPABILITIES: ClientCapabilities = {
  realtimeReporter: true,
  tray: true,
  platformSelfTest: true,
  discordPresence: true,
  autostart: true,
};

export const EMPTY_REPORTER: RealtimeReporterSnapshot = {
  running: false,
  logs: [],
  currentActivity: null,
  lastHeartbeatAt: null,
  lastError: null,
};

export const EMPTY_DISCORD: DiscordPresenceSnapshot = {
  running: false,
  connected: false,
  lastSyncAt: null,
  lastError: null,
  currentSummary: null,
  debugPayload: null,
};
