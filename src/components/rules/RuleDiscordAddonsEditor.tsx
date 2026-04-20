import type { AppMessageRuleGroup, DiscordRichPresenceButtonConfig } from "../../types";

function createDiscordButton(): DiscordRichPresenceButtonConfig {
  return { label: "", url: "" };
}

export function RuleDiscordAddonsEditor({
  activeRule,
  activeRuleIndex,
  activeRuleAdvancedAddonsConfigured,
  customAddonsConfigured,
  customOverrideEnabled,
  buttonClass,
  dangerButtonClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  patchRuleAt,
  patchRuleDiscordButtonAt,
  normalizePositiveNumberInput,
}: {
  activeRule: AppMessageRuleGroup;
  activeRuleIndex: number;
  activeRuleAdvancedAddonsConfigured: boolean;
  customAddonsConfigured: boolean;
  customOverrideEnabled: boolean;
  buttonClass: string;
  dangerButtonClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  patchRuleAt: (index: number, updater: (rule: AppMessageRuleGroup) => AppMessageRuleGroup) => void;
  patchRuleDiscordButtonAt: (
    ruleIndex: number,
    buttonIndex: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) => void;
  normalizePositiveNumberInput: (value: string) => number | null;
}) {
  return (
    <details className="discord-advanced-panel rounded-box border border-base-300 bg-base-200/45 p-4">
      <summary className="discord-advanced-summary flex cursor-pointer list-none items-center justify-between gap-3">
        <div>
          <strong className="block font-semibold">Discord add-ons</strong>
          <p className="mt-1 text-sm text-base-content/70">
            The matched rule group can publish buttons, party metadata, or social secrets outside Custom mode. Title subrules can override the buttons only.
          </p>
        </div>
        <div className="discord-advanced-summary-meta">
          {activeRule.buttons.length > 0 ? <span className="badge badge-soft">{activeRule.buttons.length} / 2 buttons</span> : null}
          {activeRuleAdvancedAddonsConfigured ? <span className="badge badge-soft">Configured</span> : null}
          {customOverrideEnabled && customAddonsConfigured ? <span className="badge badge-soft">Custom override</span> : null}
          <span className="discord-advanced-summary-hint" aria-hidden="true">
            <span className="discord-advanced-summary-hint-closed">Expand</span>
            <span className="discord-advanced-summary-hint-open">Collapse</span>
            <span className="discord-advanced-summary-caret">v</span>
          </span>
        </div>
      </summary>
      <div className="mt-4 space-y-4">
        <div className="space-y-3">
          {activeRule.buttons.map((button, index) => (
            <div key={`rule-${activeRuleIndex}-discord-button-${index}`} className="rounded-box border border-base-300 bg-base-100/80 p-3 space-y-3">
              <div className="field-grid">
                <label className={fieldClass}>
                  <span>Button label</span>
                  <input
                    className={inputClass}
                    value={button.label}
                    onChange={(e) => {
                      const value = e.currentTarget.value;
                      patchRuleDiscordButtonAt(activeRuleIndex, index, (current) => ({ ...current, label: value }));
                    }}
                    placeholder="Open website"
                  />
                </label>
                <label className={fieldClass}>
                  <span>Button URL</span>
                  <input
                    className={inputClass}
                    value={button.url}
                    onChange={(e) => {
                      const value = e.currentTarget.value;
                      patchRuleDiscordButtonAt(activeRuleIndex, index, (current) => ({ ...current, url: value }));
                    }}
                    placeholder="https://example.com or myapp://open"
                  />
                </label>
              </div>
              <div className="card-actions justify-end">
                <button
                  className={dangerButtonClass}
                  type="button"
                  onClick={() =>
                    patchRuleAt(activeRuleIndex, (rule) => ({
                      ...rule,
                      buttons: rule.buttons.filter((_, itemIndex) => itemIndex !== index),
                    }))
                  }
                >
                  Remove button
                </button>
              </div>
            </div>
          ))}
          {activeRule.buttons.length < 2 ? (
            <button
              className={buttonClass}
              type="button"
              onClick={() =>
                patchRuleAt(activeRuleIndex, (rule) => ({
                  ...rule,
                  buttons: [...rule.buttons, createDiscordButton()],
                }))
              }
            >
              Add button
            </button>
          ) : null}
        </div>

        <details className="discord-advanced-panel rounded-box border border-base-300 bg-base-100/70 p-4">
          <summary className="discord-advanced-summary flex cursor-pointer list-none items-center justify-between gap-3">
            <div>
              <strong className="block font-semibold">Advanced</strong>
              <p className="mt-1 text-sm text-base-content/70">Party metadata and Discord social secrets.</p>
            </div>
            <div className="discord-advanced-summary-meta">
              {activeRuleAdvancedAddonsConfigured ? <span className="badge badge-soft">Configured</span> : null}
              <span className="discord-advanced-summary-hint" aria-hidden="true">
                <span className="discord-advanced-summary-hint-closed">Expand</span>
                <span className="discord-advanced-summary-hint-open">Collapse</span>
                <span className="discord-advanced-summary-caret">v</span>
              </span>
            </div>
          </summary>
          <div className="field-grid mt-4">
            <label className={fieldClass}>
              <span>Party ID</span>
              <input
                className={inputClass}
                value={activeRule.partyId}
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, partyId: value }));
                }}
                placeholder="party-123"
              />
            </label>
            <label className={fieldClass}>
              <span>Party current size</span>
              <input
                className={inputClass}
                type="number"
                min={1}
                value={activeRule.partySizeCurrent ?? ""}
                onChange={(e) => {
                  const value = normalizePositiveNumberInput(e.currentTarget.value);
                  patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, partySizeCurrent: value }));
                }}
                placeholder="2"
              />
            </label>
            <label className={fieldClass}>
              <span>Party max size</span>
              <input
                className={inputClass}
                type="number"
                min={1}
                value={activeRule.partySizeMax ?? ""}
                onChange={(e) => {
                  const value = normalizePositiveNumberInput(e.currentTarget.value);
                  patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, partySizeMax: value }));
                }}
                placeholder="3"
              />
            </label>
            <label className={fieldSpanClass}>
              <span>Join secret</span>
              <input
                className={inputClass}
                value={activeRule.joinSecret}
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, joinSecret: value }));
                }}
                placeholder="join-secret"
              />
            </label>
            <label className={fieldSpanClass}>
              <span>Spectate secret</span>
              <input
                className={inputClass}
                value={activeRule.spectateSecret}
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, spectateSecret: value }));
                }}
                placeholder="spectate-secret"
              />
            </label>
            <label className={fieldSpanClass}>
              <span>Match secret</span>
              <input
                className={inputClass}
                value={activeRule.matchSecret}
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  patchRuleAt(activeRuleIndex, (rule) => ({ ...rule, matchSecret: value }));
                }}
                placeholder="match-secret"
              />
            </label>
          </div>
        </details>
      </div>
    </details>
  );
}
