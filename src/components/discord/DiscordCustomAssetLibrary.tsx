import { useEffect, useMemo, useRef, useState } from "react";

import { clampPage, pageCount } from "../../app/appFormatting";
import type { DiscordCustomAsset } from "../../types";
import { useDiscordCustomAssetPreviews } from "./useDiscordCustomAssetPreviews";

const GALLERY_PAGE_SIZE = 12;

function formatBytes(value: number) {
  if (value >= 1024 * 1024) {
    return `${(value / (1024 * 1024)).toFixed(1)} MB`;
  }
  if (value >= 1024) {
    return `${Math.round(value / 1024)} KB`;
  }
  return `${value} B`;
}

function formatCreatedAt(value: string) {
  const timestamp = Date.parse(value);
  if (Number.isNaN(timestamp)) {
    return "Unknown date";
  }
  return new Date(timestamp).toLocaleString();
}

export function DiscordCustomAssetLibrary({
  assets,
  panelHeadClass,
  buttonClass,
  dangerButtonClass,
  onImportFiles,
  onDeleteAsset,
}: {
  assets: DiscordCustomAsset[];
  panelHeadClass: string;
  buttonClass: string;
  dangerButtonClass: string;
  onImportFiles: (files: File[]) => void;
  onDeleteAsset: (assetId: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [galleryPage, setGalleryPage] = useState(0);
  const safeGalleryPage = clampPage(galleryPage, assets.length, GALLERY_PAGE_SIZE);
  const galleryTotalPages = pageCount(assets.length, GALLERY_PAGE_SIZE);
  const galleryPageStart = safeGalleryPage * GALLERY_PAGE_SIZE;
  const galleryPageEnd = Math.min(galleryPageStart + GALLERY_PAGE_SIZE, assets.length);
  const visibleAssets = useMemo(
    () => assets.slice(galleryPageStart, galleryPageEnd),
    [assets, galleryPageEnd, galleryPageStart],
  );
  const { previewMap, loading } = useDiscordCustomAssetPreviews(visibleAssets);

  useEffect(() => {
    setGalleryPage((current) => clampPage(current, assets.length, GALLERY_PAGE_SIZE));
  }, [assets.length]);

  return (
    <div className="rounded-box border border-base-300 bg-base-100 p-4 space-y-4">
      <div className={panelHeadClass}>
        <div>
          <strong className="block font-semibold">Local artwork gallery</strong>
          <p className="mt-1 text-sm text-base-content/70">
            Upload PNG or JPEG files here, browse them visually, and remove the ones you no longer need.
          </p>
        </div>
        <div className="card-actions gap-2">
          <span className="badge badge-soft">{assets.length} image{assets.length === 1 ? "" : "s"}</span>
          <button className={buttonClass} type="button" onClick={() => inputRef.current?.click()}>
            Upload images
          </button>
        </div>
      </div>

      <input
        ref={inputRef}
        className="hidden"
        type="file"
        accept="image/png,image/jpeg,image/jpg"
        multiple
        onChange={(event) => {
          const files = Array.from(event.currentTarget.files ?? []);
          if (files.length > 0) {
            onImportFiles(files);
          }
          event.currentTarget.value = "";
        }}
      />

      {assets.length === 0 ? (
        <div className="empty-state">
          Your gallery is empty. Upload PNG or JPEG files here and they will show up in the Custom artwork pickers.
        </div>
      ) : (
        <>
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
            {visibleAssets.map((asset) => (
              <article key={asset.id} className="min-w-0 rounded-box border border-base-300 bg-base-100 p-3 space-y-3">
                <div className="aspect-[4/3] overflow-hidden rounded-box border border-base-300 bg-base-200/60">
                  {previewMap[asset.id] ? (
                    <img className="h-full w-full object-contain" src={previewMap[asset.id]} alt={asset.name} />
                  ) : (
                    <div className="flex h-full items-center justify-center px-4 text-center text-sm text-base-content/60">
                      {loading ? "Loading preview..." : "Preview unavailable"}
                    </div>
                  )}
                </div>
                <div className="space-y-2 min-w-0">
                  <strong className="block min-w-0 break-all font-semibold">{asset.name}</strong>
                  <p className="break-all text-sm text-base-content/70">{asset.fileName}</p>
                  <p className="text-xs text-base-content/60">
                    {asset.contentType} · {formatBytes(asset.byteSize)} · {formatCreatedAt(asset.createdAt)}
                  </p>
                </div>
                <div className="card-actions">
                  <button className={dangerButtonClass} type="button" onClick={() => onDeleteAsset(asset.id)}>
                    Delete
                  </button>
                </div>
              </article>
            ))}
          </div>

          {galleryTotalPages > 1 ? (
            <div className="pagination-row">
              <span className="pagination-copy">
                {galleryPageStart + 1}-{galleryPageEnd} of {assets.length}
              </span>
              <div className="join">
                <button
                  className="btn btn-outline btn-xs join-item"
                  type="button"
                  disabled={safeGalleryPage === 0}
                  onClick={() => setGalleryPage((current) => clampPage(current - 1, assets.length, GALLERY_PAGE_SIZE))}
                >
                  Prev
                </button>
                <span className="btn btn-ghost btn-xs join-item no-animation">
                  Page {safeGalleryPage + 1} / {galleryTotalPages}
                </span>
                <button
                  className="btn btn-outline btn-xs join-item"
                  type="button"
                  disabled={safeGalleryPage >= galleryTotalPages - 1}
                  onClick={() => setGalleryPage((current) => clampPage(current + 1, assets.length, GALLERY_PAGE_SIZE))}
                >
                  Next
                </button>
              </div>
            </div>
          ) : null}
        </>
      )}
    </div>
  );
}
