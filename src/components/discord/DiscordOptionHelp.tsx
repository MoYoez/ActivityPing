import {
  DISCORD_APP_NAME_OPTIONS,
  DISCORD_CUSTOM_LINE_OPTIONS,
  DISCORD_STATUS_DISPLAY_OPTIONS,
} from "./discordOptions";

export function DiscordOptionHelp({
  idPrefix,
  includeSmartModeNote = true,
}: {
  idPrefix: string;
  includeSmartModeNote?: boolean;
}) {
  const helpEntries = [
    ...DISCORD_CUSTOM_LINE_OPTIONS.map((option) => ({
      key: `${idPrefix}-line-${option.value || "hidden"}`,
      label: option.label,
      helper: option.helper,
    })),
    ...DISCORD_STATUS_DISPLAY_OPTIONS.map((option) => ({
      key: `${idPrefix}-status-${option.value}`,
      label: option.label,
      helper: option.helper,
    })),
    ...DISCORD_APP_NAME_OPTIONS.map((option) => ({
      key: `${idPrefix}-app-name-${option.value}`,
      label: option.label,
      helper: option.helper,
    })),
    ...(includeSmartModeNote
      ? [
          {
            key: `${idPrefix}-smart-mode`,
            label: "Smart mode",
            helper:
              "line 1 follows the matched title, line 2 can show the foreground app, and line 3 carries music when playback is active.",
          },
        ]
      : []),
  ];

  return (
    <details className="dropdown dropdown-end discord-option-help">
      <summary className="discord-option-help-trigger" aria-label="Show Discord option help">
        ?
      </summary>
      <div className="discord-option-help-panel dropdown-content z-[20] w-[min(38rem,calc(100vw-2rem))] max-w-[calc(100vw-2rem)] rounded-box border border-base-300 bg-base-100 p-4 text-sm text-base-content/70 shadow-xl">
        <div className="discord-option-help-grid">
          {helpEntries.map((entry) => (
            <p key={entry.key}>
              <strong>{entry.label}:</strong> {entry.helper}
            </p>
          ))}
        </div>
      </div>
    </details>
  );
}
