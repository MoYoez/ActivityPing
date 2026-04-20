import type {
  DiscordCustomAsset,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
} from "../../types";
import { DiscordCustomPresetEditorModal } from "./DiscordCustomPresetEditorModal";

export function DiscordCustomPresetEditorContainer({
  preset,
  presetIndex,
  assets,
  detailsForceCustomChoice,
  stateForceCustomChoice,
  presetAdvancedAddonsConfigured,
  panelHeadClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  selectClass,
  buttonClass,
  primaryButtonClass,
  dangerButtonClass,
  normalizePositiveNumberInput,
  patchPresetAt,
  createDiscordButton,
  applyPreset,
  onClose,
  onCloseCustomPresets,
  setDetailsForceCustomChoice,
  setStateForceCustomChoice,
}: {
  preset: DiscordCustomPreset;
  presetIndex: number;
  assets: DiscordCustomAsset[];
  detailsForceCustomChoice: boolean;
  stateForceCustomChoice: boolean;
  presetAdvancedAddonsConfigured: boolean;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  dangerButtonClass: string;
  normalizePositiveNumberInput: (value: string) => number | null;
  patchPresetAt: (index: number, updater: (preset: DiscordCustomPreset) => DiscordCustomPreset) => void;
  createDiscordButton: () => DiscordRichPresenceButtonConfig;
  applyPreset: (preset: DiscordCustomPreset) => void;
  onClose: () => void;
  onCloseCustomPresets: () => void;
  setDetailsForceCustomChoice: (value: boolean) => void;
  setStateForceCustomChoice: (value: boolean) => void;
}) {
  return (
    <DiscordCustomPresetEditorModal
      preset={preset}
      presetIndex={presetIndex}
      assets={assets}
      detailsForceCustomChoice={detailsForceCustomChoice}
      stateForceCustomChoice={stateForceCustomChoice}
      presetAdvancedAddonsConfigured={presetAdvancedAddonsConfigured}
      panelHeadClass={panelHeadClass}
      fieldClass={fieldClass}
      fieldSpanClass={fieldSpanClass}
      inputClass={inputClass}
      selectClass={selectClass}
      buttonClass={buttonClass}
      primaryButtonClass={primaryButtonClass}
      dangerButtonClass={dangerButtonClass}
      onClose={onClose}
      onUsePreset={() => {
        applyPreset(preset);
        onClose();
        onCloseCustomPresets();
      }}
      onNameChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, name: value }))
      }
      onActivityTypeChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, activityType: value }))
      }
      onStatusDisplayChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, statusDisplay: value }))
      }
      onAppNameModeChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, appNameMode: value }))
      }
      onCustomAppNameChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customAppName: value }))
      }
      onArtworkSourceChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customArtworkSource: value }))
      }
      onArtworkTextModeChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customArtworkTextMode: value }))
      }
      onArtworkTextChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customArtworkText: value }))
      }
      onArtworkAssetIdChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customArtworkAssetId: value }))
      }
      onAppIconSourceChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customAppIconSource: value }))
      }
      onAppIconTextModeChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customAppIconTextMode: value }))
      }
      onAppIconTextChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customAppIconText: value }))
      }
      onAppIconAssetIdChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, customAppIconAssetId: value }))
      }
      setDetailsForceCustomChoice={setDetailsForceCustomChoice}
      setStateForceCustomChoice={setStateForceCustomChoice}
      onDetailsFormatChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, detailsFormat: value }))
      }
      onStateFormatChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, stateFormat: value }))
      }
      onPatchButtonAt={(buttonIndex, updater) =>
        patchPresetAt(presetIndex, (item) => ({
          ...item,
          buttons: item.buttons.map((button, index) => (index === buttonIndex ? updater(button) : button)),
        }))
      }
      onRemoveButtonAt={(buttonIndex) =>
        patchPresetAt(presetIndex, (item) => ({
          ...item,
          buttons: item.buttons.filter((_, index) => index !== buttonIndex),
        }))
      }
      onAddButton={() =>
        patchPresetAt(presetIndex, (item) => ({
          ...item,
          buttons: [...item.buttons, createDiscordButton()],
        }))
      }
      onPartyIdChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, partyId: value }))
      }
      onPartySizeCurrentChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({
          ...item,
          partySizeCurrent: normalizePositiveNumberInput(value),
        }))
      }
      onPartySizeMaxChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({
          ...item,
          partySizeMax: normalizePositiveNumberInput(value),
        }))
      }
      onJoinSecretChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, joinSecret: value }))
      }
      onSpectateSecretChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, spectateSecret: value }))
      }
      onMatchSecretChange={(value) =>
        patchPresetAt(presetIndex, (item) => ({ ...item, matchSecret: value }))
      }
    />
  );
}
