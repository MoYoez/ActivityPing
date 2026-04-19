# Configuration and Runtime

## Configuration Flow

1. Open the app and save a Discord Application ID.
2. Choose a reporting mode: Smart, Music, App, or Custom.
3. Add app rules, filters, and privacy lists as needed.
4. Optionally configure an artwork uploader service for app or album images.
5. Start runtime and inspect the live activity preview.
6. Open the Discord payload dialog if you need to debug the exact outbound data.

## Runtime Outputs

ActivityPing exposes processed activity in two places:

- the built-in runtime monitor
- Discord Rich Presence

## Artwork Uploads

If app artwork or music artwork is enabled, ActivityPing normalizes images before uploading them through your configured uploader service. App icons stay PNG so transparency is preserved. Music artwork is re-encoded as JPEG. Both are normalized around a 256px target before upload. The uploader must return a public image URL that Discord can fetch.

If you do not need image-rich Rich Presence, leave artwork uploads disabled.

### Request Format

ActivityPing sends an HTTP `POST` request to your configured uploader URL with:

- `Content-Type: application/json`
- `Authorization: Bearer <token>` when `discordArtworkWorkerToken` is configured
- a 10 second request timeout

Before sending, ActivityPing:

- decodes the original artwork
- keeps app icons as PNG while preserving transparency
- resizes music artwork to fit within `256x256`
- re-encodes music artwork as JPEG
- keeps the final uploaded payload within roughly `30 KB`
- wraps it as a data URL

Request JSON:

```json
{
  "base64": "data:image/png;base64,...",
  "imageBase64": "data:image/png;base64,...",
  "fileName": "etag.png",
  "expiresIn": 3600
}
```

Notes:

- `base64` and `imageBase64` currently contain the same value
- app icons use a PNG data URL and `.png` file name
- music artwork uses a JPEG data URL and `.jpg` file name
- `expiresIn` is the requested URL lifetime in seconds

### Accepted Response Format

ActivityPing treats the upload as successful when:

- the HTTP status is `2xx`
- and a non-empty public URL exists in either `url` or `data.url`

Accepted success examples:

```json
{
  "ok": true,
  "url": "https://example.com/files/etag.jpg"
}
```

```json
{
  "ok": true,
  "data": {
    "url": "https://example.com/files/etag.jpg"
  }
}
```

Accepted error examples:

```json
{
  "ok": false,
  "error": "invalid token"
}
```

```json
{
  "ok": false,
  "data": {
    "error": "upload failed"
  }
}
```

### Python Example

A reference uploader is available here:

- [`docs/examples/artwork_uploader_server.py`](./examples/artwork_uploader_server.py)

It:

- accepts the exact JSON payload ActivityPing sends
- optionally verifies a bearer token
- stores the uploaded image on disk
- returns a public URL in the format ActivityPing accepts
- serves saved files from `/files/<name>`
