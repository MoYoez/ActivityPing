import { SuggestionInput } from "../ListEditor";
import { summarizeRuleGroup } from "../../lib/rules";
import type { AppMessageRuleGroup, AppMessageTitleRule } from "../../types";
import { RuleDiscordAddonsEditor } from "./RuleDiscordAddonsEditor";

type TitleRuleMode = AppMessageTitleRule["mode"];

export function RuleGroupsEditorSection({
  rulesCount,
  showProcessName,
  customOverrideEnabled,
  rulesImportOpen,
  rulesImportValue,
  activeRule,
  activeRuleIndex,
  pagedRuleGroups,
  ruleGroupPageStart,
  safeRuleGroupPage,
  ruleGroupTotalPages,
  pagedTitleRules,
  titleRulePageStart,
  safeTitleRulePage,
  titleRuleTotalPages,
  activeTitleRuleCount,
  appSuggestions,
  activeRuleAdvancedAddonsConfigured,
  customAddonsConfigured,
  panelClass,
  cardClass,
  panelHeadClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  textareaClass,
  selectClass,
  buttonClass,
  primaryButtonClass,
  dangerButtonClass,
  badgeClass,
  subruleCardClass,
  toggleTileClass,
  onShowProcessNameChange,
  onCustomOverrideChange,
  onAddRuleGroup,
  onCopyRulesJson,
  onToggleImport,
  onRulesImportValueChange,
  onApplyImportedRules,
  onSelectRule,
  onRuleGroupPageChange,
  onMoveActiveRuleUp,
  onMoveActiveRuleDown,
  onDeleteActiveRule,
  onActiveRuleProcessMatchChange,
  onActiveRuleDefaultTextChange,
  onAddTitleRule,
  onTitleRulePageChange,
  onMoveTitleRuleUp,
  onMoveTitleRuleDown,
  onRemoveTitleRule,
  onTitleRuleModeChange,
  onTitleRulePatternChange,
  onTitleRuleTextChange,
  patchRuleAt,
  patchRuleDiscordButtonAt,
  normalizePositiveNumberInput,
}: {
  rulesCount: number;
  showProcessName: boolean;
  customOverrideEnabled: boolean;
  rulesImportOpen: boolean;
  rulesImportValue: string;
  activeRule: AppMessageRuleGroup | null;
  activeRuleIndex: number;
  pagedRuleGroups: AppMessageRuleGroup[];
  ruleGroupPageStart: number;
  safeRuleGroupPage: number;
  ruleGroupTotalPages: number;
  pagedTitleRules: AppMessageTitleRule[];
  titleRulePageStart: number;
  safeTitleRulePage: number;
  titleRuleTotalPages: number;
  activeTitleRuleCount: number;
  appSuggestions: string[];
  activeRuleAdvancedAddonsConfigured: boolean;
  customAddonsConfigured: boolean;
  panelClass: string;
  cardClass: string;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  textareaClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  badgeClass: string;
  subruleCardClass: string;
  toggleTileClass: string;
  onShowProcessNameChange: (value: boolean) => void;
  onCustomOverrideChange: (value: boolean) => void;
  onAddRuleGroup: () => void;
  onCopyRulesJson: () => void;
  onToggleImport: () => void;
  onRulesImportValueChange: (value: string) => void;
  onApplyImportedRules: () => void;
  onSelectRule: (index: number) => void;
  onRuleGroupPageChange: (page: number) => void;
  onMoveActiveRuleUp: () => void;
  onMoveActiveRuleDown: () => void;
  onDeleteActiveRule: () => void;
  onActiveRuleProcessMatchChange: (value: string) => void;
  onActiveRuleDefaultTextChange: (value: string) => void;
  onAddTitleRule: () => void;
  onTitleRulePageChange: (page: number) => void;
  onMoveTitleRuleUp: (index: number) => void;
  onMoveTitleRuleDown: (index: number) => void;
  onRemoveTitleRule: (index: number) => void;
  onTitleRuleModeChange: (index: number, mode: TitleRuleMode) => void;
  onTitleRulePatternChange: (index: number, value: string) => void;
  onTitleRuleTextChange: (index: number, value: string) => void;
  patchRuleAt: (index: number, updater: (rule: AppMessageRuleGroup) => AppMessageRuleGroup) => void;
  patchRuleDiscordButtonAt: (
    ruleIndex: number,
    buttonIndex: number,
    updater: (button: AppMessageRuleGroup["buttons"][number]) => AppMessageRuleGroup["buttons"][number],
  ) => void;
  normalizePositiveNumberInput: (value: string) => number | null;
}) {
  return (
    <section className={panelClass}>
      <div className={panelHeadClass}>
        <div>
          <p className="eyebrow">App rules</p>
          <h3>Message rule groups</h3>
        </div>
        <span className={badgeClass}>{rulesCount} groups</span>
      </div>

      <div className="toggle-grid compact-toggles rules-toggles">
        <label className={toggleTileClass}>
          <div>
            <strong>Show process name on rule hit</strong>
            <span>Append the executable name after a matching rule text. In Smart mode without media, the process name moves to the last line.</span>
          </div>
          <input
            className="toggle toggle-primary"
            type="checkbox"
            checked={showProcessName}
            onChange={(e) => onShowProcessNameChange(e.currentTarget.checked)}
          />
        </label>
        <label className={toggleTileClass}>
          <div>
            <strong>Force Custom add-ons override rule add-ons</strong>
            <span>When Custom add-ons are configured, reuse them instead of the matched rule group's buttons or social metadata.</span>
          </div>
          <input
            className="toggle toggle-primary"
            type="checkbox"
            checked={customOverrideEnabled}
            onChange={(e) => onCustomOverrideChange(e.currentTarget.checked)}
          />
        </label>
      </div>

      <div className="card-actions gap-2">
        <button className={primaryButtonClass} type="button" onClick={onAddRuleGroup}>
          Add rule group
        </button>
        <button className={buttonClass} type="button" onClick={onCopyRulesJson}>
          Copy rules JSON
        </button>
        <button className={buttonClass} type="button" onClick={onToggleImport}>
          {rulesImportOpen ? "Hide import" : "Import rules JSON"}
        </button>
      </div>

      {rulesImportOpen ? (
        <div className="import-panel">
          <label className={fieldClass}>
            <span>Rules JSON</span>
            <textarea
              className={textareaClass}
              value={rulesImportValue}
              onChange={(e) => onRulesImportValueChange(e.currentTarget.value)}
              placeholder='{"version":2,"rules":{"appMessageRules":[]}}'
            />
          </label>
          <div className="card-actions gap-2">
            <button className={primaryButtonClass} type="button" onClick={onApplyImportedRules}>
              Apply imported rules
            </button>
          </div>
        </div>
      ) : null}

      <div className="rules-shell">
        <div className="rule-list-panel rounded-box border border-base-300 bg-base-200/45 p-4">
          <div className="list-editor-summary">
            <div className="list-editor-copy">
              <strong className="block font-semibold">Process groups</strong>
              <p>Rules run from top to bottom.</p>
            </div>
          </div>
          {rulesCount === 0 ? (
            <div className="empty-state compact-empty">No app rule groups yet.</div>
          ) : (
            <>
              <div className="grid gap-2">
                {pagedRuleGroups.map((rule, offset) => {
                  const index = ruleGroupPageStart + offset;
                  return (
                    <button
                      key={`${rule.processMatch || "rule"}-${index}`}
                      className={`btn btn-ghost h-auto min-h-16 w-full flex-col items-start justify-start gap-1 text-left normal-case ${activeRuleIndex === index ? "btn-active" : ""}`}
                      type="button"
                      onClick={() => onSelectRule(index)}
                    >
                      <strong className="block break-words">{rule.processMatch || `Rule group ${index + 1}`}</strong>
                      <span className="mt-1 block text-sm text-base-content/70">{summarizeRuleGroup(rule)}</span>
                    </button>
                  );
                })}
              </div>
              {ruleGroupTotalPages > 1 ? (
                <div className="pagination-row">
                  <span className="pagination-copy">
                    {ruleGroupPageStart + 1}-{Math.min(ruleGroupPageStart + pagedRuleGroups.length, rulesCount)} of {rulesCount}
                  </span>
                  <div className="join">
                    <button
                      className="btn btn-outline btn-xs join-item"
                      type="button"
                      disabled={safeRuleGroupPage <= 0}
                      onClick={() => onRuleGroupPageChange(safeRuleGroupPage - 1)}
                    >
                      Prev
                    </button>
                    <span className="btn btn-ghost btn-xs join-item no-animation">
                      Page {safeRuleGroupPage + 1} / {ruleGroupTotalPages}
                    </span>
                    <button
                      className="btn btn-outline btn-xs join-item"
                      type="button"
                      disabled={safeRuleGroupPage >= ruleGroupTotalPages - 1}
                      onClick={() => onRuleGroupPageChange(safeRuleGroupPage + 1)}
                    >
                      Next
                    </button>
                  </div>
                </div>
              ) : null}
            </>
          )}
        </div>

        <div className={`${cardClass} space-y-4 p-4 rules-editor`}>
          {activeRule ? (
            <>
              <div className={panelHeadClass}>
                <div>
                  <strong className="block font-semibold">Group editor</strong>
                  <p className="mt-1 text-sm text-base-content/70">
                    Match the process first, then the title subrules. Use {"{process}"} and {"{title}"} in text.
                  </p>
                </div>
                <div className="card-actions gap-2">
                  <button className={buttonClass} type="button" disabled={activeRuleIndex <= 0} onClick={onMoveActiveRuleUp}>
                    Move up
                  </button>
                  <button
                    className={buttonClass}
                    type="button"
                    disabled={activeRuleIndex >= rulesCount - 1}
                    onClick={onMoveActiveRuleDown}
                  >
                    Move down
                  </button>
                  <button className={dangerButtonClass} type="button" onClick={onDeleteActiveRule}>
                    Delete group
                  </button>
                </div>
              </div>

              <div className="field-grid">
                <label className={fieldClass}>
                  <span>Process match</span>
                  <SuggestionInput
                    value={activeRule.processMatch}
                    onChange={onActiveRuleProcessMatchChange}
                    suggestions={appSuggestions}
                    placeholder="code.exe"
                  />
                </label>
                <label className={fieldSpanClass}>
                  <span>Default text</span>
                  <textarea
                    className={textareaClass}
                    value={activeRule.defaultText}
                    onChange={(e) => onActiveRuleDefaultTextChange(e.currentTarget.value)}
                    placeholder="Coding"
                  />
                </label>
              </div>

              <div className="rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
                <div className={panelHeadClass}>
                  <div>
                    <strong className="block font-semibold">Title subrules</strong>
                    <p className="mt-1 text-sm text-base-content/70">Choose plain contains or regex matching for the window title.</p>
                  </div>
                  <button className={buttonClass} type="button" onClick={onAddTitleRule}>
                    Add title rule
                  </button>
                </div>

                {activeRule.titleRules.length === 0 ? (
                  <div className="empty-state compact-empty">No title subrules yet.</div>
                ) : (
                  <>
                    <div className="grid gap-2">
                      {pagedTitleRules.map((titleRule, offset) => {
                        const titleRuleIndex = titleRulePageStart + offset;
                        return (
                          <article key={`${activeRuleIndex}-${titleRuleIndex}`} className={`${subruleCardClass} p-4`}>
                            <div className={panelHeadClass}>
                              <strong className="block font-semibold">
                                Title rule {titleRuleIndex + 1} / {activeRule.titleRules.length}
                              </strong>
                              <div className="card-actions gap-2">
                                <button
                                  className={buttonClass}
                                  type="button"
                                  disabled={titleRuleIndex <= 0}
                                  onClick={() => onMoveTitleRuleUp(titleRuleIndex)}
                                >
                                  Up
                                </button>
                                <button
                                  className={buttonClass}
                                  type="button"
                                  disabled={titleRuleIndex >= activeRule.titleRules.length - 1}
                                  onClick={() => onMoveTitleRuleDown(titleRuleIndex)}
                                >
                                  Down
                                </button>
                                <button
                                  className={dangerButtonClass}
                                  type="button"
                                  onClick={() => onRemoveTitleRule(titleRuleIndex)}
                                >
                                  Remove
                                </button>
                              </div>
                            </div>

                            <div className="field-grid compact-fields">
                              <label className={fieldClass}>
                                <span>Mode</span>
                                <select
                                  className={selectClass}
                                  value={titleRule.mode}
                                  onChange={(e) =>
                                    onTitleRuleModeChange(
                                      titleRuleIndex,
                                      e.currentTarget.value === "regex" ? "regex" : "plain",
                                    )
                                  }
                                >
                                  <option value="plain">Plain contains</option>
                                  <option value="regex">Regex</option>
                                </select>
                              </label>
                              <label className={fieldSpanClass}>
                                <span>Pattern</span>
                                <textarea
                                  className={textareaClass}
                                  value={titleRule.pattern}
                                  onChange={(e) => onTitleRulePatternChange(titleRuleIndex, e.currentTarget.value)}
                                  placeholder={titleRule.mode === "regex" ? "\\.tsx$" : "Visual Studio Code"}
                                />
                              </label>
                              <label className={fieldSpanClass}>
                                <span>Text</span>
                                <textarea
                                  className={textareaClass}
                                  value={titleRule.text}
                                  onChange={(e) => onTitleRuleTextChange(titleRuleIndex, e.currentTarget.value)}
                                  placeholder="Writing frontend: {title}"
                                />
                              </label>
                            </div>
                          </article>
                        );
                      })}
                    </div>
                    {titleRuleTotalPages > 1 ? (
                      <div className="pagination-row">
                        <span className="pagination-copy">
                          {titleRulePageStart + 1}-{Math.min(titleRulePageStart + pagedTitleRules.length, activeTitleRuleCount)} of {activeTitleRuleCount}
                        </span>
                        <div className="join">
                          <button
                            className="btn btn-outline btn-xs join-item"
                            type="button"
                            disabled={safeTitleRulePage <= 0}
                            onClick={() => onTitleRulePageChange(safeTitleRulePage - 1)}
                          >
                            Prev
                          </button>
                          <span className="btn btn-ghost btn-xs join-item no-animation">
                            Page {safeTitleRulePage + 1} / {titleRuleTotalPages}
                          </span>
                          <button
                            className="btn btn-outline btn-xs join-item"
                            type="button"
                            disabled={safeTitleRulePage >= titleRuleTotalPages - 1}
                            onClick={() => onTitleRulePageChange(safeTitleRulePage + 1)}
                          >
                            Next
                          </button>
                        </div>
                      </div>
                    ) : null}
                  </>
                )}
              </div>

              <RuleDiscordAddonsEditor
                activeRule={activeRule}
                activeRuleIndex={activeRuleIndex}
                activeRuleAdvancedAddonsConfigured={activeRuleAdvancedAddonsConfigured}
                customAddonsConfigured={customAddonsConfigured}
                customOverrideEnabled={customOverrideEnabled}
                buttonClass={buttonClass}
                dangerButtonClass={dangerButtonClass}
                fieldClass={fieldClass}
                fieldSpanClass={fieldSpanClass}
                inputClass={inputClass}
                patchRuleAt={patchRuleAt}
                patchRuleDiscordButtonAt={patchRuleDiscordButtonAt}
                normalizePositiveNumberInput={normalizePositiveNumberInput}
              />
            </>
          ) : (
            <div className="empty-state rules-empty">Create a process group to start editing local app message rules.</div>
          )}
        </div>
      </div>
    </section>
  );
}
