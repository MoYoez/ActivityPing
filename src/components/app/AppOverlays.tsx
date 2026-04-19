import type { ReactNode } from "react";
import type { ComponentProps } from "react";

import { DiscardChangesDialog } from "../dialogs/DiscardChangesDialog";
import { JsonViewerDialog } from "../dialogs/JsonViewerDialog";
import { DiscordCustomPresetsDialog } from "../discord/DiscordCustomPresetsDialog";
import { DiscordCustomPresetEditorContainer } from "../discord/DiscordCustomPresetEditorContainer";
import { AppNotificationsToast } from "../notifications/AppNotificationsToast";
import { RulesEditorDialog } from "../rules/RulesEditorDialog";

export function AppOverlays({
  rulesDialogOpen,
  rulesEditorDialogProps,
  rulesDialogContent,
  customRulesDialogOpen,
  customPresetsDialogProps,
  customPresetEditorProps,
  discardDialogOpen,
  discardDialogProps,
  jsonViewerDialogProps,
  notificationsProps,
}: {
  rulesDialogOpen: boolean;
  rulesEditorDialogProps: Omit<ComponentProps<typeof RulesEditorDialog>, "children">;
  rulesDialogContent: ReactNode;
  customRulesDialogOpen: boolean;
  customPresetsDialogProps: ComponentProps<typeof DiscordCustomPresetsDialog>;
  customPresetEditorProps: ComponentProps<typeof DiscordCustomPresetEditorContainer> | null;
  discardDialogOpen: boolean;
  discardDialogProps: ComponentProps<typeof DiscardChangesDialog>;
  jsonViewerDialogProps: ComponentProps<typeof JsonViewerDialog> | null;
  notificationsProps: ComponentProps<typeof AppNotificationsToast>;
}) {
  return (
    <>
      {rulesDialogOpen ? <RulesEditorDialog {...rulesEditorDialogProps}>{rulesDialogContent}</RulesEditorDialog> : null}
      {customRulesDialogOpen ? <DiscordCustomPresetsDialog {...customPresetsDialogProps} /> : null}
      {customPresetEditorProps ? <DiscordCustomPresetEditorContainer {...customPresetEditorProps} /> : null}
      {discardDialogOpen ? <DiscardChangesDialog {...discardDialogProps} /> : null}
      {jsonViewerDialogProps ? <JsonViewerDialog {...jsonViewerDialogProps} /> : null}
      <AppNotificationsToast {...notificationsProps} />
    </>
  );
}
