import { configSignature, limitReporterSnapshotLogs } from "./appFormatting";
import {
  getDiscordPresenceSnapshot,
  getRealtimeReporterSnapshot,
  requestAccessibilityPermission,
  resolveApiError,
  runPlatformSelfTest,
  startDiscordPresenceSync,
  startRealtimeReporter,
  stopDiscordPresenceSync,
  stopRealtimeReporter,
} from "../lib/api";
import { normalizeClientConfig } from "../lib/rules";
import type { NoticeTone } from "../store/appUiStore";
import type {
  ClientConfig,
  DiscordPresenceSnapshot,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../types";

interface UseRuntimeActionsArgs {
  config: ClientConfig;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordSnapshot: DiscordPresenceSnapshot;
  runtimeReady: boolean;
  runtimeBlockReason: string | null;
  notify: (tone: NoticeTone, title: string, detail: string) => void;
  setReporterSnapshot: (snapshot: RealtimeReporterSnapshot) => void;
  setDiscordSnapshot: (snapshot: DiscordPresenceSnapshot) => void;
  setPlatformSelfTest: (result: PlatformSelfTestResult) => void;
  setAppliedRuntimeConfigSignature: (value: string | null) => void;
}

export function useRuntimeActions({
  config,
  reporterSnapshot,
  discordSnapshot,
  runtimeReady,
  runtimeBlockReason,
  notify,
  setReporterSnapshot,
  setDiscordSnapshot,
  setPlatformSelfTest,
  setAppliedRuntimeConfigSignature,
}: UseRuntimeActionsArgs) {
  async function refreshReporter() {
    const result = await getRealtimeReporterSnapshot();
    if (result.success && result.data) setReporterSnapshot(limitReporterSnapshotLogs(result.data));
  }

  async function refreshDiscord() {
    const result = await getDiscordPresenceSnapshot();
    if (result.success && result.data) setDiscordSnapshot(result.data);
  }

  async function refreshPlatformSelfTest(showNotice = false) {
    const result = await runPlatformSelfTest();
    if (!result.success || !result.data) {
      if (showNotice) {
        notify("error", "Self-test failed", resolveApiError(result, "Platform self-test could not be completed."));
      }
      return;
    }

    setPlatformSelfTest(result.data);
    if (showNotice) {
      notify("info", "Self-test updated", "Platform capture checks were refreshed.");
    }
  }

  async function requestPlatformAccessibilityPermission() {
    const result = await requestAccessibilityPermission();
    if (!result.success) {
      notify(
        "warn",
        "Permission request unavailable",
        resolveApiError(result, "Accessibility permission could not be requested."),
      );
      return;
    }

    if (result.data) {
      notify("success", "Permission granted", "Accessibility permission is available for window title capture.");
    } else {
      notify(
        "info",
        "Permission prompt opened",
        "Grant Accessibility permission in System Settings > Privacy & Security > Accessibility, then refresh the self-test.",
      );
    }

    await refreshPlatformSelfTest(false);
  }

  async function startRuntimeSession() {
    if (!runtimeReady) {
      notify("warn", "Runtime locked", runtimeBlockReason || "Save the RPC settings first.");
      return;
    }

    const normalized = normalizeClientConfig(config);
    let startedDiscordThisRun = false;

    if (!discordSnapshot.running) {
      const discordResult = await startDiscordPresenceSync(normalized);
      if (!discordResult.success || !discordResult.data) {
        notify("error", "Runtime start failed", resolveApiError(discordResult, "Discord RPC could not be started."));
        return;
      }
      startedDiscordThisRun = true;
      setDiscordSnapshot(discordResult.data);
    }

    if (!reporterSnapshot.running) {
      const reporterResult = await startRealtimeReporter(normalized);
      if (!reporterResult.success || !reporterResult.data) {
        if (startedDiscordThisRun) {
          const rollbackResult = await stopDiscordPresenceSync();
          if (rollbackResult.success && rollbackResult.data) {
            setDiscordSnapshot(rollbackResult.data);
          }
        }
        notify(
          "error",
          "Runtime start failed",
          resolveApiError(reporterResult, "The local monitor could not be started."),
        );
        return;
      }
      setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
    }

    setAppliedRuntimeConfigSignature(configSignature(normalized));
    notify("success", "Runtime online", "Local capture and Discord RPC are now running.");
  }

  async function stopRuntimeSession() {
    const failures: string[] = [];

    if (reporterSnapshot.running) {
      const reporterResult = await stopRealtimeReporter();
      if (!reporterResult.success || !reporterResult.data) {
        failures.push(resolveApiError(reporterResult, "The local monitor could not be stopped."));
      } else {
        setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
      }
    }

    if (discordSnapshot.running) {
      const discordResult = await stopDiscordPresenceSync();
      if (!discordResult.success || !discordResult.data) {
        failures.push(resolveApiError(discordResult, "Discord RPC could not be stopped."));
      } else {
        setDiscordSnapshot(discordResult.data);
      }
    }

    if (failures.length > 0) {
      notify("warn", "Runtime partially stopped", failures[0]);
      return;
    }

    setAppliedRuntimeConfigSignature(null);
    notify("info", "Runtime stopped", "Local capture and Discord RPC have been stopped.");
  }

  async function restartRuntimeSession() {
    if (!runtimeReady) {
      notify("warn", "Runtime locked", runtimeBlockReason || "Save the RPC settings first.");
      return;
    }

    const normalized = normalizeClientConfig(config);
    const failures: string[] = [];

    if (reporterSnapshot.running) {
      const reporterResult = await stopRealtimeReporter();
      if (!reporterResult.success || !reporterResult.data) {
        failures.push(resolveApiError(reporterResult, "The local monitor could not be stopped."));
      } else {
        setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
      }
    }

    if (discordSnapshot.running) {
      const discordResult = await stopDiscordPresenceSync();
      if (!discordResult.success || !discordResult.data) {
        failures.push(resolveApiError(discordResult, "Discord RPC could not be stopped."));
      } else {
        setDiscordSnapshot(discordResult.data);
      }
    }

    if (failures.length > 0) {
      notify("warn", "Runtime restart blocked", failures[0]);
      return;
    }

    const discordResult = await startDiscordPresenceSync(normalized);
    if (!discordResult.success || !discordResult.data) {
      setAppliedRuntimeConfigSignature(null);
      notify("error", "Runtime restart failed", resolveApiError(discordResult, "Discord RPC could not be started."));
      return;
    }
    setDiscordSnapshot(discordResult.data);

    const reporterResult = await startRealtimeReporter(normalized);
    if (!reporterResult.success || !reporterResult.data) {
      const rollbackResult = await stopDiscordPresenceSync();
      if (rollbackResult.success && rollbackResult.data) {
        setDiscordSnapshot(rollbackResult.data);
      }
      setAppliedRuntimeConfigSignature(null);
      notify(
        "error",
        "Runtime restart failed",
        resolveApiError(reporterResult, "The local monitor could not be started."),
      );
      return;
    }

    setReporterSnapshot(limitReporterSnapshotLogs(reporterResult.data));
    setAppliedRuntimeConfigSignature(configSignature(normalized));
    notify("success", "Runtime restarted", "Saved configuration changes are now running.");
  }

  return {
    refreshReporter,
    refreshDiscord,
    refreshPlatformSelfTest,
    requestPlatformAccessibilityPermission,
    startRuntimeSession,
    stopRuntimeSession,
    restartRuntimeSession,
  };
}
