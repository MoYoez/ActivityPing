import type {
  DiscordActivityType,
  DiscordAppNameMode,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
  DiscordStatusDisplay,
} from "../../types";
import { DiscordAddonsEditor } from "./DiscordAddonsEditor";
import { DiscordLineTemplateEditor } from "./DiscordLineTemplateEditor";
import { DiscordOptionHelp } from "./DiscordOptionHelp";
import {
  DISCORD_ACTIVITY_TYPE_OPTIONS,
  DISCORD_APP_NAME_OPTIONS,
  DISCORD_STATUS_DISPLAY_OPTIONS,
} from "./discordOptions";

export function DiscordCustomPresetEditorModal({
  preset,
  presetIndex,
  detailsForceCustomChoice,
  stateForceCustomChoice,
  presetAdvancedAddonsConfigured,
  panelHeadClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  selectClass,
  buttonClass,
  primaryButtonClass,
  dangerButtonClass,
  onClose,
  onUsePreset,
  onNameChange,
  onActivityTypeChange,
  onStatusDisplayChange,
  onAppNameModeChange,
  onCustomAppNameChange,
  setDetailsForceCustomChoice,
  setStateForceCustomChoice,
  onDetailsFormatChange,
  onStateFormatChange,
  onPatchButtonAt,
  onRemoveButtonAt,
  onAddButton,
  onPartyIdChange,
  onPartySizeCurrentChange,
  onPartySizeMaxChange,
  onJoinSecretChange,
  onSpectateSecretChange,
  onMatchSecretChange,
}: {
  preset: DiscordCustomPreset;
  presetIndex: number;
  detailsForceCustomChoice: boolean;
  stateForceCustomChoice: boolean;
  presetAdvancedAddonsConfigured: boolean;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  onClose: () => void;
  onUsePreset: () => void;
  onNameChange: (value: string) => void;
  onActivityTypeChange: (value: DiscordActivityType) => void;
  onStatusDisplayChange: (value: DiscordStatusDisplay) => void;
  onAppNameModeChange: (value: DiscordAppNameMode) => void;
  onCustomAppNameChange: (value: string) => void;
  setDetailsForceCustomChoice: (value: boolean) => void;
  setStateForceCustomChoice: (value: boolean) => void;
  onDetailsFormatChange: (value: string) => void;
  onStateFormatChange: (value: string) => void;
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
}) {
  return (
    <section className="modal modal-open">
      <div
        className="modal-box w-11/12 max-w-4xl p-0"
        role="dialog"
        aria-modal="true"
        aria-labelledby="custom-preset-editor-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="card-body">
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">Custom preset</p>
              <h3 id="custom-preset-editor-title" className="card-title">
                {preset.name.trim() || `Custom preset ${presetIndex + 1}`}
              </h3>
              <p>Edit the preset fields that will be imported into Custom mode output.</p>
            </div>
            <div className="card-actions gap-2">
              <DiscordOptionHelp idPrefix="custom-preset-help" includeSmartModeNote={false} />
              <button className={primaryButtonClass} type="button" onClick={onUsePreset}>
                Use preset
              </button>
              <button className={buttonClass} type="button" onClick={onClose}>
                Close
              </button>
            </div>
          </div>

          <div className="field-grid compact-fields">
            <label className={fieldClass}>
              <span>Name</span>
              <input
                className={inputClass}
                value={preset.name}
                onChange={(e) => onNameChange(e.currentTarget.value)}
                placeholder="Work profile"
              />
            </label>
            <label className={fieldClass}>
              <span>Activity label</span>
              <select
                className={selectClass}
                value={preset.activityType}
                onChange={(e) => onActivityTypeChange(e.currentTarget.value as DiscordActivityType)}
              >
                {DISCORD_ACTIVITY_TYPE_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label className={fieldClass}>
              <span>Compact status</span>
              <select
                className={selectClass}
                value={preset.statusDisplay}
                onChange={(e) => onStatusDisplayChange(e.currentTarget.value as DiscordStatusDisplay)}
              >
                {DISCORD_STATUS_DISPLAY_OPTIONS.map((option) => (
                  <option key={`preset-status-${option.value}`} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label className={fieldClass}>
              <span>Application name source</span>
              <select
                className={selectClass}
                value={preset.appNameMode}
                onChange={(e) => onAppNameModeChange(e.currentTarget.value as DiscordAppNameMode)}
              >
                {DISCORD_APP_NAME_OPTIONS.map((option) => (
                  <option key={`preset-app-name-${option.value}`} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            {preset.appNameMode === "custom" ? (
              <label className={fieldSpanClass}>
                <span>Custom application text</span>
                <input
                  className={inputClass}
                  value={preset.customAppName}
                  onChange={(e) => onCustomAppNameChange(e.currentTarget.value)}
                  placeholder="Your custom application name"
                />
              </label>
            ) : null}
            <DiscordLineTemplateEditor
              label="Preset Line 2"
              value={preset.detailsFormat ?? ""}
              forceCustomChoice={detailsForceCustomChoice}
              placeholder="Coding in {app}"
              optionKeyPrefix="preset-details"
              fieldSpanClass={fieldSpanClass}
              selectClass={selectClass}
              inputClass={inputClass}
              setForceCustomChoice={setDetailsForceCustomChoice}
              onValueChange={onDetailsFormatChange}
            />
            <DiscordLineTemplateEditor
              label="Preset Line 3"
              value={preset.stateFormat ?? ""}
              forceCustomChoice={stateForceCustomChoice}
              placeholder="With {artist}"
              optionKeyPrefix="preset-state"
              fieldSpanClass={fieldSpanClass}
              selectClass={selectClass}
              inputClass={inputClass}
              setForceCustomChoice={setStateForceCustomChoice}
              onValueChange={onStateFormatChange}
            />
          </div>

          <DiscordAddonsEditor
            title="Preset add-ons"
            description="Buttons, party metadata, and social secrets saved here will be imported with this preset."
            buttons={preset.buttons}
            advancedConfigured={presetAdvancedAddonsConfigured}
            panelHeadClass={panelHeadClass}
            fieldClass={fieldClass}
            fieldSpanClass={fieldSpanClass}
            inputClass={inputClass}
            buttonClass={buttonClass}
            dangerButtonClass={dangerButtonClass}
            onPatchButtonAt={onPatchButtonAt}
            onRemoveButtonAt={onRemoveButtonAt}
            onAddButton={onAddButton}
            partyId={preset.partyId}
            partySizeCurrent={preset.partySizeCurrent ?? null}
            partySizeMax={preset.partySizeMax ?? null}
            joinSecret={preset.joinSecret}
            spectateSecret={preset.spectateSecret}
            matchSecret={preset.matchSecret}
            onPartyIdChange={onPartyIdChange}
            onPartySizeCurrentChange={onPartySizeCurrentChange}
            onPartySizeMaxChange={onPartySizeMaxChange}
            onJoinSecretChange={onJoinSecretChange}
            onSpectateSecretChange={onSpectateSecretChange}
            onMatchSecretChange={onMatchSecretChange}
          />
        </div>
      </div>
    </section>
  );
}
