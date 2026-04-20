import type { ComponentProps, Dispatch, SetStateAction } from "react";

import { DiscardChangesDialog } from "../components/dialogs/DiscardChangesDialog";
import { JsonViewerDialog } from "../components/dialogs/JsonViewerDialog";
import { DiscordCustomPresetsDialog } from "../components/discord/DiscordCustomPresetsDialog";
import { DiscordCustomPresetEditorContainer } from "../components/discord/DiscordCustomPresetEditorContainer";
import { AppNotificationsToast } from "../components/notifications/AppNotificationsToast";
import { RulesEditorDialog } from "../components/rules/RulesEditorDialog";
import type { ClientConfig, DiscordCustomPreset } from "../types";

import { createDiscordButton, createDiscordCustomPreset } from "./appConfig";
import { CUSTOM_PRESET_PAGE_SIZE } from "./appConstants";
import { clampPage, moveItem, pageForIndex } from "./appFormatting";

interface JsonViewerState {
  eyebrow: string;
  title: string;
  description: string;
  value: unknown | null;
  emptyText: string;
}

interface CreateOverlayPropsArgs {
  config: ClientConfig;
  activeCustomPresetIndex: number | null;
  pagedCustomPresets: DiscordCustomPreset[];
  customPresetPageStart: number;
  safeCustomPresetPage: number;
  customPresetTotalPages: number;
  activeCustomPreset: DiscordCustomPreset | null;
  activeCustomPresetAdvancedAddonsConfigured: boolean;
  presetDetailsForceCustomChoice: boolean;
  presetStateForceCustomChoice: boolean;
  rulesDialogOpen: boolean;
  customRulesDialogOpen: boolean;
  discardDialogOpen: boolean;
  jsonViewer: JsonViewerState | null;
  jsonViewerJson: string;
  runtimeNeedsRestart: boolean;
  dirty: boolean;
  notices: Array<{ id: number; tone: "info" | "success" | "warn" | "error"; title: string; detail: string }>;
  busy: Record<string, boolean>;
  panelHeadClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  subruleCardClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  normalizePositiveNumberInput: (value: string) => number | null;
  summarizePreset: (preset: DiscordCustomPreset) => string;
  alertClass: (tone: "info" | "success" | "warn" | "error") => string;
  applyPreset: (preset: DiscordCustomPreset) => void;
  patchPresetAt: (index: number, updater: (preset: DiscordCustomPreset) => DiscordCustomPreset) => void;
  onCloseRulesDialog: () => void;
  onSaveCurrentAsPreset: () => void;
  onCloseCustomPresets: () => void;
  onCloseDiscardDialog: () => void;
  onConfirmDiscard: () => void;
  onCloseJsonViewer: () => void;
  onRestartRuntime: () => void;
  onOpenDiscardDialog: () => void;
  onSaveDraft: () => void;
  setConfig: Dispatch<SetStateAction<ClientConfig>>;
  setCustomPresetPage: Dispatch<SetStateAction<number>>;
  setActiveCustomPresetIndex: Dispatch<SetStateAction<number | null>>;
  setCustomRulesDialogOpen: Dispatch<SetStateAction<boolean>>;
  setPresetDetailsForceCustomChoice: Dispatch<SetStateAction<boolean>>;
  setPresetStateForceCustomChoice: Dispatch<SetStateAction<boolean>>;
}

