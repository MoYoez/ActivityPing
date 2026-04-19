import type { ClientCapabilities, PlatformProbeResult, PlatformSelfTestResult } from "../../types";

function probeBadgeClass(probe: PlatformProbeResult) {
  return probe.success ? "badge badge-success badge-soft" : "badge badge-error badge-soft";
}

export function GeneralSettingsSections({
  capabilities,
  runtimeAutostartEnabled,
  launchOnStartup,
  pollIntervalMs,
  heartbeatIntervalMs,
  platformSelfTest,
  currentLocalModeText,
  ruleGroupCount,
  savedAppsCount,
  mediaSourceCount,
  panelClass,
  panelHeadClass,
  fieldClass,
  inputClass,
  buttonClass,
  statCardClass,
  toggleTileClass,
  busyPlatformSelfTest,
  busyAccessibilityPermission,
  onRuntimeAutostartChange,
  onLaunchOnStartupChange,
  onPollIntervalChange,
  onHeartbeatIntervalChange,
  onHideToTray,
  onRunSelfTest,
  onRequestAccessibilityPermission,
}: {
  capabilities: ClientCapabilities;
  runtimeAutostartEnabled: boolean;
  launchOnStartup: boolean;
  pollIntervalMs: number;
  heartbeatIntervalMs: number;
  platformSelfTest: PlatformSelfTestResult | null;
  currentLocalModeText: string;
  ruleGroupCount: number;
  savedAppsCount: number;
  mediaSourceCount: number;
  panelClass: string;
  panelHeadClass: string;
  fieldClass: string;
  inputClass: string;
  buttonClass: string;
  statCardClass: string;
  toggleTileClass: string;
  busyPlatformSelfTest: boolean;
  busyAccessibilityPermission: boolean;
  onRuntimeAutostartChange: (value: boolean) => void;
  onLaunchOnStartupChange: (value: boolean) => void;
  onPollIntervalChange: (value: string) => void;
  onHeartbeatIntervalChange: (value: string) => void;
  onHideToTray: () => void;
  onRunSelfTest: () => void;
  onRequestAccessibilityPermission: () => void;
}) {
  const probes = platformSelfTest
    ? [
        { label: "Foreground app", probe: platformSelfTest.foreground },
        { label: "Window title", probe: platformSelfTest.windowTitle },
        { label: "Media capture", probe: platformSelfTest.media },
      ]
    : [];

  return (
    <>
      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">Behavior</p>
            <h3>Startup and timing</h3>
          </div>
        </div>
        <div className="toggle-grid">
          <label className={toggleTileClass}>
            <div>
              <strong>Auto-start runtime</strong>
              <span>Start local capture and Discord RPC together when the app launches.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={runtimeAutostartEnabled} onChange={(e) => onRuntimeAutostartChange(e.currentTarget.checked)} />
          </label>
          <label className={toggleTileClass}>
            <div>
              <strong>Launch with system</strong>
              <span>Register the app in the OS startup list.</span>
            </div>
            <input className="toggle toggle-primary" type="checkbox" checked={launchOnStartup} onChange={(e) => onLaunchOnStartupChange(e.currentTarget.checked)} />
          </label>
        </div>
        <div className="field-grid compact-fields">
          <label className={fieldClass}>
            <span>Poll interval (ms)</span>
            <input className={inputClass} type="number" min={1000} value={pollIntervalMs} onChange={(e) => onPollIntervalChange(e.currentTarget.value)} />
          </label>
          <label className={fieldClass}>
            <span>Heartbeat interval (ms)</span>
            <input className={inputClass} type="number" min={0} value={heartbeatIntervalMs} onChange={(e) => onHeartbeatIntervalChange(e.currentTarget.value)} />
          </label>
        </div>
        <div className="card-actions gap-2">
          {capabilities.tray ? (
            <button className={buttonClass} onClick={onHideToTray}>
              Hide to tray
            </button>
          ) : null}
        </div>
      </section>

      {capabilities.platformSelfTest ? (
        <section className={panelClass}>
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">Platform</p>
              <h3>Permissions and self-test</h3>
            </div>
          </div>
          <div className="card-actions gap-2">
            <button
              className={buttonClass}
              type="button"
              disabled={busyPlatformSelfTest}
              onClick={onRunSelfTest}
            >
              {busyPlatformSelfTest ? "Running..." : "Run self-test"}
            </button>
            <button
              className={buttonClass}
              type="button"
              disabled={busyAccessibilityPermission}
              onClick={onRequestAccessibilityPermission}
            >
              {busyAccessibilityPermission ? "Requesting..." : "Request Accessibility Permission"}
            </button>
          </div>
          {platformSelfTest ? (
            <>
              <div className="stat-grid">
                {probes.map(({ label, probe }) => (
                  <div key={label} className={statCardClass}>
                    <span>{label}</span>
                    <strong>{probe.summary}</strong>
                    <div className="platform-probe-badge-row">
                      <span className={probeBadgeClass(probe)}>{probe.success ? "OK" : "Needs attention"}</span>
                    </div>
                  </div>
                ))}
              </div>
              <div className="platform-probe-list">
                {probes.map(({ label, probe }) => (
                  <div key={label} className="empty-state platform-probe-card">
                    <strong>{label}</strong>
                    <p>{probe.detail}</p>
                    {probe.guidance?.length ? (
                      <ul>
                        {probe.guidance.map((item) => (
                          <li key={`${label}-${item}`}>{item}</li>
                        ))}
                      </ul>
                    ) : null}
                  </div>
                ))}
              </div>
            </>
          ) : (
            <div className="empty-state">
              Run the self-test to check foreground app capture, window titles, and media capture on this machine.
            </div>
          )}
        </section>
      ) : null}

      <section className={panelClass}>
        <div className={panelHeadClass}>
          <div>
            <p className="eyebrow">Summary</p>
            <h3>Current local mode</h3>
          </div>
        </div>
        <div className="stat-grid">
          <div className={statCardClass}>
            <span>Working mode</span>
            <strong>{currentLocalModeText}</strong>
          </div>
          <div className={statCardClass}>
            <span>Rule groups</span>
            <strong>{ruleGroupCount}</strong>
          </div>
          <div className={statCardClass}>
            <span>Saved apps</span>
            <strong>{savedAppsCount}</strong>
          </div>
          <div className={statCardClass}>
            <span>Media sources</span>
            <strong>{mediaSourceCount}</strong>
          </div>
        </div>
      </section>
    </>
  );
}
