import {
  CUSTOM_PRESET_PAGE_SIZE,
} from "./appConstants";
import {
  buildPayload,
  createDiscordCustomPresetFromConfig,
  validateArtworkPublishing,
  validateRuleRegex,
} from "./appConfig";
import { pageForIndex } from "./appFormatting";
import { normalizeDiscordLineTemplate } from "../components/discord/discordOptions";
import type { ViewSection } from "../components/pages/pageSections";
import { saveAppState, setAutostartEnabled } from "../lib/api";
import { normalizeClientConfig } from "../lib/rules";
import type { NoticeTone } from "../store/appUiStore";
import type { AppStatePayload, ClientConfig, ClientCapabilities, DiscordCustomPreset } from "../types";

interface UseProfileActionsArgs {
  capabilities: ClientCapabilities;
  baseState: AppStatePayload;
  config: ClientConfig;
  appliedCustomPresetIndex: number | null;
  notify: (tone: NoticeTone, title: string, detail: string) => void;
  setActiveSection: (section: ViewSection) => void;
  setBaseState: (payload: AppStatePayload) => void;
  setConfig: (value: ClientConfig | ((current: ClientConfig) => ClientConfig)) => void;
  setCustomPresetPage: (page: number | ((current: number) => number)) => void;
  setActiveCustomPresetIndex: (index: number | null) => void;
  setAppliedCustomPresetIndex: (index: number | null) => void;
  setCustomRulesDialogOpen: (open: boolean) => void;
}

export function useProfileActions({
  capabilities,
  baseState,
  config,
  appliedCustomPresetIndex,
  notify,
  setActiveSection,
  setBaseState,
  setConfig,
  setCustomPresetPage,
  setActiveCustomPresetIndex,
  setAppliedCustomPresetIndex,
  setCustomRulesDialogOpen,
}: UseProfileActionsArgs) {
  async function persistPayload(payload: AppStatePayload, syncConfig: boolean) {
    await saveAppState(payload);
    setBaseState(payload);
    if (syncConfig) setConfig(payload.config);
  }

  async function persist(nextConfig: ClientConfig, syncConfig = true) {
    await persistPayload(buildPayload(baseState, nextConfig), syncConfig);
  }

  async function saveProfile(successTitle: string, successDetail: string) {
    const normalized = normalizeClientConfig(config);
    const regexError = validateRuleRegex(normalized);
    if (regexError) {
      notify("error", "Rules not saved", regexError);
      return;
    }
    const artworkPublishingError = validateArtworkPublishing(normalized);
    if (artworkPublishingError) {
      setActiveSection("settings");
      notify("warn", "Artwork publishing required", artworkPublishingError);
      return;
    }
    try {
      if (capabilities.autostart && normalized.launchOnStartup !== baseState.config.launchOnStartup) {
        await setAutostartEnabled(normalized.launchOnStartup);
      }
      await persist(normalized);
      notify("success", successTitle, successDetail);
    } catch (error) {
      notify(
        "error",
        "Save failed",
        error instanceof Error ? error.message : "The current settings could not be saved.",
      );
    }
  }

  function discardDraftChanges() {
    setConfig(normalizeClientConfig(baseState.config));
    notify("info", "Draft reverted", "The current form was reset to the last saved settings.");
  }

  function saveCurrentCustomSettingsAsPreset() {
    const nextPreset = createDiscordCustomPresetFromConfig(config);
    const targetPreset =
      appliedCustomPresetIndex === null ? null : config.discordCustomPresets[appliedCustomPresetIndex] ?? null;

    if (targetPreset && appliedCustomPresetIndex !== null) {
      setConfig((current) => ({
        ...current,
        discordCustomPresets: current.discordCustomPresets.map((preset, index) =>
          index === appliedCustomPresetIndex ? { ...nextPreset, name: preset.name } : preset,
        ),
      }));
      setCustomPresetPage(pageForIndex(appliedCustomPresetIndex, CUSTOM_PRESET_PAGE_SIZE));
      setActiveCustomPresetIndex(appliedCustomPresetIndex);
      setAppliedCustomPresetIndex(appliedCustomPresetIndex);
      setCustomRulesDialogOpen(true);
      return;
    }

    const nextIndex = config.discordCustomPresets.length;
    setConfig((current) => ({
      ...current,
      discordCustomPresets: [...current.discordCustomPresets, nextPreset],
    }));
    setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
    setActiveCustomPresetIndex(nextIndex);
    setAppliedCustomPresetIndex(nextIndex);
    setCustomRulesDialogOpen(true);
  }

  function applyDiscordCustomPreset(preset: DiscordCustomPreset, presetIndex?: number) {
    setConfig((current) => ({
      ...current,
      discordReportMode: "custom",
      discordActivityType: preset.activityType,
      discordCustomModeStatusDisplay: preset.statusDisplay,
      discordCustomModeAppNameMode: preset.appNameMode,
      discordCustomModeCustomAppName: preset.customAppName,
      discordCustomArtworkSource: preset.customArtworkSource,
      discordCustomArtworkTextMode: preset.customArtworkTextMode,
      discordCustomArtworkText: preset.customArtworkText,
      discordCustomArtworkAssetId: preset.customArtworkAssetId,
      discordCustomAppIconSource: preset.customAppIconSource,
      discordCustomAppIconTextMode: preset.customAppIconTextMode,
      discordCustomAppIconText: preset.customAppIconText,
      discordCustomAppIconAssetId: preset.customAppIconAssetId,
      discordDetailsFormat: normalizeDiscordLineTemplate(preset.detailsFormat),
      discordStateFormat: normalizeDiscordLineTemplate(preset.stateFormat),
      discordCustomButtons: preset.buttons.map((button) => ({ ...button })),
      discordCustomPartyId: preset.partyId,
      discordCustomPartySizeCurrent: preset.partySizeCurrent ?? null,
      discordCustomPartySizeMax: preset.partySizeMax ?? null,
      discordCustomJoinSecret: preset.joinSecret,
      discordCustomSpectateSecret: preset.spectateSecret,
      discordCustomMatchSecret: preset.matchSecret,
    }));
    setAppliedCustomPresetIndex(typeof presetIndex === "number" ? presetIndex : null);
  }

  return {
    persistPayload,
    saveProfile,
    discardDraftChanges,
    saveCurrentCustomSettingsAsPreset,
    applyDiscordCustomPreset,
  };
}
