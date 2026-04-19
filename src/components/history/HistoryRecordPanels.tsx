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
  historyRecordLimit,
  historyTitleLimit,
  formatDate,
}: {
  appHistory: AppHistoryEntry[];
  playSourceHistory: PlaySourceHistoryEntry[];
  historyRecordLimit: number;
  historyTitleLimit: number;
  formatDate: (value?: string | null) => string;
}) {
  return (
    <div className="history-record-grid">
      <div className="history-record-panel">
        <div className="history-record-head">
          <strong>Apps</strong>
          <span>{appHistory.length} / {historyRecordLimit}</span>
        </div>
        {appHistory.length === 0 ? (
          <div className="empty-state compact-empty">No app records yet.</div>
        ) : (
          <div className="history-record-list">
            {appHistory.map((entry) => {
              const rawTitles = normalizeAppHistoryTitles(entry, historyTitleLimit);
              return (
                <article key={`${entry.processName}-${entry.updatedAt ?? ""}`} className="history-record-item">
                  <strong>{entry.processName}</strong>
                  <span>{appHistoryDisplayTitle(entry)}</span>
                  {rawTitles.length > 0 ? (
                    <div className="history-title-list">
                      <small>Raw titles</small>
                      {rawTitles.map((title) => (
                        <code key={title}>{title}</code>
                      ))}
                    </div>
                  ) : null}
                  <small>{formatDate(entry.updatedAt)}</small>
                </article>
              );
            })}
          </div>
        )}
      </div>
      <div className="history-record-panel">
        <div className="history-record-head">
          <strong>Play sources</strong>
          <span>{playSourceHistory.length} / {historyRecordLimit}</span>
        </div>
        {playSourceHistory.length === 0 ? (
          <div className="empty-state compact-empty">No play-source records yet.</div>
        ) : (
          <div className="history-record-list">
            {playSourceHistory.map((entry) => (
              <article key={`${entry.source}-${entry.updatedAt ?? ""}`} className="history-record-item history-source-item">
                <span className="history-source-label">{entry.source}</span>
                <strong className="history-source-title">{playSourceHistoryDisplayTitle(entry)}</strong>
                <span className="history-source-meta">{playSourceHistoryMeta(entry)}</span>
                <small className="history-record-time">{formatDate(entry.updatedAt)}</small>
              </article>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
