import type { ComponentProps } from "react";

import { AboutPage } from "../components/pages/AboutPage";
import { RuntimePage } from "../components/pages/RuntimePage";
import type { DiscordDebugPayload, ReporterLogEntry } from "../types";

interface CreateRuntimeViewPropsArgs {
  runtimeReady: boolean;
  runtimeBlockReason: string | null;
  runtimeRunning: boolean;
  currentActivityText: string;
  attachedMeta: string;
  captureModeText: string;
  lastHeartbeatText: string;
  runtimeLogs: ReporterLogEntry[];
  visibleRuntimeLogs: ReporterLogEntry[];
  runtimeLogPageStart: number;
  runtimeLogPageEnd: number;
  safeRuntimeLogPage: number;
  runtimeLogPageCount: number;
  discordDebugPayload: DiscordDebugPayload | null;
  panelClass: string;
  panelHeadClass: string;
  statCardClass: string;
  primaryButtonClass: string;
  buttonClass: string;
  goodBadgeClass: string;
  badgeClass: string;
  appIconSrc: string;
  githubUrl: string;
  openGithubBusy: boolean;
  startBusy: boolean;
  stopBusy: boolean;
  refreshBusy: boolean;
  restartBusy: boolean;
  formatDate: (value?: string | null) => string;
  logEntryClass: (level: string) => string;
  onOpenSettings: () => void;
  onStart: () => void;
  onStop: () => void;
  onRefresh: () => void;
  onRuntimeLogPageChange: (page: number) => void;
  onOpenLogPayload: (entry: ReporterLogEntry) => void;
  onOpenDiscordPayload: () => void;
  onOpenGithub: () => void;
}

export function createRuntimeViewProps(args: CreateRuntimeViewPropsArgs) {
  const runtimePageProps: ComponentProps<typeof RuntimePage> = {
    prerequisiteCardProps: args.runtimeReady
      ? null
      : {
          panelClass: args.panelClass,
          panelHeadClass: args.panelHeadClass,
          primaryButtonClass: args.primaryButtonClass,
          runtimeBlockReason: args.runtimeBlockReason,
          onOpenSettings: args.onOpenSettings,
        },
    monitorCardProps: {
      panelClass: args.panelClass,
      panelHeadClass: args.panelHeadClass,
      statCardClass: args.statCardClass,
      primaryButtonClass: args.primaryButtonClass,
      buttonClass: args.buttonClass,
      goodBadgeClass: args.goodBadgeClass,
      badgeClass: args.badgeClass,
      runtimeRunning: args.runtimeRunning,
      currentActivity: args.currentActivityText,
      attachedMeta: args.attachedMeta,
      captureMode: args.captureModeText,
      lastHeartbeat: args.lastHeartbeatText,
      startBusy: args.startBusy,
      stopBusy: args.stopBusy,
      refreshBusy: args.refreshBusy,
      restartBusy: args.restartBusy,
      runtimeReady: args.runtimeReady,
      onStart: args.onStart,
      onStop: args.onStop,
      onRefresh: args.onRefresh,
    },
    logCardProps: {
      panelClass: args.panelClass,
      panelHeadClass: args.panelHeadClass,
      runtimeLogs: args.runtimeLogs,
      visibleRuntimeLogs: args.visibleRuntimeLogs,
      runtimeLogPageStart: args.runtimeLogPageStart,
      runtimeLogPageEnd: args.runtimeLogPageEnd,
      safeRuntimeLogPage: args.safeRuntimeLogPage,
      runtimeLogPageCount: args.runtimeLogPageCount,
      formatDate: args.formatDate,
      logEntryClass: args.logEntryClass,
      onOpenLogPayload: args.onOpenLogPayload,
      onRuntimeLogPageChange: args.onRuntimeLogPageChange,
    },
    debugCardProps: {
      panelClass: args.panelClass,
      panelHeadClass: args.panelHeadClass,
      buttonClass: args.buttonClass,
      discordDebugPayload: args.discordDebugPayload,
      onOpenDiscordPayload: args.onOpenDiscordPayload,
    },
  };

  const aboutPageProps: ComponentProps<typeof AboutPage> = {
    appIconSrc: args.appIconSrc,
    githubUrl: args.githubUrl,
    openGithubBusy: args.openGithubBusy,
    primaryButtonClass: args.primaryButtonClass,
    onOpenGithub: args.onOpenGithub,
  };

  return {
    runtimePageProps,
    aboutPageProps,
  };
}
