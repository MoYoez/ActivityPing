import type {
  AppFilterMode,
  AppMessageRuleGroup,
  AppMessageTitleRule,
  AppTitleRuleMode,
  ClientConfig,
  DiscordCustomPreset,
  DiscordRichPresenceButtonConfig,
} from "../../types";

import {
  normalizeDiscordActivityType,
  normalizeDiscordAppNameMode,
  normalizeDiscordButton,
  normalizeDiscordCustomPreset,
  normalizeDiscordStatusDisplay,
  normalizePartySize,
  normalizeStringList,
  normalizeTitleRule,
} from "./helpers";
import { normalizeClientConfig } from "./normalizers";
export type ParsedRulesPayload =
  | {
      ok: true;
      data: {
        appMessageRules: AppMessageRuleGroup[];
        appMessageRulesShowProcessName: boolean;
    discordCustomPresets: DiscordCustomPreset[];
    discordUseCustomAddonsOverride: boolean;
        appFilterMode: AppFilterMode;
        appBlacklist: string[];
        appWhitelist: string[];
        appNameOnlyList: string[];
        mediaPlaySourceBlocklist: string[];
      };
    }
  | { ok: false; error: string };

export function exportRulesJson(config: ClientConfig) {
  const normalized = normalizeClientConfig(config);
  return JSON.stringify(
    {
      version: 5,
      exportedAt: new Date().toISOString(),
      rules: {
        appMessageRules: normalized.appMessageRules,
        appMessageRulesShowProcessName: normalized.appMessageRulesShowProcessName,
        discordUseCustomAddonsOverride: normalized.discordUseCustomAddonsOverride,
        discordCustomPresets: normalized.discordCustomPresets,
        appFilterMode: normalized.appFilterMode,
        appBlacklist: normalized.appBlacklist,
        appWhitelist: normalized.appWhitelist,
        appNameOnlyList: normalized.appNameOnlyList,
        mediaPlaySourceBlocklist: normalized.mediaPlaySourceBlocklist,
      },
    },
    null,
    2,
  );
}

function normalizeImportedRuleGroup(raw: unknown): AppMessageRuleGroup | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return null;
  }

  const group = raw as {
    processMatch?: unknown;
    defaultText?: unknown;
    titleRules?: unknown;
    buttons?: unknown;
    partyId?: unknown;
    partySizeCurrent?: unknown;
    partySizeMax?: unknown;
    joinSecret?: unknown;
    spectateSecret?: unknown;
    matchSecret?: unknown;
    match?: unknown;
    text?: unknown;
  };
  const processMatch = String(group.processMatch ?? group.match ?? "").trim();
  const defaultText = String(group.defaultText ?? group.text ?? "").trim();
  const titleRules = (Array.isArray(group.titleRules) ? group.titleRules : [])
    .map((item) => {
      if (!item || typeof item !== "object" || Array.isArray(item)) {
        return null;
      }
      const titleRule = item as { mode?: unknown; pattern?: unknown; text?: unknown };
      return normalizeTitleRule({
        mode: String(titleRule.mode ?? "plain") as AppTitleRuleMode,
        pattern: String(titleRule.pattern ?? ""),
        text: String(titleRule.text ?? ""),
        buttons: Array.isArray((titleRule as { buttons?: unknown }).buttons)
          ? ((titleRule as { buttons?: unknown }).buttons as DiscordRichPresenceButtonConfig[])
          : [],
      });
    })
    .filter((item): item is AppMessageTitleRule => item !== null);
  const buttons = (Array.isArray(group.buttons) ? group.buttons : [])
    .map((button) => normalizeDiscordButton(button as Partial<DiscordRichPresenceButtonConfig>))
    .filter((button): button is DiscordRichPresenceButtonConfig => button !== null)
    .slice(0, 2);
  const partyId = String(group.partyId ?? "").trim();
  const partySizeCurrent = normalizePartySize(group.partySizeCurrent);
  const partySizeMax = normalizePartySize(group.partySizeMax);
  const joinSecret = String(group.joinSecret ?? "").trim();
  const spectateSecret = String(group.spectateSecret ?? "").trim();
  const matchSecret = String(group.matchSecret ?? "").trim();

  if (
    !processMatch ||
    (!defaultText &&
      titleRules.length === 0 &&
      buttons.length === 0 &&
      !partyId &&
      partySizeCurrent === null &&
      partySizeMax === null &&
      !joinSecret &&
      !spectateSecret &&
      !matchSecret)
  ) {
    return null;
  }

  return {
    processMatch,
    defaultText,
    titleRules,
    buttons,
    partyId,
    partySizeCurrent,
    partySizeMax,
    joinSecret,
    spectateSecret,
    matchSecret,
  };
}

