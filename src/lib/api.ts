import { invoke } from "@tauri-apps/api/core";
import {
  disable as disableAutostartPlugin,
  enable as enableAutostartPlugin,
  isEnabled as isAutostartEnabledPlugin,
} from "@tauri-apps/plugin-autostart";

import type {
  ApiResult,
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../types";

const ERROR_MESSAGES: Record<string, string> = {
  "backendErrors.reporterWorkerStopping": "The local monitor is still stopping. Try again in a moment.",
  "backendErrors.discordConfigAppIdMissing": "Discord application ID is required before Discord RPC can start.",
  "backendErrors.discordWorkerStopping": "Discord RPC is still stopping. Try again in a moment.",
  "backendErrors.accessibilityPermissionUnsupported":
    "Accessibility permission requests are not supported on this platform.",
};

function toInvokeError(message: string, details?: unknown): ApiResult<never> {
  return {
    success: false,
    status: 0,
    error: {
      status: 0,
      message,
      details,
    },
  };
}

async function invokeApi<T>(command: string, args?: Record<string, unknown>): Promise<ApiResult<T>> {
  try {
    return await invoke<ApiResult<T>>(command, args);
  } catch (error) {
    return toInvokeError(error instanceof Error ? error.message : "The Tauri command failed.", error);
  }
}

export function resolveApiError(result: ApiResult<unknown>, fallback: string): string {
  const code = result.error?.code ?? "";
  if (code && ERROR_MESSAGES[code]) {
    return ERROR_MESSAGES[code];
  }

  return result.error?.message?.trim() || fallback;
}

export function defaultClientConfig(): ClientConfig {
  return {
    pollIntervalMs: 5000,
    heartbeatIntervalMs: 60000,
    runtimeAutostartEnabled: false,
    reportForegroundApp: true,
    reportWindowTitle: true,
    reportMedia: true,
    reportPlaySource: true,
    discordApplicationId: "",
    discordReportMode: "mixed",
    discordActivityType: "playing",
    discordSmartEnableMusicCountdown: true,
    discordUseMediaArtwork: false,
    discordArtworkWorkerUploadUrl: "",
    discordArtworkWorkerToken: "",
    discordDetailsFormat: "{activity}",
    discordStateFormat: "{context}",
    launchOnStartup: false,
    captureReportedAppsEnabled: true,
    appMessageRules: [],
    appMessageRulesShowProcessName: true,
    appFilterMode: "blacklist",
    appBlacklist: [],
    appWhitelist: [],
    appNameOnlyList: [],
    mediaPlaySourceBlocklist: [],
  };
}

export async function loadAppState(): Promise<AppStatePayload> {
  try {
    return await invoke<AppStatePayload>("load_app_state");
  } catch {
    return {
      config: defaultClientConfig(),
      appHistory: [],
      playSourceHistory: [],
      locale: "en-US",
    };
  }
}

export async function saveAppState(payload: AppStatePayload) {
  await invoke("save_app_state", { payload });
}

export async function getClientCapabilities(): Promise<ApiResult<ClientCapabilities>> {
  return invokeApi("get_client_capabilities");
}

export async function startRealtimeReporter(
  config: ClientConfig,
): Promise<ApiResult<RealtimeReporterSnapshot>> {
  return invokeApi("start_realtime_reporter", { config });
}

export async function stopRealtimeReporter(): Promise<ApiResult<RealtimeReporterSnapshot>> {
  return invokeApi("stop_realtime_reporter");
}

export async function getRealtimeReporterSnapshot(): Promise<ApiResult<RealtimeReporterSnapshot>> {
  return invokeApi("get_realtime_reporter_snapshot");
}

export async function startDiscordPresenceSync(
  config: ClientConfig,
): Promise<ApiResult<DiscordPresenceSnapshot>> {
  return invokeApi("start_discord_presence_sync", { config });
}

export async function stopDiscordPresenceSync(): Promise<ApiResult<DiscordPresenceSnapshot>> {
  return invokeApi("stop_discord_presence_sync");
}

export async function getDiscordPresenceSnapshot(): Promise<ApiResult<DiscordPresenceSnapshot>> {
  return invokeApi("get_discord_presence_snapshot");
}

export async function runPlatformSelfTest(): Promise<ApiResult<PlatformSelfTestResult>> {
  return invokeApi("run_platform_self_test");
}

export async function requestAccessibilityPermission(): Promise<ApiResult<boolean>> {
  return invokeApi("request_accessibility_permission");
}

export async function hideToTray() {
  await invoke("hide_to_tray");
}

export async function isAutostartEnabled(): Promise<boolean> {
  return isAutostartEnabledPlugin();
}

export async function setAutostartEnabled(enabled: boolean) {
  if (enabled) {
    await enableAutostartPlugin();
    return;
  }

  await disableAutostartPlugin();
}
