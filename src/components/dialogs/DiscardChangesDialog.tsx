export function DiscardChangesDialog({
  panelHeadClass,
  buttonClass,
  dangerButtonClass,
  onClose,
  onConfirm,
}: {
  panelHeadClass: string;
  buttonClass: string;
  dangerButtonClass: string;
  onClose: () => void;
  onConfirm: () => void;
}) {
  return (
    <section className="modal modal-open" onClick={onClose}>
      <div
        className="modal-box w-11/12 max-w-xl p-0"
        role="dialog"
        aria-modal="true"
        aria-labelledby="discard-dialog-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="card-body">
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">Draft</p>
              <h3 id="discard-dialog-title" className="card-title">Revert unsaved changes?</h3>
              <p>This resets the current form back to the last saved settings.</p>
            </div>
          </div>
          <div className="card-actions justify-end gap-2">
            <button className={buttonClass} type="button" onClick={onClose}>
              Cancel
            </button>
            <button className={dangerButtonClass} type="button" onClick={onConfirm}>
              Revert changes
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
