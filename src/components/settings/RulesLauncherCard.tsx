import type { AppFilterMode } from "../../types";

export function RulesLauncherCard({
  panelClass,
  panelHeadClass,
  badgeClass,
  primaryButtonClass,
  ruleGroupCount,
  appFilterMode,
  nameOnlyCount,
  mediaBlockCount,
  onOpenRules,
}: {
  panelClass: string;
  panelHeadClass: string;
  badgeClass: string;
  primaryButtonClass: string;
  ruleGroupCount: number;
  appFilterMode: AppFilterMode;
  nameOnlyCount: number;
  mediaBlockCount: number;
  onOpenRules: () => void;
}) {
  return (
    <section className={panelClass}>
      <div className={panelHeadClass}>
        <div>
          <p className="eyebrow">Rule center</p>
          <h3>Open rule dialog</h3>
        </div>
        <span className={badgeClass}>
          {ruleGroupCount + nameOnlyCount + mediaBlockCount} items
        </span>
      </div>
      <div className="rule-entry-grid">
        <div className="rule-entry-copy">
          <strong>Rule clauses, app filter, name-only and media-source lists</strong>
          <p>Detailed rule editing now lives in a secondary dialog so the main settings page stays compact.</p>
          <div className="rule-entry-meta">
            <span className="badge badge-soft">{ruleGroupCount} rule groups</span>
            <span className="badge badge-soft">{appFilterMode} filter</span>
            <span className="badge badge-soft">{nameOnlyCount} name-only apps</span>
            <span className="badge badge-soft">{mediaBlockCount} media blocks</span>
          </div>
        </div>
        <div className="card-actions gap-2">
          <button className={primaryButtonClass} type="button" onClick={onOpenRules}>
            Open rules
          </button>
        </div>
      </div>
    </section>
  );
}
