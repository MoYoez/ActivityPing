import type { SetStateAction } from "react";

import { patchDiscordModeSettings } from "../components/discord/discordOptions";
import type {
  AppMessageRuleGroup,
  AppMessageTitleRule,
  ClientConfig,
  DiscordAppNameMode,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
  DiscordStatusDisplay,
} from "../types";

export function createConfigEditorActions(setConfig: (value: SetStateAction<ClientConfig>) => void) {
  function update<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
    setConfig((current) => ({ ...current, [key]: value }));
  }

  function updateDiscordModeSettings(patch: {
    statusDisplay?: DiscordStatusDisplay;
    appNameMode?: DiscordAppNameMode;
    customAppName?: string;
  }) {
    setConfig((current) => patchDiscordModeSettings(current, current.discordReportMode, patch));
  }

  function updateRuntimeAutostart(enabled: boolean) {
    setConfig((current) => ({
      ...current,
      runtimeAutostartEnabled: enabled,
    }));
  }

  function patchRuleAt(index: number, updater: (rule: AppMessageRuleGroup) => AppMessageRuleGroup) {
    setConfig((current) => {
      const next = [...current.appMessageRules];
      if (!next[index]) {
        return current;
      }
      next[index] = updater(next[index]);
      return { ...current, appMessageRules: next };
    });
  }

  function patchTitleRuleAt(ruleIndex: number, titleRuleIndex: number, updater: (rule: AppMessageTitleRule) => AppMessageTitleRule) {
    patchRuleAt(ruleIndex, (rule) => {
      const next = [...rule.titleRules];
      if (!next[titleRuleIndex]) {
        return rule;
      }
      next[titleRuleIndex] = updater(next[titleRuleIndex]);
      return { ...rule, titleRules: next };
    });
  }

  function patchDiscordCustomPresetAt(index: number, updater: (preset: DiscordCustomPreset) => DiscordCustomPreset) {
    setConfig((current) => {
      const next = [...current.discordCustomPresets];
      if (!next[index]) {
        return current;
      }
      next[index] = updater(next[index]);
      return { ...current, discordCustomPresets: next };
    });
  }

  function patchDiscordButtonAt(index: number, updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig) {
    setConfig((current) => {
      const next = [...current.discordCustomButtons];
      if (!next[index]) {
        return current;
      }
      next[index] = updater(next[index]);
      return { ...current, discordCustomButtons: next };
    });
  }

  function patchRuleDiscordButtonAt(
    ruleIndex: number,
    buttonIndex: number,
    updater: (button: DiscordRichPresenceButtonConfig) => DiscordRichPresenceButtonConfig,
  ) {
    patchRuleAt(ruleIndex, (rule) => {
      const next = [...rule.buttons];
      if (!next[buttonIndex]) {
        return rule;
      }
      next[buttonIndex] = updater(next[buttonIndex]);
      return { ...rule, buttons: next };
    });
  }

  return {
    update,
    updateDiscordModeSettings,
    updateRuntimeAutostart,
    patchRuleAt,
    patchTitleRuleAt,
    patchDiscordCustomPresetAt,
    patchDiscordButtonAt,
    patchRuleDiscordButtonAt,
  };
}
