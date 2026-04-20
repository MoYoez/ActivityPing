import type { SetStateAction } from "react";
import { create } from "zustand";

import { DEFAULT_CAPABILITIES, EMPTY_DISCORD, EMPTY_REPORTER } from "../app/appConstants";
import { defaultClientConfig } from "../lib/api";
import type {
  AppStatePayload,
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../types";
import type { ViewSection } from "../components/pages/pageSections";

export type NoticeTone = "info" | "success" | "warn" | "error";

export interface Notice {
  id: number;
  tone: NoticeTone;
  title: string;
  detail: string;
}

export interface JsonViewerState {
  eyebrow: string;
  title: string;
  description: string;
  value: unknown | null;
  emptyText: string;
}

function resolveState<T>(value: SetStateAction<T>, current: T) {
  return typeof value === "function" ? (value as (current: T) => T)(current) : value;
}

interface AppUiStore {
  capabilities: ClientCapabilities;
  baseState: AppStatePayload;
  config: ClientConfig;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordSnapshot: DiscordPresenceSnapshot;
  platformSelfTest: PlatformSelfTestResult | null;
  notices: Notice[];
  hydrated: boolean;
  busy: Record<string, boolean>;
  activeSection: ViewSection;
  activeRuleIndex: number;
  rulesImportOpen: boolean;
  rulesImportValue: string;
  blacklistInput: string;
  whitelistInput: string;
  nameOnlyInput: string;
  mediaSourceInput: string;
  rulesDialogOpen: boolean;
  customRulesDialogOpen: boolean;
  customPresetPage: number;
  activeCustomPresetIndex: number | null;
  appliedCustomPresetIndex: number | null;
  discordDetailsForceCustomChoice: boolean;
  discordStateForceCustomChoice: boolean;
  presetDetailsForceCustomChoice: boolean;
  presetStateForceCustomChoice: boolean;
  discardDialogOpen: boolean;
  appliedRuntimeConfigSignature: string | null;
  ruleGroupPage: number;
  titleRulePage: number;
  runtimeLogPage: number;
  jsonViewer: JsonViewerState | null;
  setCapabilities: (value: SetStateAction<ClientCapabilities>) => void;
  setBaseState: (value: SetStateAction<AppStatePayload>) => void;
  setConfig: (value: SetStateAction<ClientConfig>) => void;
  setReporterSnapshot: (value: SetStateAction<RealtimeReporterSnapshot>) => void;
  setDiscordSnapshot: (value: SetStateAction<DiscordPresenceSnapshot>) => void;
  setPlatformSelfTest: (value: SetStateAction<PlatformSelfTestResult | null>) => void;
  setHydrated: (value: SetStateAction<boolean>) => void;
  setBusy: (value: SetStateAction<Record<string, boolean>>) => void;
  setActiveSection: (value: SetStateAction<ViewSection>) => void;
  setActiveRuleIndex: (value: SetStateAction<number>) => void;
  setRulesImportOpen: (value: SetStateAction<boolean>) => void;
  setRulesImportValue: (value: SetStateAction<string>) => void;
  setBlacklistInput: (value: SetStateAction<string>) => void;
  setWhitelistInput: (value: SetStateAction<string>) => void;
  setNameOnlyInput: (value: SetStateAction<string>) => void;
  setMediaSourceInput: (value: SetStateAction<string>) => void;
  setRulesDialogOpen: (value: SetStateAction<boolean>) => void;
  setCustomRulesDialogOpen: (value: SetStateAction<boolean>) => void;
  setCustomPresetPage: (value: SetStateAction<number>) => void;
  setActiveCustomPresetIndex: (value: SetStateAction<number | null>) => void;
  setAppliedCustomPresetIndex: (value: SetStateAction<number | null>) => void;
  setDiscordDetailsForceCustomChoice: (value: SetStateAction<boolean>) => void;
  setDiscordStateForceCustomChoice: (value: SetStateAction<boolean>) => void;
  setPresetDetailsForceCustomChoice: (value: SetStateAction<boolean>) => void;
  setPresetStateForceCustomChoice: (value: SetStateAction<boolean>) => void;
  setDiscardDialogOpen: (value: SetStateAction<boolean>) => void;
  setAppliedRuntimeConfigSignature: (value: SetStateAction<string | null>) => void;
  setRuleGroupPage: (value: SetStateAction<number>) => void;
  setTitleRulePage: (value: SetStateAction<number>) => void;
  setRuntimeLogPage: (value: SetStateAction<number>) => void;
  setJsonViewer: (value: SetStateAction<JsonViewerState | null>) => void;
  addNotice: (notice: Notice) => void;
  removeNotice: (id: number) => void;
}

const INITIAL_BASE_STATE: AppStatePayload = {
  config: defaultClientConfig(),
  appHistory: [],
  playSourceHistory: [],
  locale: "en-US",
};

export const useAppUiStore = create<AppUiStore>((set) => ({
  capabilities: DEFAULT_CAPABILITIES,
  baseState: INITIAL_BASE_STATE,
  config: defaultClientConfig(),
  reporterSnapshot: EMPTY_REPORTER,
  discordSnapshot: EMPTY_DISCORD,
  platformSelfTest: null,
  notices: [],
  hydrated: false,
  busy: {},
  activeSection: "runtime",
  activeRuleIndex: 0,
  rulesImportOpen: false,
  rulesImportValue: "",
  blacklistInput: "",
  whitelistInput: "",
  nameOnlyInput: "",
  mediaSourceInput: "",
  rulesDialogOpen: false,
  customRulesDialogOpen: false,
  customPresetPage: 0,
  activeCustomPresetIndex: null,
  appliedCustomPresetIndex: null,
  discordDetailsForceCustomChoice: false,
  discordStateForceCustomChoice: false,
  presetDetailsForceCustomChoice: false,
  presetStateForceCustomChoice: false,
  discardDialogOpen: false,
  appliedRuntimeConfigSignature: null,
  ruleGroupPage: 0,
  titleRulePage: 0,
  runtimeLogPage: 0,
  jsonViewer: null,
  setCapabilities: (value) => set((state) => ({ capabilities: resolveState(value, state.capabilities) })),
  setBaseState: (value) => set((state) => ({ baseState: resolveState(value, state.baseState) })),
  setConfig: (value) => set((state) => ({ config: resolveState(value, state.config) })),
  setReporterSnapshot: (value) => set((state) => ({ reporterSnapshot: resolveState(value, state.reporterSnapshot) })),
  setDiscordSnapshot: (value) => set((state) => ({ discordSnapshot: resolveState(value, state.discordSnapshot) })),
  setPlatformSelfTest: (value) => set((state) => ({ platformSelfTest: resolveState(value, state.platformSelfTest) })),
  setHydrated: (value) => set((state) => ({ hydrated: resolveState(value, state.hydrated) })),
  setBusy: (value) => set((state) => ({ busy: resolveState(value, state.busy) })),
  setActiveSection: (value) => set((state) => ({ activeSection: resolveState(value, state.activeSection) })),
  setActiveRuleIndex: (value) => set((state) => ({ activeRuleIndex: resolveState(value, state.activeRuleIndex) })),
  setRulesImportOpen: (value) => set((state) => ({ rulesImportOpen: resolveState(value, state.rulesImportOpen) })),
  setRulesImportValue: (value) => set((state) => ({ rulesImportValue: resolveState(value, state.rulesImportValue) })),
  setBlacklistInput: (value) => set((state) => ({ blacklistInput: resolveState(value, state.blacklistInput) })),
  setWhitelistInput: (value) => set((state) => ({ whitelistInput: resolveState(value, state.whitelistInput) })),
  setNameOnlyInput: (value) => set((state) => ({ nameOnlyInput: resolveState(value, state.nameOnlyInput) })),
  setMediaSourceInput: (value) => set((state) => ({ mediaSourceInput: resolveState(value, state.mediaSourceInput) })),
  setRulesDialogOpen: (value) => set((state) => ({ rulesDialogOpen: resolveState(value, state.rulesDialogOpen) })),
  setCustomRulesDialogOpen: (value) =>
    set((state) => ({ customRulesDialogOpen: resolveState(value, state.customRulesDialogOpen) })),
  setCustomPresetPage: (value) => set((state) => ({ customPresetPage: resolveState(value, state.customPresetPage) })),
  setActiveCustomPresetIndex: (value) =>
    set((state) => ({ activeCustomPresetIndex: resolveState(value, state.activeCustomPresetIndex) })),
  setAppliedCustomPresetIndex: (value) =>
    set((state) => ({ appliedCustomPresetIndex: resolveState(value, state.appliedCustomPresetIndex) })),
  setDiscordDetailsForceCustomChoice: (value) =>
    set((state) => ({ discordDetailsForceCustomChoice: resolveState(value, state.discordDetailsForceCustomChoice) })),
  setDiscordStateForceCustomChoice: (value) =>
    set((state) => ({ discordStateForceCustomChoice: resolveState(value, state.discordStateForceCustomChoice) })),
  setPresetDetailsForceCustomChoice: (value) =>
    set((state) => ({ presetDetailsForceCustomChoice: resolveState(value, state.presetDetailsForceCustomChoice) })),
  setPresetStateForceCustomChoice: (value) =>
    set((state) => ({ presetStateForceCustomChoice: resolveState(value, state.presetStateForceCustomChoice) })),
  setDiscardDialogOpen: (value) => set((state) => ({ discardDialogOpen: resolveState(value, state.discardDialogOpen) })),
  setAppliedRuntimeConfigSignature: (value) =>
    set((state) => ({ appliedRuntimeConfigSignature: resolveState(value, state.appliedRuntimeConfigSignature) })),
  setRuleGroupPage: (value) => set((state) => ({ ruleGroupPage: resolveState(value, state.ruleGroupPage) })),
  setTitleRulePage: (value) => set((state) => ({ titleRulePage: resolveState(value, state.titleRulePage) })),
  setRuntimeLogPage: (value) => set((state) => ({ runtimeLogPage: resolveState(value, state.runtimeLogPage) })),
  setJsonViewer: (value) => set((state) => ({ jsonViewer: resolveState(value, state.jsonViewer) })),
  addNotice: (notice) => set((state) => ({ notices: [...state.notices, notice] })),
  removeNotice: (id) => set((state) => ({ notices: state.notices.filter((notice) => notice.id !== id) })),
}));
