# Rules and Templates

## App Message Rules

Each rule group matches a process name first, then optionally refines the result with title subrules.

- `processMatch`: partial match on the executable or process name
- `titleRules`: optional plain or regex checks against the window title
- `defaultText`: fallback text when the process matches but no title subrule does

Rule text supports:

- `{process}` for the captured process name
- `{title}` for the captured window title

## Custom Discord Tokens

Custom mode supports the following tokens:

- `{activity}`: primary resolved activity text
- `{context}`: secondary line, usually app name or artist
- `{app}`: captured app or process name
- `{title}`: current window title
- `{rule}`: matched rule text
- `{media}`: combined media summary
- `{song}`: media title
- `{artist}`: media artist
- `{album}`: media album
- `{source}`: media source app id

## Rule Design Notes

- Use process matching to keep rules stable across changing window titles.
- Use title subrules when the same app should map to different activities.
- Use `defaultText` as the fallback for anything inside the same process group.
- Use Custom mode when you want full control over the Discord `details` and `state` lines.

