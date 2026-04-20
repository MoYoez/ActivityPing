import type { ComponentProps } from "react";

import { DiscordBridgeView } from "../../components/discord/DiscordBridgeView";
import type { DiscordReportMode } from "../../types";
import { createDiscordButton } from "../appConfig";
import type { CreateSettingsViewPropsArgs } from "../createSettingsViewProps";

export function createDiscordBridgeProps(
  args: CreateSettingsViewPropsArgs,
): ComponentProps<typeof DiscordBridgeView> {
  return {
    config: args.config,
    discordConnected: args.discordConnected,
    activeDiscordModeName: args.activeDiscordModeName,
    activeDiscordStatusDisplay: args.activeDiscordStatusDisplay,
    activeDiscordAppNameMode: args.activeDiscordAppNameMode,
    activeDiscordCustomAppName: args.activeDiscordCustomAppName,
    customAppNameEnabled: args.customAppNameEnabled,
    customDiscordMode: args.customDiscordMode,
    customAdvancedAddonsConfigured: args.customAdvancedAddonsConfigured,
    discordDetailsForceCustomChoice: args.discordDetailsForceCustomChoice,
    discordStateForceCustomChoice: args.discordStateForceCustomChoice,
    artworkPublishingMissing: args.artworkPublishingMissing,
    panelClass: args.panelClass,
    panelHeadClass: args.panelHeadClass,
    fieldClass: args.fieldClass,
    fieldSpanClass: args.fieldSpanClass,
    inputClass: args.inputClass,
    selectClass: args.selectClass,
    buttonClass: args.buttonClass,
    primaryButtonClass: args.primaryButtonClass,
    dangerButtonClass: args.dangerButtonClass,
    badgeClass: args.badgeClass,
    goodBadgeClass: args.goodBadgeClass,
    statCardClass: args.statCardClass,
    toggleTileClass: args.toggleTileClass,
    radioCardClass: args.radioCardClass,
    activeRadioCardClass: args.activeRadioCardClass,
    discordActivityTypeText: args.discordActivityTypeText,
    discordReportModeText: args.discordReportModeText,
    linkStateText: args.discordRunning ? (args.discordConnected ? "Connected" : "Waiting for Discord") : "Stopped",
    currentSummaryText: args.discordCurrentSummary || "No local activity is being mirrored yet.",
    lastErrorText: args.discordLastError || "No Discord runtime error recorded.",
    onDiscordApplicationIdChange: (value) => args.update("discordApplicationId", value),
    onDiscordReportModeChange: (value: DiscordReportMode) => args.update("discordReportMode", value),
    onDiscordModeSettingsChange: args.updateDiscordModeSettings,
    onDiscordActivityTypeChange: (value) => args.update("discordActivityType", value),
    onDiscordDetailsForceCustomChoiceChange: args.setDiscordDetailsForceCustomChoice,
    onDiscordStateForceCustomChoiceChange: args.setDiscordStateForceCustomChoice,
    onDiscordDetailsFormatChange: (value) => args.update("discordDetailsFormat", value),
    onDiscordStateFormatChange: (value) => args.update("discordStateFormat", value),
    onDiscordCustomArtworkSourceChange: (value) => args.update("discordCustomArtworkSource", value),
    onDiscordCustomArtworkTextModeChange: (value) => args.update("discordCustomArtworkTextMode", value),
    onDiscordCustomArtworkTextChange: (value) => args.update("discordCustomArtworkText", value),
    onDiscordCustomArtworkAssetIdChange: (value) => args.update("discordCustomArtworkAssetId", value),
    onDiscordCustomAppIconSourceChange: (value) => args.update("discordCustomAppIconSource", value),
    onDiscordCustomAppIconTextModeChange: (value) => args.update("discordCustomAppIconTextMode", value),
    onDiscordCustomAppIconTextChange: (value) => args.update("discordCustomAppIconText", value),
    onDiscordCustomAppIconAssetIdChange: (value) => args.update("discordCustomAppIconAssetId", value),
    onPatchDiscordButtonAt: args.patchDiscordButtonAt,
    onRemoveDiscordButtonAt: (index) =>
      args.setConfig((current) => ({
        ...current,
        discordCustomButtons: current.discordCustomButtons.filter((_, itemIndex) => itemIndex !== index),
      })),
    onAddDiscordButton: () =>
      args.setConfig((current) => ({
        ...current,
        discordCustomButtons: [...current.discordCustomButtons, createDiscordButton()],
      })),
    onDiscordCustomPartyIdChange: (value) => args.update("discordCustomPartyId", value),
    onDiscordCustomPartySizeCurrentChange: (value) =>
      args.update("discordCustomPartySizeCurrent", args.normalizePositiveNumberInput(value)),
    onDiscordCustomPartySizeMaxChange: (value) =>
      args.update("discordCustomPartySizeMax", args.normalizePositiveNumberInput(value)),
    onDiscordCustomJoinSecretChange: (value) => args.update("discordCustomJoinSecret", value),
    onDiscordCustomSpectateSecretChange: (value) => args.update("discordCustomSpectateSecret", value),
    onDiscordCustomMatchSecretChange: (value) => args.update("discordCustomMatchSecret", value),
    onSaveCurrentCustomSettingsAsPreset: args.onSaveCurrentCustomSettingsAsPreset,
    onOpenCustomPresets: args.onOpenCustomPresets,
    onDiscordSmartEnableMusicCountdownChange: (value) => args.update("discordSmartEnableMusicCountdown", value),
    onDiscordSmartShowAppNameChange: (value) => args.update("discordSmartShowAppName", value),
    onDiscordSmartArtworkPreferenceChange: (value) => args.update("discordSmartArtworkPreference", value),
    onReportStoppedMediaChange: (value) => args.update("reportStoppedMedia", value),
    onDiscordUseAppArtworkChange: (value) => args.update("discordUseAppArtwork", value),
    onDiscordUseMusicArtworkChange: (value) => args.update("discordUseMusicArtwork", value),
    onDiscordArtworkWorkerUploadUrlChange: (value) => args.update("discordArtworkWorkerUploadUrl", value),
    onDiscordArtworkWorkerTokenChange: (value) => args.update("discordArtworkWorkerToken", value),
  };
}
