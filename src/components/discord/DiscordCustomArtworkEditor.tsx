import { useMemo, useState } from "react";

import type {
  DiscordAssetTextMode,
  DiscordCustomAppIconSource,
  DiscordCustomArtworkSource,
  DiscordCustomAsset,
} from "../../types";
import { DiscordCustomAssetPickerDialog } from "./DiscordCustomAssetPickerDialog";
import {
  DISCORD_ASSET_TEXT_MODE_OPTIONS,
  DISCORD_CUSTOM_APP_ICON_SOURCE_OPTIONS,
  DISCORD_CUSTOM_ARTWORK_SOURCE_OPTIONS,
} from "./discordOptions";

function sourceHelper<T extends { value: string; helper: string }>(options: readonly T[], value: string) {
  return options.find((option) => option.value === value)?.helper ?? "";
}

export function DiscordCustomArtworkEditor({
  title = "Custom artwork",
  description = "Control the large and small asset slots that end up as artworkUrl, artworkHoverText, appIconUrl, and appIconText in Custom mode.",
  assets,
  artworkSource,
  artworkTextMode,
  artworkText,
  artworkAssetId,
  appIconSource,
  appIconTextMode,
  appIconText,
  appIconAssetId,
  panelHeadClass,
  fieldClass,
  fieldSpanClass,
  inputClass,
  selectClass,
  buttonClass,
  primaryButtonClass,
  onArtworkSourceChange,
  onArtworkTextModeChange,
  onArtworkTextChange,
  onArtworkAssetIdChange,
  onAppIconSourceChange,
  onAppIconTextModeChange,
  onAppIconTextChange,
  onAppIconAssetIdChange,
}: {
  title?: string;
  description?: string;
  assets: DiscordCustomAsset[];
  artworkSource: DiscordCustomArtworkSource;
  artworkTextMode: DiscordAssetTextMode;
  artworkText: string;
  artworkAssetId: string;
  appIconSource: DiscordCustomAppIconSource;
  appIconTextMode: DiscordAssetTextMode;
  appIconText: string;
  appIconAssetId: string;
  panelHeadClass: string;
  fieldClass: string;
  fieldSpanClass: string;
  inputClass: string;
  selectClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  onArtworkSourceChange: (value: DiscordCustomArtworkSource) => void;
  onArtworkTextModeChange: (value: DiscordAssetTextMode) => void;
  onArtworkTextChange: (value: string) => void;
  onArtworkAssetIdChange: (value: string) => void;
  onAppIconSourceChange: (value: DiscordCustomAppIconSource) => void;
  onAppIconTextModeChange: (value: DiscordAssetTextMode) => void;
  onAppIconTextChange: (value: string) => void;
  onAppIconAssetIdChange: (value: string) => void;
}) {
  const [activePickerTarget, setActivePickerTarget] = useState<"artwork" | "appIcon" | null>(null);
  const artworkVisible = artworkSource !== "none";
  const appIconVisible = appIconSource !== "none";
  const selectedArtworkAsset = useMemo(
    () => assets.find((asset) => asset.id === artworkAssetId) ?? null,
    [assets, artworkAssetId],
  );
  const selectedAppIconAsset = useMemo(
    () => assets.find((asset) => asset.id === appIconAssetId) ?? null,
    [assets, appIconAssetId],
  );

  return (
    <div className="rounded-box border border-base-300 bg-base-100 p-4 space-y-4">
      <div className={panelHeadClass}>
        <div>
          <strong className="block font-semibold">{title}</strong>
          <p className="mt-1 text-sm text-base-content/70">{description}</p>
        </div>
        <span className="badge badge-soft shrink-0 self-start whitespace-nowrap">Asset slots</span>
      </div>

      <div className="field-grid compact-fields">
        <label className={fieldClass}>
          <span>Large image source</span>
          <select
            className={selectClass}
            value={artworkSource}
            onChange={(event) => onArtworkSourceChange(event.currentTarget.value as DiscordCustomArtworkSource)}
          >
            {DISCORD_CUSTOM_ARTWORK_SOURCE_OPTIONS.map((option) => (
              <option key={`custom-artwork-source-${option.value}`} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <p className="text-xs text-base-content/60">
            {sourceHelper(DISCORD_CUSTOM_ARTWORK_SOURCE_OPTIONS, artworkSource)}
          </p>
        </label>
        <label className={fieldClass}>
          <span>Small image source</span>
          <select
            className={selectClass}
            value={appIconSource}
            onChange={(event) => onAppIconSourceChange(event.currentTarget.value as DiscordCustomAppIconSource)}
          >
            {DISCORD_CUSTOM_APP_ICON_SOURCE_OPTIONS.map((option) => (
              <option key={`custom-app-icon-source-${option.value}`} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <p className="text-xs text-base-content/60">
            {sourceHelper(DISCORD_CUSTOM_APP_ICON_SOURCE_OPTIONS, appIconSource)}
          </p>
        </label>

        {artworkSource === "library" ? (
          <div className={`${fieldSpanClass} space-y-2`}>
            <span>Large gallery image</span>
            <div className="rounded-box border border-base-300 bg-base-200/45 p-3 space-y-3">
              <div>
                <strong className="block font-semibold">
                  {selectedArtworkAsset?.name || "No image selected"}
                </strong>
                <p className="mt-1 text-sm text-base-content/70">
                  {selectedArtworkAsset
                    ? `${selectedArtworkAsset.fileName} · ${selectedArtworkAsset.contentType}`
                    : assets.length > 0
                      ? "Choose a gallery image from the preview picker."
                      : "No gallery images yet. Upload them from the Gallery page in the sidebar."}
                </p>
              </div>
              <div className="card-actions gap-2">
                <button className={buttonClass} type="button" onClick={() => setActivePickerTarget("artwork")}>
                  Choose image
                </button>
                {artworkAssetId ? (
                  <button className={buttonClass} type="button" onClick={() => onArtworkAssetIdChange("")}>
                    Clear
                  </button>
                ) : null}
              </div>
            </div>
          </div>
        ) : null}

        {appIconSource === "library" ? (
          <div className={`${fieldSpanClass} space-y-2`}>
            <span>Small gallery image</span>
            <div className="rounded-box border border-base-300 bg-base-200/45 p-3 space-y-3">
              <div>
                <strong className="block font-semibold">
                  {selectedAppIconAsset?.name || "No image selected"}
                </strong>
                <p className="mt-1 text-sm text-base-content/70">
                  {selectedAppIconAsset
                    ? `${selectedAppIconAsset.fileName} · ${selectedAppIconAsset.contentType}`
                    : assets.length > 0
                      ? "Choose a gallery image from the preview picker."
                      : "No gallery images yet. Upload them from the Gallery page in the sidebar."}
                </p>
              </div>
              <div className="card-actions gap-2">
                <button className={buttonClass} type="button" onClick={() => setActivePickerTarget("appIcon")}>
                  Choose image
                </button>
                {appIconAssetId ? (
                  <button className={buttonClass} type="button" onClick={() => onAppIconAssetIdChange("")}>
                    Clear
                  </button>
                ) : null}
              </div>
            </div>
          </div>
        ) : null}

        {artworkVisible ? (
          <label className={fieldClass}>
            <span>Large image hover text</span>
            <select
              className={selectClass}
              value={artworkTextMode}
              onChange={(event) => onArtworkTextModeChange(event.currentTarget.value as DiscordAssetTextMode)}
            >
              {DISCORD_ASSET_TEXT_MODE_OPTIONS.map((option) => (
                <option key={`custom-artwork-text-mode-${option.value}`} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ) : null}

        {appIconVisible ? (
          <label className={fieldClass}>
            <span>Small image hover text</span>
            <select
              className={selectClass}
              value={appIconTextMode}
              onChange={(event) => onAppIconTextModeChange(event.currentTarget.value as DiscordAssetTextMode)}
            >
              {DISCORD_ASSET_TEXT_MODE_OPTIONS.map((option) => (
                <option key={`custom-app-icon-text-mode-${option.value}`} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ) : null}

        {artworkVisible && artworkTextMode === "custom" ? (
          <label className={fieldSpanClass}>
            <span>Large image custom hover text</span>
            <input
              className={inputClass}
              value={artworkText}
              onChange={(event) => onArtworkTextChange(event.currentTarget.value)}
              placeholder="Shown when hovering the large image"
            />
          </label>
        ) : null}

        {appIconVisible && appIconTextMode === "custom" ? (
          <label className={fieldSpanClass}>
            <span>Small image custom hover text</span>
            <input
              className={inputClass}
              value={appIconText}
              onChange={(event) => onAppIconTextChange(event.currentTarget.value)}
              placeholder="Shown when hovering the small image"
            />
          </label>
        ) : null}
      </div>

      {activePickerTarget === "artwork" ? (
        <DiscordCustomAssetPickerDialog
          title="Choose large image"
          assets={assets}
          selectedAssetId={artworkAssetId}
          panelHeadClass={panelHeadClass}
          buttonClass={buttonClass}
          primaryButtonClass={primaryButtonClass}
          onClose={() => setActivePickerTarget(null)}
          onSelect={(assetId) => {
            onArtworkAssetIdChange(assetId);
            setActivePickerTarget(null);
          }}
        />
      ) : null}

      {activePickerTarget === "appIcon" ? (
        <DiscordCustomAssetPickerDialog
          title="Choose small image"
          assets={assets}
          selectedAssetId={appIconAssetId}
          panelHeadClass={panelHeadClass}
          buttonClass={buttonClass}
          primaryButtonClass={primaryButtonClass}
          onClose={() => setActivePickerTarget(null)}
          onSelect={(assetId) => {
            onAppIconAssetIdChange(assetId);
            setActivePickerTarget(null);
          }}
        />
      ) : null}
    </div>
  );
}
