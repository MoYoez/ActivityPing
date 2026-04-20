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
import { DiscordArtworkPublishingPanel } from "./DiscordArtworkPublishingPanel";
import { DiscordCustomModePanel } from "./DiscordCustomModePanel";
import { DiscordModeSettingsPanels } from "./DiscordModeSettingsPanels";
import {
  DISCORD_REPORT_MODE_OPTIONS,
  DISCORD_SMART_ARTWORK_PREFERENCE_OPTIONS,
} from "./discordOptions";

export function DiscordBridgeView({
  config,
  discordConnected,
  activeDiscordModeName,
  activeDiscordStatusDisplay,
  activeDiscordAppNameMode,
  activeDiscordCustomAppName,
  customAppNameEnabled,
  customDiscordMode,
  customAdvancedAddonsConfigured,
  discordDetailsForceCustomChoice,
  discordStateForceCustomChoice,
  artworkPublishingMissing,
  panelClass,
  panelHeadClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  selectClass,
  buttonClass,
  primaryButtonClass,
  dangerButtonClass,
  badgeClass,
  goodBadgeClass,
  statCardClass,
  toggleTileClass,
  radioCardClass,
  activeRadioCardClass,
  discordActivityTypeText,
  discordReportModeText,
  linkStateText,
  currentSummaryText,
  lastErrorText,
  onDiscordApplicationIdChange,
  onDiscordReportModeChange,
  onDiscordModeSettingsChange,
  onDiscordActivityTypeChange,
  onDiscordDetailsForceCustomChoiceChange,
  onDiscordStateForceCustomChoiceChange,
  onDiscordDetailsFormatChange,
  onDiscordStateFormatChange,
  onDiscordCustomArtworkSourceChange,
  onDiscordCustomArtworkTextModeChange,
  onDiscordCustomArtworkTextChange,
  onDiscordCustomArtworkAssetIdChange,
  onDiscordCustomAppIconSourceChange,
  onDiscordCustomAppIconTextModeChange,
  onDiscordCustomAppIconTextChange,
  onDiscordCustomAppIconAssetIdChange,
  onPatchDiscordButtonAt,
  onRemoveDiscordButtonAt,
  onAddDiscordButton,
  onDiscordCustomPartyIdChange,
  onDiscordCustomPartySizeCurrentChange,
  onDiscordCustomPartySizeMaxChange,
  onDiscordCustomJoinSecretChange,
  onDiscordCustomSpectateSecretChange,
  onDiscordCustomMatchSecretChange,
  onSaveCurrentCustomSettingsAsPreset,
  onOpenCustomPresets,
  onDiscordSmartEnableMusicCountdownChange,
  onDiscordSmartShowAppNameChange,
  onDiscordSmartArtworkPreferenceChange,
  onReportStoppedMediaChange,
  onDiscordUseAppArtworkChange,
  onDiscordUseMusicArtworkChange,
  onDiscordArtworkWorkerUploadUrlChange,
  onDiscordArtworkWorkerTokenChange,
}: {
  config: ClientConfig;
  discordConnected: boolean;
  activeDiscordModeName: string;
  activeDiscordStatusDisplay: DiscordStatusDisplay;
  activeDiscordAppNameMode: DiscordAppNameMode;
  activeDiscordCustomAppName: string;
  customAppNameEnabled: boolean;
  customDiscordMode: boolean;
  customAdvancedAddonsConfigured: boolean;
  discordDetailsForceCustomChoice: boolean;
  discordStateForceCustomChoice: boolean;
  artworkPublishingMissing: boolean;
  panelClass: string;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  badgeClass: string;
  goodBadgeClass: string;
  statCardClass: string;
  toggleTileClass: string;
  radioCardClass: string;
  activeRadioCardClass: string;
  discordActivityTypeText: (value: DiscordActivityType) => string;
  discordReportModeText: (config: ClientConfig) => string;
  linkStateText: string;
  currentSummaryText: string;
  lastErrorText: string;
  onDiscordApplicationIdChange: (value: string) => void;
  onDiscordReportModeChange: (value: DiscordReportMode) => void;
  onDiscordModeSettingsChange: (patch: {
    statusDisplay?: DiscordStatusDisplay;
    appNameMode?: DiscordAppNameMode;
    customAppName?: string;
  }) => void;
  onDiscordActivityTypeChange: (value: DiscordActivityType) => void;
  onDiscordDetailsForceCustomChoiceChange: (value: boolean) => void;
  onDiscordStateForceCustomChoiceChange: (value: boolean) => void;
  onDiscordDetailsFormatChange: (value: string) => void;
  onDiscordStateFormatChange: (value: string) => void;
  onDiscordCustomArtworkSourceChange: (value: DiscordCustomArtworkSource) => void;
  onDiscordCustomArtworkTextModeChange: (value: DiscordAssetTextMode) => void;
  onDiscordCustomArtworkTextChange: (value: string) => void;
  onDiscordCustomArtworkAssetIdChange: (value: string) => void;
  onDiscordCustomAppIconSourceChange: (value: DiscordCustomAppIconSource) => void;
  onDiscordCustomAppIconTextModeChange: (value: DiscordAssetTextMode) => void;
  onDiscordCustomAppIconTextChange: (value: string) => void;
  onDiscordCustomAppIconAssetIdChange: (value: string) => void;
  onPatchDiscordButtonAt: (index: number, updater: (button: { label: string; url: string }) => { label: string; url: string }) => void;
  onRemoveDiscordButtonAt: (index: number) => void;
  onAddDiscordButton: () => void;
  onDiscordCustomPartyIdChange: (value: string) => void;
  onDiscordCustomPartySizeCurrentChange: (value: string) => void;
  onDiscordCustomPartySizeMaxChange: (value: string) => void;
  onDiscordCustomJoinSecretChange: (value: string) => void;
  onDiscordCustomSpectateSecretChange: (value: string) => void;
  onDiscordCustomMatchSecretChange: (value: string) => void;
  onSaveCurrentCustomSettingsAsPreset: () => void;
  onOpenCustomPresets: () => void;
  onDiscordSmartEnableMusicCountdownChange: (value: boolean) => void;
  onDiscordSmartShowAppNameChange: (value: boolean) => void;
  onDiscordSmartArtworkPreferenceChange: (value: DiscordSmartArtworkPreference) => void;
  onReportStoppedMediaChange: (value: boolean) => void;
  onDiscordUseAppArtworkChange: (value: boolean) => void;
  onDiscordUseMusicArtworkChange: (value: boolean) => void;
  onDiscordArtworkWorkerUploadUrlChange: (value: string) => void;
  onDiscordArtworkWorkerTokenChange: (value: string) => void;
}) {
  return (
    <>
      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">Bridge</p>
            <h3>Discord RPC settings</h3>
          </div>
          <span className={discordConnected ? goodBadgeClass : badgeClass}>
            {discordConnected ? "Connected" : "Idle"}
          </span>
        </div>
        <div className="field-grid">
          <label className={fieldSpanClass}>
            <span>Discord application ID</span>
            <input
              className={inputClass}
              value={config.discordApplicationId}
              onChange={(e) => onDiscordApplicationIdChange(e.currentTarget.value)}
              placeholder="Your Discord application ID"
            />
          </label>
        </div>
        <div className="radio-grid discord-mode-grid">
          {DISCORD_REPORT_MODE_OPTIONS.map((option) => (
            <label key={option.mode} className={config.discordReportMode === option.mode ? activeRadioCardClass : radioCardClass}>
              <input
                className="radio radio-primary mt-1"
                type="radio"
                name="discordReportMode"
                checked={config.discordReportMode === option.mode}
                onChange={() => onDiscordReportModeChange(option.mode)}
              />
              <div>
                <strong>{option.title}</strong>
                <span>{option.description}</span>
                <div className="discord-mode-layout">
                  <span>Details: {option.details}</span>
                  <span>State / Summary: {option.state}</span>
                </div>
              </div>
            </label>
          ))}
        </div>
        <DiscordModeSettingsPanels
          activeDiscordModeName={activeDiscordModeName}
          activeDiscordStatusDisplay={activeDiscordStatusDisplay}
          activeDiscordAppNameMode={activeDiscordAppNameMode}
          activeDiscordCustomAppName={activeDiscordCustomAppName}
          customAppNameEnabled={customAppNameEnabled}
          fieldClass={fieldClass}
          fieldSpanClass={fieldSpanClass}
          inputClass={inputClass}
          selectClass={selectClass}
          onStatusDisplayChange={(value) => onDiscordModeSettingsChange({ statusDisplay: value })}
          onAppNameModeChange={(value) => onDiscordModeSettingsChange({ appNameMode: value })}
          onCustomAppNameChange={(value) => onDiscordModeSettingsChange({ customAppName: value })}
        />
        {customDiscordMode ? (
          <DiscordCustomModePanel
            activityType={config.discordActivityType}
            detailsFormat={config.discordDetailsFormat}
            stateFormat={config.discordStateFormat}
            detailsForceCustomChoice={discordDetailsForceCustomChoice}
            stateForceCustomChoice={discordStateForceCustomChoice}
            assets={config.discordCustomAssets}
            artworkSource={config.discordCustomArtworkSource}
            artworkTextMode={config.discordCustomArtworkTextMode}
            artworkText={config.discordCustomArtworkText}
            artworkAssetId={config.discordCustomArtworkAssetId}
            appIconSource={config.discordCustomAppIconSource}
            appIconTextMode={config.discordCustomAppIconTextMode}
            appIconText={config.discordCustomAppIconText}
            appIconAssetId={config.discordCustomAppIconAssetId}
            buttons={config.discordCustomButtons}
            customAdvancedAddonsConfigured={customAdvancedAddonsConfigured}
            presetCount={config.discordCustomPresets.length}
            partyId={config.discordCustomPartyId}
            partySizeCurrent={config.discordCustomPartySizeCurrent ?? null}
            partySizeMax={config.discordCustomPartySizeMax ?? null}
            joinSecret={config.discordCustomJoinSecret}
            spectateSecret={config.discordCustomSpectateSecret}
            matchSecret={config.discordCustomMatchSecret}
            panelHeadClass={panelHeadClass}
            fieldClass={fieldClass}
            fieldSpanClass={fieldSpanClass}
            inputClass={inputClass}
            selectClass={selectClass}
            buttonClass={buttonClass}
            primaryButtonClass={primaryButtonClass}
            dangerButtonClass={dangerButtonClass}
            radioCardClass={radioCardClass}
            activeRadioCardClass={activeRadioCardClass}
            discordActivityTypeText={discordActivityTypeText}
            onActivityTypeChange={onDiscordActivityTypeChange}
            setDetailsForceCustomChoice={onDiscordDetailsForceCustomChoiceChange}
            setStateForceCustomChoice={onDiscordStateForceCustomChoiceChange}
            onDetailsFormatChange={onDiscordDetailsFormatChange}
            onStateFormatChange={onDiscordStateFormatChange}
            onArtworkSourceChange={onDiscordCustomArtworkSourceChange}
            onArtworkTextModeChange={onDiscordCustomArtworkTextModeChange}
            onArtworkTextChange={onDiscordCustomArtworkTextChange}
            onArtworkAssetIdChange={onDiscordCustomArtworkAssetIdChange}
            onAppIconSourceChange={onDiscordCustomAppIconSourceChange}
            onAppIconTextModeChange={onDiscordCustomAppIconTextModeChange}
            onAppIconTextChange={onDiscordCustomAppIconTextChange}
            onAppIconAssetIdChange={onDiscordCustomAppIconAssetIdChange}
            onPatchButtonAt={onPatchDiscordButtonAt}
            onRemoveButtonAt={onRemoveDiscordButtonAt}
            onAddButton={onAddDiscordButton}
            onPartyIdChange={onDiscordCustomPartyIdChange}
            onPartySizeCurrentChange={onDiscordCustomPartySizeCurrentChange}
            onPartySizeMaxChange={onDiscordCustomPartySizeMaxChange}
            onJoinSecretChange={onDiscordCustomJoinSecretChange}
            onSpectateSecretChange={onDiscordCustomSpectateSecretChange}
            onMatchSecretChange={onDiscordCustomMatchSecretChange}
            onSaveCurrentAsPreset={onSaveCurrentCustomSettingsAsPreset}
            onOpenPresets={onOpenCustomPresets}
          />
        ) : null}
        <div className="toggle-grid compact-toggles">
          {config.discordReportMode === "mixed" ? (
            <label className={toggleTileClass}>
              <div>
                <strong>Enable music countdown in Smart mode</strong>
                <span>Keep Discord's song timer in sync while Smart mode is tracking active media.</span>
              </div>
              <input
                className="toggle toggle-primary"
                type="checkbox"
                checked={config.discordSmartEnableMusicCountdown}
                onChange={(e) => onDiscordSmartEnableMusicCountdownChange(e.currentTarget.checked)}
              />
            </label>
          ) : null}
          {config.discordReportMode === "mixed" ? (
            <label className={toggleTileClass}>
              <div>
                <strong>Smart artwork priority</strong>
                <span>Choose whether Smart mode uses music artwork or the current foreground app as the main image.</span>
              </div>
              <select
                className={selectClass}
                value={config.discordSmartArtworkPreference}
                onChange={(e) => onDiscordSmartArtworkPreferenceChange(e.currentTarget.value as DiscordSmartArtworkPreference)}
              >
                {DISCORD_SMART_ARTWORK_PREFERENCE_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
          {config.discordReportMode === "mixed" ? (
            <label className={toggleTileClass}>
              <div>
                <strong>Show app name in Smart mode</strong>
                <span>Show the current foreground app on line 2 in Smart mode when an app is active.</span>
              </div>
              <input
                className="toggle toggle-primary"
                type="checkbox"
                checked={config.discordSmartShowAppName}
                onChange={(e) => onDiscordSmartShowAppNameChange(e.currentTarget.checked)}
              />
            </label>
          ) : null}
          <label className={toggleTileClass}>
            <div>
              <strong>Report paused media</strong>
              <span>Keep the latest media visible after playback pauses or stops, with a frozen Discord song timer.</span>
            </div>
            <input
              className="toggle toggle-primary"
              type="checkbox"
              checked={config.reportStoppedMedia}
              onChange={(e) => onReportStoppedMediaChange(e.currentTarget.checked)}
            />
          </label>
          <label className={toggleTileClass}>
            <div>
              <strong>Use app artwork</strong>
              <span>Upload the current foreground app icon for Rich Presence. Custom mode can override this above with its own asset selectors.</span>
            </div>
            <input
              className="toggle toggle-primary"
              type="checkbox"
              checked={config.discordUseAppArtwork}
              onChange={(e) => onDiscordUseAppArtworkChange(e.currentTarget.checked)}
            />
          </label>
          <label className={toggleTileClass}>
            <div>
              <strong>Use music artwork</strong>
              <span>Upload media cover art when available and keep the playback source icon attached while music is active. Custom mode can override this above with its own asset selectors.</span>
            </div>
            <input
              className="toggle toggle-primary"
              type="checkbox"
              checked={config.discordUseMusicArtwork}
              onChange={(e) => onDiscordUseMusicArtworkChange(e.currentTarget.checked)}
            />
          </label>
        </div>
        {config.discordUseAppArtwork || config.discordUseMusicArtwork ? (
          <DiscordArtworkPublishingPanel
            artworkPublishingMissing={artworkPublishingMissing}
            uploadUrl={config.discordArtworkWorkerUploadUrl}
            token={config.discordArtworkWorkerToken}
            fieldSpanClass={fieldSpanClass}
            inputClass={inputClass}
            onUploadUrlChange={onDiscordArtworkWorkerUploadUrlChange}
            onTokenChange={onDiscordArtworkWorkerTokenChange}
          />
        ) : null}
      </section>

      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">State</p>
            <h3>Current bridge status</h3>
          </div>
        </div>
        <div className="stat-grid">
          <div className={statCardClass}>
            <span>Link state</span>
            <strong>{linkStateText}</strong>
          </div>
          <div className={statCardClass}>
            <span>Current summary</span>
            <strong>{currentSummaryText}</strong>
          </div>
          <div className={statCardClass}>
            <span>Report mode</span>
            <strong>{customDiscordMode ? `${discordReportModeText(config)} · ${discordActivityTypeText(config.discordActivityType)}` : discordReportModeText(config)}</strong>
          </div>
          <div className={statCardClass}>
            <span>Last error</span>
            <strong>{lastErrorText}</strong>
          </div>
        </div>
      </section>
    </>
  );
}
