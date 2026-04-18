import { useMemo, useState } from "react";

interface SuggestionInputProps {
  value: string;
  onChange: (value: string) => void;
  suggestions: string[];
  exclude?: string[];
  placeholder: string;
  lowercase?: boolean;
  size?: "sm" | "md";
}

interface ListEditorProps {
  title: string;
  description: string;
  placeholder: string;
  value: string[];
  inputValue: string;
  onInputValueChange: (value: string) => void;
  onAdd: () => void;
  onRemove: (index: number) => void;
  suggestions: string[];
  lowercase?: boolean;
}

export function SuggestionInput({ value, onChange, suggestions, exclude = [], placeholder, lowercase = false, size = "md" }: SuggestionInputProps) {
  const [open, setOpen] = useState(false);
  const excludeKeys = useMemo(
    () => new Set(exclude.map((item) => item.trim().toLowerCase()).filter(Boolean)),
    [exclude],
  );
  const normalizedSuggestions = useMemo(
    () =>
      Array.from(new Set(suggestions.map((item) => (lowercase ? item.toLowerCase() : item)).filter(Boolean))).filter(
        (item) => !excludeKeys.has(item.trim().toLowerCase()),
      ),
    [excludeKeys, lowercase, suggestions],
  );
  const visibleSuggestions = useMemo(() => {
    const query = value.trim().toLowerCase();
    return normalizedSuggestions
      .filter((item) => !query || item.toLowerCase().includes(query))
      .slice(0, 8);
  }, [normalizedSuggestions, value]);
  const showSuggestions = open && visibleSuggestions.length > 0;

  return (
    <div className={`dropdown dropdown-bottom suggestion-dropdown w-full ${showSuggestions ? "dropdown-open" : ""}`}>
      <input
        className={`input input-bordered w-full ${size === "sm" ? "input-sm" : ""}`}
        value={value}
        onBlur={() => window.setTimeout(() => setOpen(false), 120)}
        onChange={(event) => {
          onChange(event.currentTarget.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        placeholder={placeholder}
        role="combobox"
        aria-expanded={showSuggestions}
        spellCheck={false}
      />
      {showSuggestions ? (
        <ul className="dropdown-content menu suggestion-menu z-20 mt-2 rounded-box border border-base-300 bg-base-100 p-1 shadow-lg">
          {visibleSuggestions.map((item) => (
            <li key={item}>
              <button
                type="button"
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => {
                  onChange(item);
                  setOpen(false);
                }}
              >
                {item}
              </button>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

export function ListEditor({
  title,
  description,
  placeholder,
  value,
  inputValue,
  onInputValueChange,
  onAdd,
  onRemove,
  suggestions,
  lowercase = false,
}: ListEditorProps) {
  return (
    <div className="list-editor rounded-box border border-base-300 bg-base-200/45 p-4">
      <div className="list-editor-summary">
        <div className="list-editor-copy">
          <strong className="block font-semibold">{title}</strong>
          <p>{description}</p>
        </div>
        <span className="badge badge-soft">{value.length} item{value.length === 1 ? "" : "s"}</span>
      </div>

      <div className="list-editor-input">
        <SuggestionInput
          value={inputValue}
          onChange={onInputValueChange}
          suggestions={suggestions}
          exclude={value}
          placeholder={placeholder}
          lowercase={lowercase}
          size="sm"
        />
        <button className="btn btn-primary btn-sm" type="button" onClick={onAdd}>
          Add
        </button>
      </div>

      {value.length === 0 ? (
        <div className="empty-state compact-empty">No entries yet.</div>
      ) : (
        <div className="tag-list">
          {value.map((item, index) => (
            <button
              key={`${item}-${index}`}
              className="btn btn-outline btn-xs h-auto min-h-8 max-w-full justify-between gap-2 rounded-md normal-case"
              type="button"
              onClick={() => onRemove(index)}
              title="Remove item"
            >
              <span className="truncate">{item}</span>
              <strong>Remove</strong>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
