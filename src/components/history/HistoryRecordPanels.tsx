import { useEffect, useState } from "react";

import { HISTORY_RECORD_PAGE_SIZE } from "../../app/appConstants";
import { clampPage, pageCount } from "../../app/appFormatting";
import type { AppHistoryEntry, PlaySourceHistoryEntry } from "../../types";

function normalizeAppHistoryTitles(entry: AppHistoryEntry, limit: number) {
  const result: string[] = [];
  const seen = new Set<string>();
  for (const value of [...(entry.processTitles ?? []), entry.processTitle ?? ""]) {
    const trimmed = String(value ?? "").trim();
    if (!trimmed || seen.has(trimmed)) {
      continue;
    }
    seen.add(trimmed);
    result.push(trimmed);
    if (result.length >= limit) {
      break;
    }
  }
  return result;
}

function appHistoryDisplayTitle(entry: AppHistoryEntry) {
  return entry.statusText?.trim() || entry.processTitle?.trim() || "No title captured";
}

function playSourceHistoryDisplayTitle(entry: PlaySourceHistoryEntry) {
  return entry.mediaTitle?.trim() || entry.mediaSummary?.trim() || "No media title captured";
}

function playSourceHistoryMeta(entry: PlaySourceHistoryEntry) {
  return [entry.mediaArtist, entry.mediaAlbum].map((item) => item?.trim()).filter(Boolean).join(" · ");
}

