import { ListEditor } from "../ListEditor";
import { HistoryRecordPanels } from "../history/HistoryRecordPanels";
import type { AppFilterMode, AppHistoryEntry, PlaySourceHistoryEntry } from "../../types";

export function RuleSupportSections({
  appFilterMode,
  appBlacklist,
  appWhitelist,
  appNameOnlyList,
  mediaPlaySourceBlocklist,
  appSuggestions,
  playSourceSuggestions,
  blacklistInput,
  whitelistInput,
  nameOnlyInput,
  mediaSourceInput,
  captureReportedAppsEnabled,
  historyTitleLimit,
  appHistory,
  playSourceHistory,
  appRawTitleCount,
  panelClass,
  panelHeadClass,
  fieldClass,
  inputClass,
  badgeClass,
  buttonClass,
  dangerButtonClass,
  toggleTileClass,
  minHistoryLimit,
  maxHistoryLimit,
  onAppFilterModeChange,
  onBlacklistInputChange,
  onWhitelistInputChange,
  onNameOnlyInputChange,
  onMediaSourceInputChange,
  onAddBlacklist,
  onAddWhitelist,
  onAddNameOnly,
  onAddMediaSource,
  onRemoveBlacklist,
  onRemoveWhitelist,
  onRemoveNameOnly,
  onRemoveMediaSource,
  onCaptureReportedAppsChange,
  onHistoryTitleLimitChange,
  formatDate,
  onExportHistoryJson,
  onClearHistory,
}: {
  appFilterMode: AppFilterMode;
  appBlacklist: string[];
  appWhitelist: string[];
  appNameOnlyList: string[];
  mediaPlaySourceBlocklist: string[];
  appSuggestions: string[];
  playSourceSuggestions: string[];
  blacklistInput: string;
  whitelistInput: string;
  nameOnlyInput: string;
  mediaSourceInput: string;
  captureReportedAppsEnabled: boolean;
  historyTitleLimit: number;
  appHistory: AppHistoryEntry[];
  playSourceHistory: PlaySourceHistoryEntry[];
  appRawTitleCount: number;
  panelClass: string;
  panelHeadClass: string;
  fieldClass: string;
  inputClass: string;
  badgeClass: string;
  buttonClass: string;
  dangerButtonClass: string;
  toggleTileClass: string;
  minHistoryLimit: number;
  maxHistoryLimit: number;
  onAppFilterModeChange: (mode: AppFilterMode) => void;
  onBlacklistInputChange: (value: string) => void;
  onWhitelistInputChange: (value: string) => void;
  onNameOnlyInputChange: (value: string) => void;
  onMediaSourceInputChange: (value: string) => void;
  onAddBlacklist: () => void;
  onAddWhitelist: () => void;
  onAddNameOnly: () => void;
  onAddMediaSource: () => void;
  onRemoveBlacklist: (index: number) => void;
  onRemoveWhitelist: (index: number) => void;
  onRemoveNameOnly: (index: number) => void;
  onRemoveMediaSource: (index: number) => void;
  onCaptureReportedAppsChange: (value: boolean) => void;
  onHistoryTitleLimitChange: (value: string) => void;
  formatDate: (value?: string | null) => string;
  onExportHistoryJson: () => void;
  onClearHistory: () => void;
}) {
  return (
    <>
      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">Filter</p>
            <h3>App filter mode</h3>
          </div>
          <span className={badgeClass}>{appFilterMode}</span>
        </div>

        <div className="radio-grid">
          <label className={appFilterMode === "blacklist" ? "flex items-start gap-3 rounded-box border border-primary bg-primary/10 p-4 text-left" : "flex items-start gap-3 rounded-box border border-base-300 bg-base-100 p-4 text-left"}>
            <input className="radio radio-primary mt-1" type="radio" name="appFilterMode" checked={appFilterMode === "blacklist"} onChange={() => onAppFilterModeChange("blacklist")} />
            <div>
              <strong>Blacklist</strong>
              <span>Hide any app whose process name exactly matches the blocked list.</span>
            </div>
          </label>
          <label className={appFilterMode === "whitelist" ? "flex items-start gap-3 rounded-box border border-primary bg-primary/10 p-4 text-left" : "flex items-start gap-3 rounded-box border border-base-300 bg-base-100 p-4 text-left"}>
            <input className="radio radio-primary mt-1" type="radio" name="appFilterMode" checked={appFilterMode === "whitelist"} onChange={() => onAppFilterModeChange("whitelist")} />
            <div>
              <strong>Whitelist</strong>
              <span>Only allow apps that exist in the allowed list.</span>
            </div>
          </label>
        </div>

        {appFilterMode === "blacklist" ? (
          <ListEditor
            title="Blocked apps"
            description="Blocked process names are dropped from local status output."
            placeholder="wechat.exe"
            value={appBlacklist}
            inputValue={blacklistInput}
            onInputValueChange={onBlacklistInputChange}
            onAdd={onAddBlacklist}
            onRemove={onRemoveBlacklist}
            suggestions={appSuggestions}
          />
        ) : (
          <ListEditor
            title="Allowed apps"
            description="Only these process names are allowed to reach the local feed and Discord RPC."
            placeholder="code.exe"
            value={appWhitelist}
            inputValue={whitelistInput}
            onInputValueChange={onWhitelistInputChange}
            onAdd={onAddWhitelist}
            onRemove={onRemoveWhitelist}
            suggestions={appSuggestions}
          />
        )}
      </section>

      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">Clauses</p>
            <h3>Name-only and media-source rules</h3>
          </div>
        </div>
        <div className="rules-utility-grid">
          <ListEditor
            title="App-name-only list"
            description="These apps keep only the app name. Their window title is masked before local display."
            placeholder="chrome.exe"
            value={appNameOnlyList}
            inputValue={nameOnlyInput}
            onInputValueChange={onNameOnlyInputChange}
            onAdd={onAddNameOnly}
            onRemove={onRemoveNameOnly}
            suggestions={appSuggestions}
          />
          <ListEditor
            title="Media-source blocklist"
            description="When a play_source hits this list, the media metadata is hidden locally."
            placeholder="system_media"
            value={mediaPlaySourceBlocklist}
            inputValue={mediaSourceInput}
            onInputValueChange={onMediaSourceInputChange}
            onAdd={onAddMediaSource}
            onRemove={onRemoveMediaSource}
            suggestions={playSourceSuggestions}
            lowercase
          />
        </div>
      </section>

      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">History</p>
            <h3>Saved local records</h3>
          </div>
          <span className={badgeClass}>
            {appHistory.length + playSourceHistory.length} records · {appRawTitleCount} raw titles
          </span>
        </div>

        <div className="toggle-grid compact-toggles rules-toggles">
          <label className={toggleTileClass}>
            <div>
              <strong>Capture reported apps</strong>
              <span>Save app and play-source records for local suggestions and export.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={captureReportedAppsEnabled} onChange={(e) => onCaptureReportedAppsChange(e.currentTarget.checked)} />
          </label>
        </div>
        <div className="field-grid compact-fields">
          <label className={fieldClass}>
            <span>Raw titles per app</span>
            <input
              className={inputClass}
              type="number"
              min={minHistoryLimit}
              max={maxHistoryLimit}
              value={historyTitleLimit}
              onChange={(e) => onHistoryTitleLimitChange(e.currentTarget.value)}
            />
          </label>
        </div>

        <HistoryRecordPanels
          appHistory={appHistory}
          playSourceHistory={playSourceHistory}
          historyTitleLimit={historyTitleLimit}
          formatDate={formatDate}
        />

        <div className="card-actions gap-2">
          <button className={buttonClass} type="button" onClick={onExportHistoryJson}>
            Export History Json
          </button>
          <button className={dangerButtonClass} type="button" onClick={onClearHistory}>
            Clear history
          </button>
        </div>
      </section>
    </>
  );
}
