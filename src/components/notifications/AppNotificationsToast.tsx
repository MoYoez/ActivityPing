type ToastNotice = {
  id: number;
  tone: "info" | "success" | "warn" | "error";
  title: string;
  detail: string;
};

export function AppNotificationsToast({
  runtimeNeedsRestart,
  dirty,
  notices,
  primaryButtonClass,
  buttonClass,
  restartRuntimeBusy,
  startRuntimeBusy,
  stopRuntimeBusy,
  saveDraftBusy,
  alertClass,
  onRestartRuntime,
  onOpenDiscardDialog,
  onSaveDraft,
}: {
  runtimeNeedsRestart: boolean;
  dirty: boolean;
  notices: ToastNotice[];
  primaryButtonClass: string;
  buttonClass: string;
  restartRuntimeBusy: boolean;
  startRuntimeBusy: boolean;
  stopRuntimeBusy: boolean;
  saveDraftBusy: boolean;
  alertClass: (tone: ToastNotice["tone"]) => string;
  onRestartRuntime: () => void;
  onOpenDiscardDialog: () => void;
  onSaveDraft: () => void;
}) {
  return (
    <section className="toast toast-end toast-bottom">
      {runtimeNeedsRestart ? (
        <article className="save-reminder card card-compact border border-warning bg-base-100 shadow-lg">
          <div className="card-body">
            <div className="save-reminder-copy">
              <span className="badge badge-warning badge-soft">Restart</span>
              <div className="notice-copy">
                <strong>Runtime restart required</strong>
                <span>Restart runtime to apply the saved configuration changes.</span>
              </div>
            </div>
            <div className="save-reminder-actions">
              <button
                className={primaryButtonClass}
                type="button"
                disabled={restartRuntimeBusy || startRuntimeBusy || stopRuntimeBusy}
                onClick={onRestartRuntime}
              >
                {restartRuntimeBusy ? "Restarting..." : "Restart now"}
              </button>
            </div>
          </div>
        </article>
      ) : null}
      {dirty ? (
        <article className="save-reminder card card-compact border border-base-300 bg-base-100 shadow-lg">
          <div className="card-body">
            <div className="save-reminder-copy">
              <span className="badge badge-warning badge-soft">Draft</span>
              <div className="notice-copy">
                <strong>Unsaved draft</strong>
                <span>Changes in the current form are not saved yet.</span>
              </div>
            </div>
            <div className="save-reminder-actions">
              <button
                className={buttonClass}
                type="button"
                disabled={saveDraftBusy}
                onClick={onOpenDiscardDialog}
              >
                Revert changes
              </button>
              <button
                className={primaryButtonClass}
                type="button"
                disabled={saveDraftBusy}
                onClick={onSaveDraft}
              >
                {saveDraftBusy ? "Saving..." : "Save changes"}
              </button>
            </div>
          </div>
        </article>
      ) : null}
      {notices.map((item) => (
        <article key={item.id} className={alertClass(item.tone)}>
          <div className="notice-copy">
            <strong>{item.title}</strong>
            <span>{item.detail}</span>
          </div>
        </article>
      ))}
    </section>
  );
}
