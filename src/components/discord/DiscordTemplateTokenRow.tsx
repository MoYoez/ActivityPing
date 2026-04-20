import { DISCORD_TEMPLATE_TOKENS } from "./discordOptions";

export function DiscordTemplateTokenRow({
  onInsert,
}: {
  onInsert: (token: string) => void;
}) {
  return (
    <div className="discord-template-token-row">
      <span>Quick insert</span>
      {DISCORD_TEMPLATE_TOKENS.map((token) => (
        <button
          key={token}
          className="btn btn-ghost btn-xs no-animation"
          type="button"
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => onInsert(token)}
        >
          {token}
        </button>
      ))}
    </div>
  );
}
