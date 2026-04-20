export function JsonViewerDialog({
  panelHeadClass,
  buttonClass,
  eyebrow,
  title,
  description,
  emptyText,
  hasValue,
  valueJson,
  onClose,
}: {
  panelHeadClass: string;
  buttonClass: string;
  eyebrow: string;
  title: string;
  description: string;
  emptyText: string;
  hasValue: boolean;
  valueJson: string;
  onClose: () => void;
}) {
  return (
    <section className="modal modal-open">
      <div
        className="modal-box w-11/12 max-w-4xl p-0"
        role="dialog"
        aria-modal="true"
        aria-labelledby="json-viewer-dialog-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="card-body">
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">{eyebrow}</p>
              <h3 id="json-viewer-dialog-title" className="card-title">{title}</h3>
              <p>{description}</p>
            </div>
            <button className={buttonClass} type="button" onClick={onClose}>
              Close
            </button>
          </div>
          {hasValue ? <pre className="debug-json">{valueJson}</pre> : <div className="empty-state">{emptyText}</div>}
        </div>
      </div>
    </section>
  );
}
