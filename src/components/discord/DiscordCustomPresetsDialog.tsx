import type { DiscordCustomPreset } from "../../types";

function shortenPresetName(value: string, maxLength = 18) {
  return value.length > maxLength ? `${value.slice(0, maxLength - 3)}...` : value;
}

export function DiscordCustomPresetsDialog({
  presets,
  activePresetIndex,
  appliedPresetIndex,
  appliedPresetName,
  pagedPresets,
  presetPageStart,
  presetPageSize,
  safePresetPage,
  presetTotalPages,
  panelHeadClass,
  subruleCardClass,
  buttonClass,
  primaryButtonClass,
  dangerButtonClass,
  summarizePreset,
  onClose,
  onSaveCurrentAsPreset,
  onAddPreset,
  onOpenPreset,
  onMovePresetUp,
  onMovePresetDown,
  onRemovePreset,
  onPresetPageChange,
}: {
  presets: DiscordCustomPreset[];
  activePresetIndex: number | null;
  appliedPresetIndex: number | null;
  appliedPresetName: string | null;
  pagedPresets: DiscordCustomPreset[];
  presetPageStart: number;
  presetPageSize: number;
  safePresetPage: number;
  presetTotalPages: number;
  panelHeadClass: string;
  subruleCardClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  summarizePreset: (preset: DiscordCustomPreset) => string;
  onClose: () => void;
  onSaveCurrentAsPreset: () => void;
  onAddPreset: () => void;
  onOpenPreset: (index: number) => void;
  onMovePresetUp: (index: number) => void;
  onMovePresetDown: (index: number) => void;
  onRemovePreset: (index: number) => void;
  onPresetPageChange: (page: number) => void;
}) {
  const savePresetButtonLabel =
    appliedPresetName === null
      ? "Save current as preset"
      : `Update "${shortenPresetName(appliedPresetName)}"`;

  return (
    <section className="modal modal-open">
      <div
        className="modal-box w-11/12 max-w-5xl p-0"
        role="dialog"
        aria-modal="true"
        aria-labelledby="custom-presets-dialog-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="card-body">
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">Custom</p>
              <h3 id="custom-presets-dialog-title" className="card-title">Custom presets</h3>
              <p>
                {appliedPresetName
                  ? `Currently applied: ${appliedPresetName}. Browse saved Custom profiles page by page.`
                  : "Browse saved Custom profiles page by page. Click one to open its editor."}
              </p>
            </div>
            <div className="card-actions gap-2">
              <button
                className={primaryButtonClass}
                type="button"
                title={appliedPresetName ? `Update "${appliedPresetName}"` : undefined}
                onClick={onSaveCurrentAsPreset}
              >
                {savePresetButtonLabel}
              </button>
              <button className={buttonClass} type="button" onClick={onAddPreset}>
                Add preset
              </button>
              <button className={buttonClass} type="button" onClick={onClose}>
                Close
              </button>
            </div>
          </div>
          <div className="rule-modal-body">
            {presets.length === 0 ? (
              <div className="empty-state">No Custom presets yet.</div>
            ) : (
              <>
                <div className="grid gap-2">
                  {pagedPresets.map((preset, offset) => {
                    const index = presetPageStart + offset;
                    return (
                      <article
                        key={`custom-preset-${index}`}
                        className={`${subruleCardClass} log-entry-clickable p-4`}
                        role="button"
                        tabIndex={0}
                        onClick={() => onOpenPreset(index)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            event.preventDefault();
                            onOpenPreset(index);
                          }
                        }}
                      >
                        <div className={panelHeadClass}>
                          <div>
                            <strong className="block font-semibold">
                              {preset.name.trim() || `Custom preset ${index + 1}`}
                            </strong>
                            <p className="mt-1 text-sm text-base-content/70">{summarizePreset(preset)}</p>
                          </div>
                          <div className="card-actions gap-2">
                            {appliedPresetIndex === index ? <span className="badge badge-soft">Applied</span> : null}
                            {activePresetIndex === index ? (
                              <span className="badge badge-soft">Editing</span>
                            ) : (
                              <span className="badge badge-soft">Open</span>
                            )}
                            <button
                              className={buttonClass}
                              type="button"
                              disabled={index <= 0}
                              onClick={(event) => {
                                event.stopPropagation();
                                onMovePresetUp(index);
                              }}
                            >
                              Up
                            </button>
                            <button
                              className={buttonClass}
                              type="button"
                              disabled={index >= presets.length - 1}
                              onClick={(event) => {
                                event.stopPropagation();
                                onMovePresetDown(index);
                              }}
                            >
                              Down
                            </button>
                            <button
                              className={dangerButtonClass}
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                onRemovePreset(index);
                              }}
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      </article>
                    );
                  })}
                </div>
                {presetTotalPages > 1 ? (
                  <div className="pagination-row">
                    <span className="pagination-copy">
                      {presetPageStart + 1}-{Math.min(presetPageStart + presetPageSize, presets.length)} of {presets.length}
                    </span>
                    <div className="join">
                      <button
                        className="btn btn-outline btn-xs join-item"
                        type="button"
                        disabled={safePresetPage <= 0}
                        onClick={() => onPresetPageChange(safePresetPage - 1)}
                      >
                        Prev
                      </button>
                      <span className="btn btn-ghost btn-xs join-item no-animation">
                        Page {safePresetPage + 1} / {presetTotalPages}
                      </span>
                      <button
                        className="btn btn-outline btn-xs join-item"
                        type="button"
                        disabled={safePresetPage >= presetTotalPages - 1}
                        onClick={() => onPresetPageChange(safePresetPage + 1)}
                      >
                        Next
                      </button>
                    </div>
                  </div>
                ) : null}
              </>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
