import { motion } from "motion/react";

import type { DiscordDebugPayload, ReporterLogEntry } from "../../types";

const CARD_MOTION = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
};

const LOG_MOTION = {
  initial: { opacity: 0, x: 10 },
  animate: { opacity: 1, x: 0 },
};

const MOTION_TRANSITION = { duration: 0.18, ease: "easeOut" } as const;

function hasJsonPayload(value: unknown) {
  return typeof value === "object" && value !== null && Object.keys(value).length > 0;
}

export function RuntimePrerequisiteCard({
  panelClass,
  panelHeadClass,
  primaryButtonClass,
  runtimeBlockReason,
  onOpenSettings,
}: {
  panelClass: string;
  panelHeadClass: string;
  primaryButtonClass: string;
  runtimeBlockReason: string | null;
  onOpenSettings: () => void;
}) {
  return (
    <section className={`${panelClass} runtime-prerequisite-card`}>
      <div className={panelHeadClass}>
        <div>
          <p className="eyebrow">Prerequisite</p>
          <h3>RPC setup required</h3>
        </div>
        <span className="badge badge-warning badge-soft">Locked</span>
      </div>
      <div className="empty-state">
        {runtimeBlockReason || "Save the RPC settings first to unlock runtime."}
      </div>
      <div className="card-actions gap-2">
        <button className={primaryButtonClass} type="button" onClick={onOpenSettings}>
          Open settings
        </button>
      </div>
    </section>
  );
}

export function RuntimeMonitorCard({
  panelClass,
  panelHeadClass,
  statCardClass,
  primaryButtonClass,
  buttonClass,
  goodBadgeClass,
  badgeClass,
  runtimeRunning,
  currentActivity,
  attachedMeta,
  captureMode,
  lastHeartbeat,
  startBusy,
  stopBusy,
  refreshBusy,
  restartBusy,
  runtimeReady,
  onStart,
  onStop,
  onRefresh,
}: {
  panelClass: string;
  panelHeadClass: string;
  statCardClass: string;
  primaryButtonClass: string;
  buttonClass: string;
  goodBadgeClass: string;
  badgeClass: string;
  runtimeRunning: boolean;
  currentActivity: string;
  attachedMeta: string;
  captureMode: string;
  lastHeartbeat: string;
  startBusy: boolean;
  stopBusy: boolean;
  refreshBusy: boolean;
  restartBusy: boolean;
  runtimeReady: boolean;
  onStart: () => void;
  onStop: () => void;
  onRefresh: () => void;
}) {
  return (
    <motion.section className={`${panelClass} runtime-card runtime-main-card`} {...CARD_MOTION} transition={MOTION_TRANSITION}>
      <div className={panelHeadClass}>
        <div>
          <p className="eyebrow">Monitor</p>
          <h3>Live runtime controls</h3>
        </div>
        <span className={runtimeRunning ? goodBadgeClass : badgeClass}>
          {runtimeRunning ? "Running" : "Stopped"}
        </span>
      </div>
      <div className="stat-grid">
        <div className={statCardClass}>
          <span>Current activity</span>
          <strong>{currentActivity}</strong>
        </div>
        <div className={statCardClass}>
          <span>Attached meta</span>
          <strong>{attachedMeta}</strong>
        </div>
        <div className={statCardClass}>
          <span>Capture mode</span>
          <strong>{captureMode}</strong>
        </div>
        <div className={statCardClass}>
          <span>Last heartbeat</span>
          <strong>{lastHeartbeat}</strong>
        </div>
      </div>
      <div className="card-actions gap-2">
        <button
          className={primaryButtonClass}
          disabled={startBusy || restartBusy || runtimeRunning || !runtimeReady}
          onClick={onStart}
        >
          {startBusy ? "Starting..." : runtimeReady ? "Start runtime" : "RPC required"}
        </button>
        <button
          className={buttonClass}
          disabled={stopBusy || restartBusy || !runtimeRunning}
          onClick={onStop}
        >
          Stop runtime
        </button>
        <button
          className={buttonClass}
          disabled={refreshBusy || restartBusy || !runtimeReady}
          onClick={onRefresh}
        >
          {refreshBusy ? "Refreshing..." : runtimeReady ? "Refresh" : "RPC required"}
        </button>
      </div>
    </motion.section>
  );
}

