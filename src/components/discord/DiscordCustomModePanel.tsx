import type {
  DiscordActivityType,
  DiscordAssetTextMode,
  DiscordCustomAppIconSource,
  DiscordCustomArtworkSource,
  DiscordCustomAsset,
  DiscordRichPresenceButtonConfig,
} from "../../types";
import { DiscordOptionHelp } from "./DiscordOptionHelp";
import { DiscordAddonsEditor } from "./DiscordAddonsEditor";
import { DiscordCustomArtworkEditor } from "./DiscordCustomArtworkEditor";
import { DiscordLineTemplateEditor } from "./DiscordLineTemplateEditor";
import { DISCORD_ACTIVITY_TYPE_OPTIONS } from "./discordOptions";

function presetCountLabel(count: number) {
  return `${count} preset${count === 1 ? "" : "s"}`;
}

export function DiscordCustomModePanel({
  activityType,
  detailsFormat,
  stateFormat,
  detailsForceCustomChoice,
  stateForceCustomChoice,
  assets,
  artworkSource,
  artworkTextMode,
  artworkText,
  artworkAssetId,
  appIconSource,
  appIconTextMode,
  appIconText,
  appIconAssetId,
  buttons,
  customAdvancedAddonsConfigured,
  presetCount,
  appliedPresetName,
  partyId,
  partySizeCurrent,
  partySizeMax,
  joinSecret,
  spectateSecret,
  matchSecret,
  panelHeadClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  selectClass,
  buttonClass,
  primaryButtonClass,
  dangerButtonClass,
  radioCardClass,
  activeRadioCardClass,
  discordActivityTypeText,
  onActivityTypeChange,
  setDetailsForceCustomChoice,
  setStateForceCustomChoice,
  onDetailsFormatChange,
  onStateFormatChange,
  onArtworkSourceChange,
  onArtworkTextModeChange,
  onArtworkTextChange,
  onArtworkAssetIdChange,
  onAppIconSourceChange,
  onAppIconTextModeChange,
  onAppIconTextChange,
  onAppIconAssetIdChange,
  onPatchButtonAt,
  onRemoveButtonAt,
  onAddButton,
  onPartyIdChange,
  onPartySizeCurrentChange,
  onPartySizeMaxChange,
  onJoinSecretChange,
  onSpectateSecretChange,
  onMatchSecretChange,
  onSaveCurrentAsPreset,
  onOpenPresets,
}: {
  activityType: DiscordActivityType;
  detailsFormat: string;
  stateFormat: string;
  detailsForceCustomChoice: boolean;
  stateForceCustomChoice: boolean;
  assets: DiscordCustomAsset[];
  artworkSource: DiscordCustomArtworkSource;
  artworkTextMode: DiscordAssetTextMode;
  artworkText: string;
  artworkAssetId: string;
  appIconSource: DiscordCustomAppIconSource;
  appIconTextMode: DiscordAssetTextMode;
  appIconText: string;
  appIconAssetId: string;
  buttons: DiscordRichPresenceButtonConfig[];
  customAdvancedAddonsConfigured: boolean;
  presetCount: number;
  appliedPresetName: string | null;
  partyId: string;
  partySizeCurrent: number | null;
  partySizeMax: number | null;
  joinSecret: string;
  spectateSecret: string;
  matchSecret: string;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  radioCardClass: string;
  activeRadioCardClass: string;
  discordActivityTypeText: (value: DiscordActivityType) => string;
  onActivityTypeChange: (value: DiscordActivityType) => void;
  setDetailsForceCustomChoice: (value: boolean) => void;
  setStateForceCustomChoice: (value: boolean) => void;
  onDetailsFormatChange: (value: string) => void;
  onStateFormatChange: (value: string) => void;
  onArtworkSourceChange: (value: DiscordCustomArtworkSource) => void;
  onArtworkTextModeChange: (value: DiscordAssetTextMode) => void;
  onArtworkTextChange: (value: string) => void;
  onArtworkAssetIdChange: (value: string) => void;
  onAppIconSourceChange: (value: DiscordCustomAppIconSource) => void;
  onAppIconTextModeChange: (value: DiscordAssetTextMode) => void;
  onAppIconTextChange: (value: string) => void;
  onAppIconAssetIdChange: (value: string) => void;
  onPatchButtonAt: (
    index: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) => void;
  onRemoveButtonAt: (index: number) => void;
  onAddButton: () => void;
  onPartyIdChange: (value: string) => void;
  onPartySizeCurrentChange: (value: string) => void;
  onPartySizeMaxChange: (value: string) => void;
  onJoinSecretChange: (value: string) => void;
  onSpectateSecretChange: (value: string) => void;
  onMatchSecretChange: (value: string) => void;
  onSaveCurrentAsPreset: () => void;
  onOpenPresets: () => void;
}) {
  const appliedPresetButtonLabel = appliedPresetName
    ? "Update applied preset"
    : "Save current as preset";

  return (
    <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
      <div className="list-editor-summary">
        <div className="list-editor-copy">
          <strong className="block font-semibold">Custom mode</strong>
          <p>Choose the activity label and pick each Discord line from a short list of built-in options.</p>
        </div>
        <div className="card-actions gap-2">
          <span className="badge badge-soft">{discordActivityTypeText(activityType)}</span>
          <DiscordOptionHelp idPrefix="custom-mode-help" />
        </div>
      </div>

      <div className="radio-grid discord-activity-grid">
        {DISCORD_ACTIVITY_TYPE_OPTIONS.map((option) => (
          <label key={option.value} className={activityType === option.value ? activeRadioCardClass : radioCardClass}>
            <input
              className="radio radio-primary mt-1"
              type="radio"
              name="discordActivityType"
              checked={activityType === option.value}
              onChange={() => onActivityTypeChange(option.value)}
            />
            <div>
              <strong>{option.label}</strong>
              <span>{option.helper}</span>
            </div>
          </label>
        ))}
      </div>

      <div className="field-grid">
        <DiscordLineTemplateEditor
          label="Line 2"
          value={detailsFormat}
          forceCustomChoice={detailsForceCustomChoice}
          placeholder="Coding in {app}"
          optionKeyPrefix="details"
          fieldSpanClass={fieldSpanClass}
          selectClass={selectClass}
          inputClass={inputClass}
          setForceCustomChoice={setDetailsForceCustomChoice}
          onValueChange={onDetailsFormatChange}
        />
        <DiscordLineTemplateEditor
          label="Line 3"
          value={stateFormat}
          forceCustomChoice={stateForceCustomChoice}
          placeholder="With {artist}"
          optionKeyPrefix="state"
          fieldSpanClass={fieldSpanClass}
          selectClass={selectClass}
          inputClass={inputClass}
          setForceCustomChoice={setStateForceCustomChoice}
          onValueChange={onStateFormatChange}
        />
      </div>

      <DiscordCustomArtworkEditor
        assets={assets}
        artworkSource={artworkSource}
        artworkTextMode={artworkTextMode}
        artworkText={artworkText}
        artworkAssetId={artworkAssetId}
        appIconSource={appIconSource}
        appIconTextMode={appIconTextMode}
        appIconText={appIconText}
        appIconAssetId={appIconAssetId}
        panelHeadClass={panelHeadClass}
        fieldClass={fieldClass}
        fieldSpanClass={fieldSpanClass}
        inputClass={inputClass}
        selectClass={selectClass}
        buttonClass={buttonClass}
        primaryButtonClass={primaryButtonClass}
        onArtworkSourceChange={onArtworkSourceChange}
        onArtworkTextModeChange={onArtworkTextModeChange}
        onArtworkTextChange={onArtworkTextChange}
        onArtworkAssetIdChange={onArtworkAssetIdChange}
        onAppIconSourceChange={onAppIconSourceChange}
        onAppIconTextModeChange={onAppIconTextModeChange}
        onAppIconTextChange={onAppIconTextChange}
        onAppIconAssetIdChange={onAppIconAssetIdChange}
      />

      <DiscordAddonsEditor
        title="Custom add-ons"
        description="URL buttons stay visible here. Party and secrets live under Advanced."
        buttons={buttons}
        advancedConfigured={customAdvancedAddonsConfigured}
        panelHeadClass={panelHeadClass}
        fieldClass={fieldClass}
        fieldSpanClass={fieldSpanClass}
        inputClass={inputClass}
        buttonClass={buttonClass}
        dangerButtonClass={dangerButtonClass}
        onPatchButtonAt={onPatchButtonAt}
        onRemoveButtonAt={onRemoveButtonAt}
        onAddButton={onAddButton}
        partyId={partyId}
        partySizeCurrent={partySizeCurrent}
        partySizeMax={partySizeMax}
        joinSecret={joinSecret}
        spectateSecret={spectateSecret}
        matchSecret={matchSecret}
        onPartyIdChange={onPartyIdChange}
        onPartySizeCurrentChange={onPartySizeCurrentChange}
        onPartySizeMaxChange={onPartySizeMaxChange}
        onJoinSecretChange={onJoinSecretChange}
        onSpectateSecretChange={onSpectateSecretChange}
        onMatchSecretChange={onMatchSecretChange}
      />

      <div className="rounded-box border border-base-300 bg-base-100 p-4">
        <div className={panelHeadClass}>
          <div>
            <strong className="block font-semibold">Custom presets</strong>
            <p className="mt-1 text-sm text-base-content/70">
              {appliedPresetName
                ? `Applied preset: ${appliedPresetName}`
                : "Save ready-to-use Custom profiles for one-click selection and import."}
            </p>
          </div>
          <div className="card-actions gap-2">
            <span className="badge badge-soft">{presetCountLabel(presetCount)}</span>
            <button
              className={primaryButtonClass}
              type="button"
              title={appliedPresetName ? `Update applied preset: ${appliedPresetName}` : undefined}
              onClick={onSaveCurrentAsPreset}
            >
              {appliedPresetButtonLabel}
            </button>
            <button className={buttonClass} type="button" onClick={onOpenPresets}>
              Open presets
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
