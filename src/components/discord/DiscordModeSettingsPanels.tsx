import type { DiscordAppNameMode, DiscordStatusDisplay } from "../../types";
import { DISCORD_APP_NAME_OPTIONS, DISCORD_STATUS_DISPLAY_OPTIONS } from "./discordOptions";

export function DiscordModeSettingsPanels({
  activeDiscordModeName,
  activeDiscordStatusDisplay,
  activeDiscordAppNameMode,
  activeDiscordCustomAppName,
  customAppNameEnabled,
  fieldClass,
  fieldSpanClass,
  inputClass,
  selectClass,
  onStatusDisplayChange,
  onAppNameModeChange,
  onCustomAppNameChange,
}: {
  activeDiscordModeName: string;
  activeDiscordStatusDisplay: DiscordStatusDisplay;
  activeDiscordAppNameMode: DiscordAppNameMode;
  activeDiscordCustomAppName: string;
  customAppNameEnabled: boolean;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  onStatusDisplayChange: (value: DiscordStatusDisplay) => void;
  onAppNameModeChange: (value: DiscordAppNameMode) => void;
  onCustomAppNameChange: (value: string) => void;
}) {
  return (
    <>
      <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
        <div className="list-editor-summary">
          <div className="list-editor-copy">
            <strong className="block font-semibold">Compact status</strong>
            <p>Controls Discord&apos;s compact member-list text only. This setting is saved separately for the current mode.</p>
          </div>
          <span className="badge badge-soft">{activeDiscordModeName} mode</span>
        </div>
        <div className="field-grid">
          <label className={fieldClass}>
            <span>Compact status</span>
            <select
              className={selectClass}
              value={activeDiscordStatusDisplay}
              onChange={(e) => onStatusDisplayChange(e.currentTarget.value as DiscordStatusDisplay)}
            >
              {DISCORD_STATUS_DISPLAY_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        </div>
      </div>

      <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
        <div className="list-editor-summary">
          <div className="list-editor-copy">
            <strong className="block font-semibold">Custom application name</strong>
            <p>This setting is saved separately for the current mode. Smart and App still keep their fixed app-first output while an app is active.</p>
          </div>
          <span className="badge badge-soft">{activeDiscordModeName} mode</span>
        </div>
        <div className="field-grid">
          <label className={fieldClass}>
            <span>Application name source</span>
            <select
              className={selectClass}
              value={activeDiscordAppNameMode}
              onChange={(e) => onAppNameModeChange(e.currentTarget.value as DiscordAppNameMode)}
            >
              {DISCORD_APP_NAME_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          {customAppNameEnabled ? (
            <label className={fieldSpanClass}>
              <span>Custom application text</span>
              <input
                className={inputClass}
                value={activeDiscordCustomAppName}
                onChange={(e) => onCustomAppNameChange(e.currentTarget.value)}
                placeholder="Your custom application name"
              />
            </label>
          ) : null}
        </div>
      </div>
    </>
  );
}
