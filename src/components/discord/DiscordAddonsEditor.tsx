import type { DiscordRichPresenceButtonConfig } from "../../types";

export function DiscordAddonsEditor({
  title,
  description,
  buttons,
  advancedConfigured,
  buttonClass,
  dangerButtonClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  panelHeadClass,
  onAddButton,
  onPatchButtonAt,
  onRemoveButtonAt,
  partyId,
  partySizeCurrent,
  partySizeMax,
  joinSecret,
  spectateSecret,
  matchSecret,
  onPartyIdChange,
  onPartySizeCurrentChange,
  onPartySizeMaxChange,
  onJoinSecretChange,
  onSpectateSecretChange,
  onMatchSecretChange,
}: {
  title: string;
  description: string;
  buttons: DiscordRichPresenceButtonConfig[];
  advancedConfigured: boolean;
  buttonClass: string;
  dangerButtonClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  panelHeadClass: string;
  onAddButton: () => void;
  onPatchButtonAt: (
    index: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) => void;
  onRemoveButtonAt: (index: number) => void;
  partyId: string;
  partySizeCurrent: number | null;
  partySizeMax: number | null;
  joinSecret: string;
  spectateSecret: string;
  matchSecret: string;
  onPartyIdChange: (value: string) => void;
  onPartySizeCurrentChange: (value: string) => void;
  onPartySizeMaxChange: (value: string) => void;
  onJoinSecretChange: (value: string) => void;
  onSpectateSecretChange: (value: string) => void;
  onMatchSecretChange: (value: string) => void;
}) {
  return (
    <div className="rounded-box border border-base-300 bg-base-100 p-4 space-y-4">
      <div className={panelHeadClass}>
        <div>
          <strong className="block font-semibold">{title}</strong>
          <p className="mt-1 text-sm text-base-content/70">{description}</p>
        </div>
        <span className="badge badge-soft">{buttons.length} / 2 buttons</span>
      </div>
      <div className="space-y-3">
        {buttons.map((button, index) => (
          <div key={`${title}-button-${index}`} className="rounded-box border border-base-300 bg-base-200/50 p-3 space-y-3">
            <div className="field-grid">
              <label className={fieldClass}>
                <span>Button label</span>
                <input
                  className={inputClass}
                  value={button.label}
                  onChange={(e) => {
                    const value = e.currentTarget.value;
                    onPatchButtonAt(index, (current) => ({ ...current, label: value }));
                  }}
                  placeholder="Open website"
                />
              </label>
              <label className={fieldClass}>
                <span>Button URL</span>
                <input
                  className={inputClass}
                  value={button.url}
                  onChange={(e) => {
                    const value = e.currentTarget.value;
                    onPatchButtonAt(index, (current) => ({ ...current, url: value }));
                  }}
                  placeholder="https://example.com or myapp://open"
                />
              </label>
            </div>
            <div className="card-actions justify-end">
              <button className={dangerButtonClass} type="button" onClick={() => onRemoveButtonAt(index)}>
                Remove button
              </button>
            </div>
          </div>
        ))}
        {buttons.length < 2 ? (
          <button className={buttonClass} type="button" onClick={onAddButton}>
            Add button
          </button>
        ) : null}
      </div>

      <details className="discord-advanced-panel rounded-box border border-base-300 bg-base-200/45 p-4">
        <summary className="discord-advanced-summary flex cursor-pointer list-none items-center justify-between gap-3">
          <div>
            <strong className="block font-semibold">Advanced</strong>
            <p className="mt-1 text-sm text-base-content/70">Party metadata and Discord social secrets.</p>
          </div>
          <div className="discord-advanced-summary-meta">
            {advancedConfigured ? <span className="badge badge-soft">Configured</span> : null}
            <span className="discord-advanced-summary-hint" aria-hidden="true">
              <span className="discord-advanced-summary-hint-closed">Expand</span>
              <span className="discord-advanced-summary-hint-open">Collapse</span>
              <span className="discord-advanced-summary-caret">v</span>
            </span>
          </div>
        </summary>
        <div className="field-grid mt-4">
          <label className={fieldClass}>
            <span>Party ID</span>
            <input
              className={inputClass}
              value={partyId}
              onChange={(e) => onPartyIdChange(e.currentTarget.value)}
              placeholder="party-123"
            />
          </label>
          <label className={fieldClass}>
            <span>Party current size</span>
            <input
              className={inputClass}
              type="number"
              min={1}
              value={partySizeCurrent ?? ""}
              onChange={(e) => onPartySizeCurrentChange(e.currentTarget.value)}
              placeholder="2"
            />
          </label>
          <label className={fieldClass}>
            <span>Party max size</span>
            <input
              className={inputClass}
              type="number"
              min={1}
              value={partySizeMax ?? ""}
              onChange={(e) => onPartySizeMaxChange(e.currentTarget.value)}
              placeholder="3"
            />
          </label>
          <label className={fieldSpanClass}>
            <span>Join secret</span>
            <input
              className={inputClass}
              value={joinSecret}
              onChange={(e) => onJoinSecretChange(e.currentTarget.value)}
              placeholder="join-secret"
            />
          </label>
          <label className={fieldSpanClass}>
            <span>Spectate secret</span>
            <input
              className={inputClass}
              value={spectateSecret}
              onChange={(e) => onSpectateSecretChange(e.currentTarget.value)}
              placeholder="spectate-secret"
            />
          </label>
          <label className={fieldSpanClass}>
            <span>Match secret</span>
            <input
              className={inputClass}
              value={matchSecret}
              onChange={(e) => onMatchSecretChange(e.currentTarget.value)}
              placeholder="match-secret"
            />
          </label>
        </div>
      </details>
    </div>
  );
}
