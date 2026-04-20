import {
  DISCORD_STATUS_DISPLAY_OPTIONS,
  discordLineOptionLabel,
  normalizeDiscordLineTemplate,
} from "../components/discord/discordOptions";
import { normalizeClientConfig } from "../lib/rules";
import type {
  AppMessageRuleGroup,
  AppStatePayload,
  ClientConfig,
  DiscordCustomAsset,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
} from "../types";

import { DEFAULT_HISTORY_TITLE_LIMIT } from "./appConstants";
import { clampHistoryLimit, normalizeAppHistory, normalizePlaySourceHistory } from "./appHistory";
import { discordActivityTypeText } from "./appFormatting";

export function appendUniqueListValue(values: string[], value: string, lowercase: boolean) {
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

export function createDiscordButton(): DiscordRichPresenceButtonConfig {
  return { label: "", url: "" };
}

export function createAppMessageTitleRule() {
  return { mode: "plain" as const, pattern: "", text: "", buttons: [] };
}

export function createAppMessageRuleGroup(): AppMessageRuleGroup {
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

export function createDiscordCustomPreset(): DiscordCustomPreset {
  return {
    name: "",
    activityType: "playing",
    statusDisplay: "name",
    appNameMode: "default",
    customAppName: "",
    detailsFormat: "{activity}",
    stateFormat: "{context}",
    customArtworkSource: "auto",
    customArtworkTextMode: "auto",
    customArtworkText: "",
    customArtworkAssetId: "",
    customAppIconSource: "auto",
    customAppIconTextMode: "auto",
    customAppIconText: "",
    customAppIconAssetId: "",
    buttons: [],
    partyId: "",
    partySizeCurrent: null,
    partySizeMax: null,
    joinSecret: "",
    spectateSecret: "",
    matchSecret: "",
  };
}

export function createDiscordCustomPresetFromConfig(config: ClientConfig): DiscordCustomPreset {
  return {
    name: "",
    activityType: config.discordActivityType,
    statusDisplay: config.discordCustomModeStatusDisplay,
    appNameMode: config.discordCustomModeAppNameMode,
    customAppName: config.discordCustomModeCustomAppName,
    detailsFormat: normalizeDiscordLineTemplate(config.discordDetailsFormat),
    stateFormat: normalizeDiscordLineTemplate(config.discordStateFormat),
    customArtworkSource: config.discordCustomArtworkSource,
    customArtworkTextMode: config.discordCustomArtworkTextMode,
    customArtworkText: config.discordCustomArtworkText,
    customArtworkAssetId: config.discordCustomArtworkAssetId,
    customAppIconSource: config.discordCustomAppIconSource,
    customAppIconTextMode: config.discordCustomAppIconTextMode,
    customAppIconText: config.discordCustomAppIconText,
    customAppIconAssetId: config.discordCustomAppIconAssetId,
    buttons: config.discordCustomButtons.map((button) => ({ ...button })),
    partyId: config.discordCustomPartyId,
    partySizeCurrent: config.discordCustomPartySizeCurrent ?? null,
    partySizeMax: config.discordCustomPartySizeMax ?? null,
    joinSecret: config.discordCustomJoinSecret,
    spectateSecret: config.discordCustomSpectateSecret,
    matchSecret: config.discordCustomMatchSecret,
  };
}

export function summarizeDiscordCustomPreset(preset: DiscordCustomPreset) {
  const extras = [];
  if (
    preset.customArtworkSource !== "auto" ||
    preset.customArtworkTextMode !== "auto" ||
    preset.customAppIconSource !== "auto" ||
    preset.customAppIconTextMode !== "auto" ||
    preset.customArtworkAssetId.trim() ||
    preset.customAppIconAssetId.trim()
  ) {
    extras.push("artwork");
  }
  if (preset.buttons.length > 0) extras.push(`${preset.buttons.length} button${preset.buttons.length === 1 ? "" : "s"}`);
  if (preset.partyId.trim() || preset.partySizeCurrent || preset.partySizeMax) extras.push("party");
  if (preset.joinSecret.trim() || preset.spectateSecret.trim() || preset.matchSecret.trim()) extras.push("secrets");
  const extraText = extras.length > 0 ? ` · ${extras.join(" · ")}` : "";
  return `${discordActivityTypeText(preset.activityType)} · ${DISCORD_STATUS_DISPLAY_OPTIONS.find((option) => option.value === preset.statusDisplay)?.label ?? "Application name"} · ${discordLineOptionLabel(preset.detailsFormat.trim())} / ${discordLineOptionLabel(preset.stateFormat.trim())}${extraText}`;
}

export function normalizePositiveNumberInput(value: string) {
  const parsed = Math.floor(Number(value));
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }
  return parsed;
}

export function validateRuleRegex(config: ClientConfig) {
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

export function validateArtworkPublishing(config: ClientConfig) {
  if (usesArtworkPublishing(config) && !config.discordArtworkWorkerUploadUrl.trim()) {
    return "Artwork publishing requires an uploader service URL when app artwork, music artwork, or Custom Gallery image slots are enabled.";
  }
  return null;
}

export function usesArtworkPublishing(config: ClientConfig) {
  return (
    config.discordUseAppArtwork ||
    config.discordUseMusicArtwork ||
    config.discordCustomArtworkSource === "music" ||
    config.discordCustomArtworkSource === "app" ||
    (config.discordCustomArtworkSource === "library" && config.discordCustomArtworkAssetId.trim().length > 0) ||
    config.discordCustomAppIconSource === "app" ||
    config.discordCustomAppIconSource === "source" ||
    (config.discordCustomAppIconSource === "library" && config.discordCustomAppIconAssetId.trim().length > 0)
  );
}

export function replaceDiscordCustomAssets(
  config: ClientConfig,
  assets: DiscordCustomAsset[],
): ClientConfig {
  return {
    ...config,
    discordCustomAssets: assets.map((asset) => ({ ...asset })),
  };
}

export function clearDiscordCustomAssetReferences(
  config: ClientConfig,
  assetId: string,
): ClientConfig {
  const trimmedId = assetId.trim();
  if (!trimmedId) {
    return config;
  }

  const clearPreset = (preset: DiscordCustomPreset): DiscordCustomPreset => ({
    ...preset,
    customArtworkSource:
      preset.customArtworkSource === "library" && preset.customArtworkAssetId.trim() === trimmedId
        ? "none"
        : preset.customArtworkSource,
    customArtworkAssetId:
      preset.customArtworkAssetId.trim() === trimmedId ? "" : preset.customArtworkAssetId,
    customAppIconSource:
      preset.customAppIconSource === "library" && preset.customAppIconAssetId.trim() === trimmedId
        ? "none"
        : preset.customAppIconSource,
    customAppIconAssetId:
      preset.customAppIconAssetId.trim() === trimmedId ? "" : preset.customAppIconAssetId,
  });

  return {
    ...config,
    discordCustomArtworkSource:
      config.discordCustomArtworkSource === "library" && config.discordCustomArtworkAssetId.trim() === trimmedId
        ? "none"
        : config.discordCustomArtworkSource,
    discordCustomArtworkAssetId:
      config.discordCustomArtworkAssetId.trim() === trimmedId ? "" : config.discordCustomArtworkAssetId,
    discordCustomAppIconSource:
      config.discordCustomAppIconSource === "library" && config.discordCustomAppIconAssetId.trim() === trimmedId
        ? "none"
        : config.discordCustomAppIconSource,
    discordCustomAppIconAssetId:
      config.discordCustomAppIconAssetId.trim() === trimmedId ? "" : config.discordCustomAppIconAssetId,
    discordCustomPresets: config.discordCustomPresets.map(clearPreset),
    discordCustomAssets: config.discordCustomAssets.filter((asset) => asset.id !== trimmedId),
  };
}

export function buildPayload(baseState: AppStatePayload, config: ClientConfig): AppStatePayload {
  const normalizedConfig = normalizeClientConfig(config);
  const historyTitleLimit = clampHistoryLimit(normalizedConfig.captureHistoryTitleLimit, DEFAULT_HISTORY_TITLE_LIMIT);
  return {
    ...baseState,
    config: normalizedConfig,
    appHistory: normalizeAppHistory(baseState.appHistory, historyTitleLimit),
    playSourceHistory: normalizePlaySourceHistory(baseState.playSourceHistory),
    locale: "en-US",
  };
}
