export function DiscordArtworkPublishingPanel({
  artworkPublishingMissing,
  uploadUrl,
  token,
  fieldSpanClass,
  inputClass,
  onUploadUrlChange,
  onTokenChange,
}: {
  artworkPublishingMissing: boolean;
  uploadUrl: string;
  token: string;
  fieldSpanClass: string;
  inputClass: string;
  onUploadUrlChange: (value: string) => void;
  onTokenChange: (value: string) => void;
}) {
  return (
    <div className="discord-custom-panel rounded-box border border-base-300 bg-base-200/45 p-4 space-y-4">
      <div className="list-editor-summary">
        <div className="list-editor-copy">
          <strong className="block font-semibold">Artwork publishing</strong>
          <p>App icons are uploaded as PNG with transparency preserved, while music artwork is uploaded as JPEG. ActivityPing normalizes both to a 256px target and keeps each uploaded image within a 30 KB budget before sending it to your uploader service.</p>
        </div>
        <span className="badge badge-soft">Uploader service</span>
      </div>
      {artworkPublishingMissing ? (
        <div className="alert alert-warning alert-soft text-sm">
          <span>Artwork publishing needs an uploader service URL before app artwork, music artwork, or Custom Gallery images can be saved.</span>
        </div>
      ) : null}
      <div className="field-grid">
        <label className={fieldSpanClass}>
          <span>Uploader service URL</span>
          <input
            className={inputClass}
            aria-invalid={artworkPublishingMissing}
            value={uploadUrl}
            onChange={(e) => onUploadUrlChange(e.currentTarget.value)}
            placeholder="https://your-uploader.example.com/upload"
          />
        </label>
        <label className={fieldSpanClass}>
          <span>Uploader service token</span>
          <input
            className={inputClass}
            type="password"
            value={token}
            onChange={(e) => onTokenChange(e.currentTarget.value)}
            placeholder="Optional bearer token"
          />
        </label>
      </div>
    </div>
  );
}