export function createOverlayProps(args: CreateOverlayPropsArgs) {
  const rulesEditorDialogProps: Omit<ComponentProps<typeof RulesEditorDialog>, "children"> = {
    panelHeadClass: args.panelHeadClass,
    buttonClass: args.buttonClass,
    onClose: args.onCloseRulesDialog,
  };

  const customPresetsDialogProps: ComponentProps<typeof DiscordCustomPresetsDialog> = {
    presets: args.config.discordCustomPresets,
    activePresetIndex: args.activeCustomPresetIndex,
    pagedPresets: args.pagedCustomPresets,
    presetPageStart: args.customPresetPageStart,
    presetPageSize: CUSTOM_PRESET_PAGE_SIZE,
    safePresetPage: args.safeCustomPresetPage,
    presetTotalPages: args.customPresetTotalPages,
    panelHeadClass: args.panelHeadClass,
    subruleCardClass: args.subruleCardClass,
    buttonClass: args.buttonClass,
    primaryButtonClass: args.primaryButtonClass,
    dangerButtonClass: args.dangerButtonClass,
    summarizePreset: args.summarizePreset,
    onClose: () => {
      args.setActiveCustomPresetIndex(null);
      args.setCustomRulesDialogOpen(false);
    },
    onSaveCurrentAsPreset: args.onSaveCurrentAsPreset,
    onAddPreset: () => {
      const nextIndex = args.config.discordCustomPresets.length;
      args.setConfig((current) => ({
        ...current,
        discordCustomPresets: [...current.discordCustomPresets, createDiscordCustomPreset()],
      }));
      args.setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
      args.setActiveCustomPresetIndex(nextIndex);
    },
    onOpenPreset: (index) => args.setActiveCustomPresetIndex(index),
    onMovePresetUp: (index) => {
      const nextIndex = index - 1;
      args.setConfig((current) => ({
        ...current,
        discordCustomPresets: moveItem(current.discordCustomPresets, index, nextIndex),
      }));
      args.setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
    },
    onMovePresetDown: (index) => {
      const nextIndex = index + 1;
      args.setConfig((current) => ({
        ...current,
        discordCustomPresets: moveItem(current.discordCustomPresets, index, nextIndex),
      }));
      args.setCustomPresetPage(pageForIndex(nextIndex, CUSTOM_PRESET_PAGE_SIZE));
    },
    onRemovePreset: (index) => {
      args.setConfig((current) => ({
        ...current,
        discordCustomPresets: current.discordCustomPresets.filter((_, itemIndex) => itemIndex !== index),
      }));
      if (args.activeCustomPresetIndex === index) {
        args.setActiveCustomPresetIndex(null);
      }
    },
    onPresetPageChange: (page) =>
      args.setCustomPresetPage(() => clampPage(page, args.config.discordCustomPresets.length, CUSTOM_PRESET_PAGE_SIZE)),
  };

  const customPresetEditorProps: ComponentProps<typeof DiscordCustomPresetEditorContainer> | null =
    args.activeCustomPreset && args.activeCustomPresetIndex !== null
      ? {
          preset: args.activeCustomPreset,
          presetIndex: args.activeCustomPresetIndex,
          assets: args.config.discordCustomAssets,
          detailsForceCustomChoice: args.presetDetailsForceCustomChoice,
          stateForceCustomChoice: args.presetStateForceCustomChoice,
          presetAdvancedAddonsConfigured: args.activeCustomPresetAdvancedAddonsConfigured,
          panelHeadClass: args.panelHeadClass,
          fieldClass: args.fieldClass,
          fieldSpanClass: args.fieldSpanClass,
          inputClass: args.inputClass,
          selectClass: args.selectClass,
          buttonClass: args.buttonClass,
          primaryButtonClass: args.primaryButtonClass,
          dangerButtonClass: args.dangerButtonClass,
          normalizePositiveNumberInput: args.normalizePositiveNumberInput,
          patchPresetAt: args.patchPresetAt,
          createDiscordButton,
          applyPreset: args.applyPreset,
          onClose: () => args.setActiveCustomPresetIndex(null),
          onCloseCustomPresets: args.onCloseCustomPresets,
          setDetailsForceCustomChoice: args.setPresetDetailsForceCustomChoice,
          setStateForceCustomChoice: args.setPresetStateForceCustomChoice,
        }
      : null;

  const discardDialogProps: ComponentProps<typeof DiscardChangesDialog> = {
    panelHeadClass: args.panelHeadClass,
    buttonClass: args.buttonClass,
    dangerButtonClass: args.dangerButtonClass,
    onClose: args.onCloseDiscardDialog,
    onConfirm: args.onConfirmDiscard,
  };

  const jsonViewerDialogProps: ComponentProps<typeof JsonViewerDialog> | null = args.jsonViewer
    ? {
        panelHeadClass: args.panelHeadClass,
        buttonClass: args.buttonClass,
        eyebrow: args.jsonViewer.eyebrow,
        title: args.jsonViewer.title,
        description: args.jsonViewer.description,
        emptyText: args.jsonViewer.emptyText,
        hasValue: Boolean(args.jsonViewer.value),
        valueJson: args.jsonViewerJson,
        onClose: args.onCloseJsonViewer,
      }
    : null;

  const notificationsProps: ComponentProps<typeof AppNotificationsToast> = {
    runtimeNeedsRestart: args.runtimeNeedsRestart,
    dirty: args.dirty,
    notices: args.notices,
    primaryButtonClass: args.primaryButtonClass,
    buttonClass: args.buttonClass,
    restartRuntimeBusy: args.busy.restartRuntime,
    startRuntimeBusy: args.busy.startRuntime,
    stopRuntimeBusy: args.busy.stopRuntime,
    saveDraftBusy: args.busy.saveDraft,
    alertClass: args.alertClass,
    onRestartRuntime: args.onRestartRuntime,
    onOpenDiscardDialog: args.onOpenDiscardDialog,
    onSaveDraft: args.onSaveDraft,
  };

  return {
    rulesEditorDialogProps,
    customPresetsDialogProps,
    customPresetEditorProps,
    discardDialogProps,
    jsonViewerDialogProps,
    notificationsProps,
  };
}
