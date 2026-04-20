import { DiscordTemplateTokenRow } from "./DiscordTemplateTokenRow";
import {
  appendDiscordTemplateToken,
  DISCORD_CUSTOM_LINE_CUSTOM_VALUE,
  DISCORD_CUSTOM_LINE_OPTIONS,
  discordLineCustomTextValue,
  nextDiscordLineValue,
  resolveDiscordLineChoice,
} from "./discordOptions";

export function DiscordLineTemplateEditor({
  label,
  value,
  forceCustomChoice,
  placeholder,
  optionKeyPrefix,
  fieldSpanClass,
  selectClass,
  inputClass,
  setForceCustomChoice,
  onValueChange,
}: {
  label: string;
  value: string;
  forceCustomChoice: boolean;
  placeholder: string;
  optionKeyPrefix: string;
  fieldSpanClass: string;
  selectClass: string;
  inputClass: string;
  setForceCustomChoice: (value: boolean) => void;
  onValueChange: (value: string) => void;
}) {
  const resolvedChoice = resolveDiscordLineChoice(value, forceCustomChoice);
  const customTextValue = discordLineCustomTextValue(value, forceCustomChoice);

  return (
    <>
      <label className={fieldSpanClass}>
        <span>{label}</span>
        <select
          className={selectClass}
          value={resolvedChoice}
          onChange={(e) => {
            const nextChoice = e.currentTarget.value;
            setForceCustomChoice(nextChoice === DISCORD_CUSTOM_LINE_CUSTOM_VALUE);
            onValueChange(nextDiscordLineValue(value, nextChoice, forceCustomChoice));
          }}
        >
          {DISCORD_CUSTOM_LINE_OPTIONS.map((option) => (
            <option key={`${optionKeyPrefix}-${option.value || "hidden"}`} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>
      {resolvedChoice === DISCORD_CUSTOM_LINE_CUSTOM_VALUE ? (
        <label className={fieldSpanClass}>
          <span>{label} custom text</span>
          <input
            className={inputClass}
            value={customTextValue}
            onChange={(e) => {
              setForceCustomChoice(true);
              onValueChange(e.currentTarget.value.trim() || DISCORD_CUSTOM_LINE_CUSTOM_VALUE);
            }}
            placeholder={placeholder}
          />
          <DiscordTemplateTokenRow
            onInsert={(token) => {
              setForceCustomChoice(true);
              onValueChange(appendDiscordTemplateToken(customTextValue, token));
            }}
          />
        </label>
      ) : null}
    </>
  );
}