function normalizeImportedDiscordCustomPreset(raw: unknown): DiscordCustomPreset | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return null;
  }

  const rule = raw as {
    name?: unknown;
    activityType?: unknown;
    statusDisplay?: unknown;
    appNameMode?: unknown;
    customAppName?: unknown;
    detailsFormat?: unknown;
    stateFormat?: unknown;
    buttons?: unknown;
    partyId?: unknown;
    partySizeCurrent?: unknown;
    partySizeMax?: unknown;
    joinSecret?: unknown;
    spectateSecret?: unknown;
    matchSecret?: unknown;
  };

  return normalizeDiscordCustomPreset({
    name: String(rule.name ?? ""),
    activityType: normalizeDiscordActivityType(rule.activityType),
    statusDisplay: normalizeDiscordStatusDisplay(rule.statusDisplay),
    appNameMode: normalizeDiscordAppNameMode(rule.appNameMode),
    customAppName: String(rule.customAppName ?? ""),
    detailsFormat: String(rule.detailsFormat ?? ""),
    stateFormat: String(rule.stateFormat ?? ""),
    buttons: Array.isArray(rule.buttons) ? (rule.buttons as DiscordRichPresenceButtonConfig[]) : [],
    partyId: String(rule.partyId ?? ""),
    partySizeCurrent: rule.partySizeCurrent as number | null | undefined,
    partySizeMax: rule.partySizeMax as number | null | undefined,
    joinSecret: String(rule.joinSecret ?? ""),
    spectateSecret: String(rule.spectateSecret ?? ""),
    matchSecret: String(rule.matchSecret ?? ""),
  });
}

export function parseRulesJson(raw: string): ParsedRulesPayload {
  let json: unknown;
  try {
    json = JSON.parse(raw);
  } catch {
    return { ok: false, error: "JSON parse failed." };
  }

  if (!json || typeof json !== "object" || Array.isArray(json)) {
    return { ok: false, error: "The JSON top level must be an object." };
  }

  const root = json as Record<string, unknown>;
  if (
    typeof root.version === "number" &&
    root.version !== 1 &&
    root.version !== 2 &&
    root.version !== 3 &&
    root.version !== 4 &&
    root.version !== 5
  ) {
    return { ok: false, error: "Unsupported rules JSON version." };
  }

  const rules = root.rules;
  if (!rules || typeof rules !== "object" || Array.isArray(rules)) {
    return { ok: false, error: "Missing rules object." };
  }

  const payload = rules as Record<string, unknown>;
  return {
    ok: true,
    data: {
      appMessageRules: (Array.isArray(payload.appMessageRules) ? payload.appMessageRules : [])
        .map((rule) => normalizeImportedRuleGroup(rule))
        .filter((rule): rule is AppMessageRuleGroup => rule !== null),
      appMessageRulesShowProcessName:
        typeof payload.appMessageRulesShowProcessName === "boolean"
          ? payload.appMessageRulesShowProcessName
          : false,
      discordUseCustomAddonsOverride:
        typeof payload.discordUseCustomAddonsOverride === "boolean"
          ? payload.discordUseCustomAddonsOverride
          : false,
      discordCustomPresets: (
        Array.isArray(payload.discordCustomPresets)
          ? payload.discordCustomPresets
          : Array.isArray(payload.discordCustomRules)
            ? payload.discordCustomRules
            : []
      )
        .map((rule) => normalizeImportedDiscordCustomPreset(rule))
        .filter((rule): rule is DiscordCustomPreset => rule !== null),
      appFilterMode: String(payload.appFilterMode ?? "").trim().toLowerCase() === "whitelist" ? "whitelist" : "blacklist",
      appBlacklist: normalizeStringList(
        Array.isArray(payload.appBlacklist) ? payload.appBlacklist.map((item) => String(item ?? "")) : [],
        false,
      ),
      appWhitelist: normalizeStringList(
        Array.isArray(payload.appWhitelist) ? payload.appWhitelist.map((item) => String(item ?? "")) : [],
        false,
      ),
      appNameOnlyList: normalizeStringList(
        Array.isArray(payload.appNameOnlyList) ? payload.appNameOnlyList.map((item) => String(item ?? "")) : [],
        false,
      ),
      mediaPlaySourceBlocklist: normalizeStringList(
        Array.isArray(payload.mediaPlaySourceBlocklist)
          ? payload.mediaPlaySourceBlocklist.map((item) => String(item ?? ""))
          : [],
        true,
      ),
    },
  };
}