export function RuntimeLogCard({
  panelClass,
  panelHeadClass,
  runtimeLogs,
  visibleRuntimeLogs,
  runtimeLogPageStart,
  runtimeLogPageEnd,
  safeRuntimeLogPage,
  runtimeLogPageCount,
  formatDate,
  logEntryClass,
  onOpenLogPayload,
  onRuntimeLogPageChange,
}: {
  panelClass: string;
  panelHeadClass: string;
  runtimeLogs: ReporterLogEntry[];
  visibleRuntimeLogs: ReporterLogEntry[];
  runtimeLogPageStart: number;
  runtimeLogPageEnd: number;
  safeRuntimeLogPage: number;
  runtimeLogPageCount: number;
  formatDate: (value?: string | null) => string;
  logEntryClass: (level: string) => string;
  onOpenLogPayload: (entry: ReporterLogEntry) => void;
  onRuntimeLogPageChange: (page: number) => void;
}) {
  return (
    <motion.section className={`${panelClass} runtime-card runtime-log-card`} {...CARD_MOTION} transition={{ ...MOTION_TRANSITION, delay: 0.03 }}>
      <div className={panelHeadClass}>
        <div>
          <p className="eyebrow">Log</p>
          <h3>Recent runtime events</h3>
        </div>
      </div>
      <div className="log-feed compact-log-feed runtime-log-feed">
        {runtimeLogs.length === 0 ? (
          <div className="empty-state">No runtime entries yet.</div>
        ) : (
          visibleRuntimeLogs.map((entry) => {
            const clickable = hasJsonPayload(entry.payload);
            return (
              <motion.article
                key={entry.id}
                layout
                {...LOG_MOTION}
                transition={MOTION_TRANSITION}
                className={`${logEntryClass(entry.level)} ${clickable ? "log-entry-clickable" : ""}`}
                role={clickable ? "button" : undefined}
                tabIndex={clickable ? 0 : undefined}
                onClick={clickable ? () => onOpenLogPayload(entry) : undefined}
                onKeyDown={
                  clickable
                    ? (event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          event.preventDefault();
                          onOpenLogPayload(entry);
                        }
                      }
                    : undefined
                }
                title={clickable ? "Open reported JSON" : undefined}
              >
                <div className="card-body p-3">
                  <div className="log-entry-head">
                    <strong>{entry.title}</strong>
                    <div className="log-entry-actions">
                      <span>{formatDate(entry.timestamp)}</span>
                    </div>
                  </div>
                  <p>{entry.detail}</p>
                </div>
              </motion.article>
            );
          })
        )}
      </div>
      {runtimeLogs.length > 0 ? (
        <div className="pagination-row runtime-log-pagination">
          <span className="pagination-copy">
            {runtimeLogPageStart}-{runtimeLogPageEnd} of {runtimeLogs.length}
          </span>
          <div className="join">
            <button
              className="btn btn-outline btn-xs join-item"
              type="button"
              disabled={safeRuntimeLogPage <= 0}
              onClick={() => onRuntimeLogPageChange(safeRuntimeLogPage - 1)}
            >
              Prev
            </button>
            <span className="btn btn-ghost btn-xs join-item no-animation">
              Page {safeRuntimeLogPage + 1} / {runtimeLogPageCount}
            </span>
            <button
              className="btn btn-outline btn-xs join-item"
              type="button"
              disabled={safeRuntimeLogPage >= runtimeLogPageCount - 1}
              onClick={() => onRuntimeLogPageChange(safeRuntimeLogPage + 1)}
            >
              Next
            </button>
          </div>
        </div>
      ) : null}
    </motion.section>
  );
}

export function RuntimeDebugCard({
  panelClass,
  panelHeadClass,
  buttonClass,
  discordDebugPayload,
  onOpenDiscordPayload,
}: {
  panelClass: string;
  panelHeadClass: string;
  buttonClass: string;
  discordDebugPayload: DiscordDebugPayload | null;
  onOpenDiscordPayload: () => void;
}) {
  return (
    <motion.section className={`${panelClass} runtime-card runtime-debug-card`} {...CARD_MOTION} transition={{ ...MOTION_TRANSITION, delay: 0.06 }}>
      <div className={panelHeadClass}>
        <div>
          <p className="eyebrow">Debug</p>
          <h3>Discord payload JSON</h3>
        </div>
        <button className={buttonClass} type="button" onClick={onOpenDiscordPayload}>
          Open payload JSON
        </button>
      </div>
      {discordDebugPayload ? (
        <div className="empty-state">
          Open payload JSON to inspect the current data being pushed into Discord.
        </div>
      ) : (
        <div className="empty-state">
          No Discord payload has been published yet. Start runtime and wait for a captured activity to pass the current rules.
        </div>
      )}
    </motion.section>
  );
}
