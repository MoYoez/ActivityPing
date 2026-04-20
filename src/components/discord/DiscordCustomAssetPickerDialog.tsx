import { createPortal } from "react-dom";

import type { DiscordCustomAsset } from "../../types";
import { useDiscordCustomAssetPreviews } from "./useDiscordCustomAssetPreviews";

function formatBytes(value: number) {
  if (value >= 1024 * 1024) {
    return `${(value / (1024 * 1024)).toFixed(1)} MB`;
  }
  if (value >= 1024) {
    return `${Math.round(value / 1024)} KB`;
  }
  return `${value} B`;
}

export function DiscordCustomAssetPickerDialog({
  title,
  assets,
  selectedAssetId,
  panelHeadClass,
  buttonClass,
  primaryButtonClass,
  onClose,
  onSelect,
}: {
  title: string;
  assets: DiscordCustomAsset[];
  selectedAssetId: string;
  panelHeadClass: string;
  buttonClass: string;
  primaryButtonClass: string;
  onClose: () => void;
  onSelect: (assetId: string) => void;
}) {
  const { previewMap, loading } = useDiscordCustomAssetPreviews(assets);

  if (typeof document === "undefined") {
    return null;
  }

  return createPortal(
    <section className="modal modal-open z-[1100]">
      <div
        className="modal-box w-11/12 max-w-5xl p-0"
        role="dialog"
        aria-modal="true"
        aria-labelledby="custom-asset-picker-title"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="card-body">
          <div className={panelHeadClass}>
            <div>
              <p className="eyebrow">Gallery</p>
              <h3 id="custom-asset-picker-title" className="card-title">
                {title}
              </h3>
              <p>Pick a saved image from the local gallery.</p>
            </div>
            <div className="card-actions gap-2">
              <button className={buttonClass} type="button" onClick={onClose}>
                Close
              </button>
            </div>
          </div>

          {assets.length === 0 ? (
            <div className="empty-state">No gallery images yet.</div>
          ) : loading ? (
            <div className="empty-state">Loading gallery...</div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
              {assets.map((asset) => (
                <article key={asset.id} className="min-w-0 rounded-box border border-base-300 bg-base-100 p-3 space-y-3">
                  <div className="aspect-[4/3] overflow-hidden rounded-box border border-base-300 bg-base-200/60">
                    {previewMap[asset.id] ? (
                      <img
                        className="h-full w-full object-contain"
                        src={previewMap[asset.id]}
                        alt={asset.name}
                      />
                    ) : (
                      <div className="flex h-full items-center justify-center text-sm text-base-content/60">
                        Preview unavailable
                      </div>
                    )}
                  </div>
                  <div className="min-w-0">
                    <strong className="block overflow-hidden text-ellipsis break-all font-semibold">{asset.name}</strong>
                    <p className="mt-1 break-all text-sm text-base-content/70">
                      {asset.fileName} · {formatBytes(asset.byteSize)}
                    </p>
                  </div>
                  <div className="card-actions flex-wrap justify-between gap-2">
                    {selectedAssetId === asset.id ? <span className="badge badge-soft">Selected</span> : <span className="min-h-6" />}
                    <button className={primaryButtonClass} type="button" onClick={() => onSelect(asset.id)}>
                      Select image
                    </button>
                  </div>
                </article>
              ))}
            </div>
          )}
        </div>
      </div>
    </section>,
    document.body,
  );
}
