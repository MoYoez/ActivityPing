import type { ReactNode } from "react";

export function SettingsFlowView({
  stepOne,
  stepTwo,
  stepThree,
}: {
  stepOne: ReactNode;
  stepTwo: ReactNode;
  stepThree: ReactNode;
}) {
  return (
    <div className="settings-flow">
      <section className="settings-group">
        <div className="settings-group-head">
          <p className="eyebrow">Step 1</p>
          <h3>RPC setup</h3>
          <p>Save the Discord RPC profile first. Runtime depends on this step.</p>
        </div>
        <div className="compact-stack">{stepOne}</div>
      </section>

      <section className="settings-group">
        <div className="settings-group-head">
          <p className="eyebrow">Step 2</p>
          <h3>Rule clauses</h3>
          <p>Open the secondary rule dialog for message clauses, app filter, name-only rules and media blocking.</p>
        </div>
        <div className="compact-stack">{stepTwo}</div>
      </section>

      <section className="settings-group">
        <div className="settings-group-head">
          <p className="eyebrow">Step 3</p>
          <h3>App behavior</h3>
          <p>Keep startup, polling and local monitor behavior at the bottom of the settings page.</p>
        </div>
        <div className="compact-stack">{stepThree}</div>
      </section>
    </div>
  );
}
