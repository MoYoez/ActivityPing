import type {
  ClientConfig,
  DiscordActivityType,
  DiscordAssetTextMode,
  DiscordAppNameMode,
  DiscordCustomAppIconSource,
  DiscordCustomArtworkSource,
  DiscordReportMode,
  DiscordSmartArtworkPreference,
  DiscordStatusDisplay,
} from "../../types";

export const DISCORD_CUSTOM_LINE_CUSTOM_VALUE = "__custom__";

export const DISCORD_CUSTOM_LINE_OPTIONS = [
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

export const DISCORD_REPORT_MODE_OPTIONS: Array<{
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

export const DISCORD_ACTIVITY_TYPE_OPTIONS: Array<{
  value: DiscordActivityType;
  label: string;
  helper: string;
}> = [
  { value: "playing", label: "Playing", helper: "Default game-like activity label." },
  { value: "listening", label: "Listening", helper: "Discord shows a listening-style label." },
  { value: "watching", label: "Watching", helper: "Discord shows a watching-style label." },
  { value: "competing", label: "Competing", helper: "Discord shows a competing-style label." },
];

export const DISCORD_STATUS_DISPLAY_OPTIONS: Array<{
  value: DiscordStatusDisplay;
  label: string;
  helper: string;
}> = [
  { value: "name", label: "Application name", helper: "Use the activity name for Discord's compact member-list status text." },
  { value: "state", label: "State line", helper: "Use the state field for Discord's compact member-list status text." },
  { value: "details", label: "Details line", helper: "Use the details field for Discord's compact member-list status text." },
];

export const DISCORD_APP_NAME_OPTIONS: Array<{
  value: DiscordAppNameMode;
  label: string;
  helper: string;
}> = [
  { value: "default", label: "Application name", helper: "Keep Discord's application name. Smart and App mode still use the matched title first when an app is active." },
  { value: "song", label: "Song name", helper: "Use the current track title when the activity falls back to music-first output." },
  { value: "artist", label: "Artist", helper: "Use the current artist when the activity falls back to music-first output." },
  { value: "album", label: "Album", helper: "Use the current album when the activity falls back to music-first output." },
  { value: "source", label: "Media source", helper: "Use the current playback source app when the activity falls back to music-first output." },
  { value: "custom", label: "Custom text", helper: "Type a custom application name for the first line." },
];

export const DISCORD_SMART_ARTWORK_PREFERENCE_OPTIONS: Array<{
  value: DiscordSmartArtworkPreference;
  label: string;
  helper: string;
}> = [
  {
    value: "music",
    label: "Music prefer",
    helper: "Keep Smart mode's current artwork behavior and prioritize media artwork when it exists.",
  },
  {
    value: "app",
    label: "App prefer",
    helper: "Use the current foreground app icon as Smart mode's main artwork, while keeping music on the last line.",
  },
];

export const DISCORD_CUSTOM_ARTWORK_SOURCE_OPTIONS: Array<{
  value: DiscordCustomArtworkSource;
  label: string;
  helper: string;
}> = [
  {
    value: "none",
    label: "Disabled",
    helper: "Turn off the large image slot for Custom mode.",
  },
  {
    value: "auto",
    label: "Auto",
    helper: "Follow the old Custom behavior and mirror the global artwork switches when they are enabled.",
  },
  {
    value: "music",
    label: "Music artwork",
    helper: "Use the current song or media cover art as the large Discord image.",
  },
  {
    value: "app",
    label: "Foreground app",
    helper: "Use the current foreground app icon as the large Discord image.",
  },
  {
    value: "library",
    label: "Gallery image",
    helper: "Use a locally managed image from the Gallery page.",
  },
];

export const DISCORD_CUSTOM_APP_ICON_SOURCE_OPTIONS: Array<{
  value: DiscordCustomAppIconSource;
  label: string;
  helper: string;
}> = [
  {
    value: "none",
    label: "Disabled",
    helper: "Turn off the small image slot for Custom mode.",
  },
  {
    value: "auto",
    label: "Auto",
    helper: "Follow the old Custom behavior and keep the app icon slot tied to the global app artwork switch.",
  },
  {
    value: "app",
    label: "Foreground app",
    helper: "Use the current foreground app icon as the small Discord image.",
  },
  {
    value: "source",
    label: "Playback source",
    helper: "Use the current media source app as the small Discord image.",
  },
  {
    value: "library",
    label: "Gallery image",
    helper: "Use a locally managed image from the Gallery page.",
  },
];

export const DISCORD_ASSET_TEXT_MODE_OPTIONS: Array<{
  value: DiscordAssetTextMode;
  label: string;
  helper: string;
}> = [
  {
    value: "auto",
    label: "Auto",
    helper: "Use the default hover text generated from the chosen asset source.",
  },
  {
    value: "custom",
    label: "Custom text",
    helper: "Type the hover text that Discord should show for this asset.",
  },
];

export const DISCORD_TEMPLATE_TOKENS = [
  "{app}",
  "{title}",
  "{rule}",
  "{media}",
  "{song}",
  "{artist}",
  "{album}",
  "{source}",
] as const;

export function appendDiscordTemplateToken(currentValue: string, token: string) {
  const trimmed = String(currentValue ?? "").trim();
  return trimmed ? `${trimmed} ${token}` : token;
}

export function normalizeDiscordSmartArtworkPreference(value: unknown): DiscordSmartArtworkPreference {
  return String(value ?? "").trim().toLowerCase() === "app" ? "app" : "music";
}

export function discordModeStatusDisplay(config: ClientConfig, mode: DiscordReportMode) {
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

export function discordModeAppNameMode(config: ClientConfig, mode: DiscordReportMode) {
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

export function discordModeCustomAppName(config: ClientConfig, mode: DiscordReportMode) {
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

export function patchDiscordModeSettings(
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

export function normalizeDiscordLineTemplate(value: unknown) {
  const trimmed = String(value ?? "").trim();
  return trimmed === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? "" : trimmed;
}

function isDiscordBuiltinLineChoice(value: string) {
  return DISCORD_CUSTOM_LINE_OPTIONS.some(
    (option) => option.value === value && option.value !== DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
  );
}

export function discordLineOptionLabel(value: string) {
  const normalizedValue = normalizeDiscordLineTemplate(value);
  if (!normalizedValue) {
    return "Hidden";
  }
  const matched = DISCORD_CUSTOM_LINE_OPTIONS.find(
    (option) => option.value === normalizedValue && option.value !== DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
  )?.label;
  return matched ?? "Custom text";
}

export function resolveDiscordLineChoice(value: string, forceCustom = false) {
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

export function discordLineCustomTextValue(value: string, forceCustom = false) {
  const rawValue = String(value ?? "").trim();
  if (!rawValue || rawValue === DISCORD_CUSTOM_LINE_CUSTOM_VALUE) {
    return "";
  }
  if (forceCustom) {
    return rawValue;
  }
  return isDiscordBuiltinLineChoice(rawValue) ? "" : rawValue;
}

export function nextDiscordLineValue(
  currentValue: string,
  nextChoice: string,
  forceCustom = false,
) {
  if (nextChoice !== DISCORD_CUSTOM_LINE_CUSTOM_VALUE) {
    return nextChoice;
  }

  const currentCustomValue = discordLineCustomTextValue(currentValue, forceCustom);
  return currentCustomValue || DISCORD_CUSTOM_LINE_CUSTOM_VALUE;
}