export function HistoryRecordPanels({
  appHistory,
  playSourceHistory,
  historyTitleLimit,
  formatDate,
}: {
  appHistory: AppHistoryEntry[];
  playSourceHistory: PlaySourceHistoryEntry[];
  historyTitleLimit: number;
  formatDate: (value?: string | null) => string;
}) {
  const [appHistoryPage, setAppHistoryPage] = useState(0);
  const [playSourceHistoryPage, setPlaySourceHistoryPage] = useState(0);

  useEffect(() => {
    setAppHistoryPage((current) => clampPage(current, appHistory.length, HISTORY_RECORD_PAGE_SIZE));
  }, [appHistory.length]);

  useEffect(() => {
    setPlaySourceHistoryPage((current) => clampPage(current, playSourceHistory.length, HISTORY_RECORD_PAGE_SIZE));
  }, [playSourceHistory.length]);

  const appHistoryTotalPages = pageCount(appHistory.length, HISTORY_RECORD_PAGE_SIZE);
  const safeAppHistoryPage = clampPage(appHistoryPage, appHistory.length, HISTORY_RECORD_PAGE_SIZE);
  const appHistoryPageStart = safeAppHistoryPage * HISTORY_RECORD_PAGE_SIZE;
  const pagedAppHistory = appHistory.slice(appHistoryPageStart, appHistoryPageStart + HISTORY_RECORD_PAGE_SIZE);
  const appHistoryPageEnd = appHistoryPageStart + pagedAppHistory.length;

  const playSourceHistoryTotalPages = pageCount(playSourceHistory.length, HISTORY_RECORD_PAGE_SIZE);
  const safePlaySourceHistoryPage = clampPage(
    playSourceHistoryPage,
    playSourceHistory.length,
    HISTORY_RECORD_PAGE_SIZE,
  );
  const playSourceHistoryPageStart = safePlaySourceHistoryPage * HISTORY_RECORD_PAGE_SIZE;
  const pagedPlaySourceHistory = playSourceHistory.slice(
    playSourceHistoryPageStart,
    playSourceHistoryPageStart + HISTORY_RECORD_PAGE_SIZE,
  );
  const playSourceHistoryPageEnd = playSourceHistoryPageStart + pagedPlaySourceHistory.length;

  return (
    <div className="history-record-grid">
      <div className="history-record-panel">
        <div className="history-record-head">
          <strong>Apps</strong>
          <span>
            {appHistory.length} apps captured · keeps {historyTitleLimit} titles per app
          </span>
        </div>
        {appHistory.length === 0 ? (
          <div className="empty-state compact-empty">No app records yet.</div>
        ) : (
          <>
            <div className="history-record-list">
              {pagedAppHistory.map((entry) => {
                const rawTitles = normalizeAppHistoryTitles(entry, historyTitleLimit);
                return (
                  <details
                    key={`${entry.processName}-${entry.updatedAt ?? ""}`}
                    className="history-record-item history-record-toggle discord-advanced-panel"
                  >
                    <summary className="history-record-summary discord-advanced-summary">
                      <div className="history-record-summary-copy">
                        <strong>{entry.processName}</strong>
                        <span>{appHistoryDisplayTitle(entry)}</span>
                      </div>
                      <div className="history-record-summary-meta">
                        <small>{formatDate(entry.updatedAt)}</small>
                        <span className="discord-advanced-summary-hint" aria-hidden="true">
                          <span className="discord-advanced-summary-hint-closed">Expand</span>
                          <span className="discord-advanced-summary-hint-open">Collapse</span>
                          <span className="discord-advanced-summary-caret">v</span>
                        </span>
                      </div>
                    </summary>
                    <div className="history-record-body">
                      {rawTitles.length > 0 ? (
                        <div className="history-title-list">
                          <small>Raw titles</small>
                          {rawTitles.map((title) => (
                            <code key={title}>{title}</code>
                          ))}
                        </div>
                      ) : (
                        <span className="history-record-empty">No raw titles captured yet.</span>
                      )}
                    </div>
                  </details>
                );
              })}
            </div>
            {appHistoryTotalPages > 1 ? (
              <div className="pagination-row">
                <span className="pagination-copy">
                  {appHistoryPageStart + 1}-{appHistoryPageEnd} of {appHistory.length}
                </span>
                <div className="join">
                  <button
                    className="btn btn-outline btn-xs join-item"
                    type="button"
                    disabled={safeAppHistoryPage <= 0}
                    onClick={() => setAppHistoryPage(safeAppHistoryPage - 1)}
                  >
                    Prev
                  </button>
                  <span className="btn btn-ghost btn-xs join-item no-animation">
                    Page {safeAppHistoryPage + 1} / {appHistoryTotalPages}
                  </span>
                  <button
                    className="btn btn-outline btn-xs join-item"
                    type="button"
                    disabled={safeAppHistoryPage >= appHistoryTotalPages - 1}
                    onClick={() => setAppHistoryPage(safeAppHistoryPage + 1)}
                  >
                    Next
                  </button>
                </div>
              </div>
            ) : null}
          </>
        )}
      </div>
      <div className="history-record-panel">
        <div className="history-record-head">
          <strong>Play sources</strong>
          <span>{playSourceHistory.length} sources captured</span>
        </div>
        {playSourceHistory.length === 0 ? (
          <div className="empty-state compact-empty">No play-source records yet.</div>
        ) : (
          <>
            <div className="history-record-list">
              {pagedPlaySourceHistory.map((entry) => {
                const meta = playSourceHistoryMeta(entry);
                return (
                  <details
                    key={`${entry.source}-${entry.updatedAt ?? ""}`}
                    className="history-record-item history-record-toggle history-source-item discord-advanced-panel"
                  >
                    <summary className="history-record-summary discord-advanced-summary">
                      <div className="history-record-summary-copy">
                        <span className="history-source-label">{entry.source}</span>
                        <strong className="history-source-title">{playSourceHistoryDisplayTitle(entry)}</strong>
                      </div>
                      <div className="history-record-summary-meta">
                        <small className="history-record-time">{formatDate(entry.updatedAt)}</small>
                        <span className="discord-advanced-summary-hint" aria-hidden="true">
                          <span className="discord-advanced-summary-hint-closed">Expand</span>
                          <span className="discord-advanced-summary-hint-open">Collapse</span>
                          <span className="discord-advanced-summary-caret">v</span>
                        </span>
                      </div>
                    </summary>
                    <div className="history-record-body">
                      {meta ? (
                        <span className="history-source-meta">{meta}</span>
                      ) : (
                        <span className="history-record-empty">No artist or album captured.</span>
                      )}
                    </div>
                  </details>
                );
              })}
            </div>
            {playSourceHistoryTotalPages > 1 ? (
              <div className="pagination-row">
                <span className="pagination-copy">
                  {playSourceHistoryPageStart + 1}-{playSourceHistoryPageEnd} of {playSourceHistory.length}
                </span>
                <div className="join">
                  <button
                    className="btn btn-outline btn-xs join-item"
                    type="button"
                    disabled={safePlaySourceHistoryPage <= 0}
                    onClick={() => setPlaySourceHistoryPage(safePlaySourceHistoryPage - 1)}
                  >
                    Prev
                  </button>
                  <span className="btn btn-ghost btn-xs join-item no-animation">
                    Page {safePlaySourceHistoryPage + 1} / {playSourceHistoryTotalPages}
                  </span>
                  <button
                    className="btn btn-outline btn-xs join-item"
                    type="button"
                    disabled={safePlaySourceHistoryPage >= playSourceHistoryTotalPages - 1}
                    onClick={() => setPlaySourceHistoryPage(safePlaySourceHistoryPage + 1)}
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
  );
}
