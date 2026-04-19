import type { ReactNode } from "react";

export function RulesEditorDialog({
  panelHeadClass,
  buttonClass,
  onClose,
  children,
}: {
  panelHeadClass: string;
  buttonClass: string;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <section className="modal modal-open" onClick={onClose}>
      <div
        className="modal-box w-11/12 max-w-6xl p-0"
        role="dialog"
        aria-modal="true"
        aria-labelledby="rules-dialog-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="card-body">
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">Rules</p>
              <h3 id="rules-dialog-title" className="card-title">Detailed rule editor</h3>
              <p>Edit rule clauses, app filter, name-only and media-source lists here, then save from the bottom-right notice.</p>
            </div>
            <button className={buttonClass} type="button" onClick={onClose}>
              Close
            </button>
          </div>
          <div className="rule-modal-body">{children}</div>
        </div>
      </div>
    </section>
  );
}
