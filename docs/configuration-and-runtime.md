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

If app artwork or music artwork is enabled, ActivityPing converts images to 128x128 JPEG and uploads them through your configured uploader service. The uploader must return a public image URL that Discord can fetch.  (SO WHY THEY DONT ACCEPT BASE64 IMG??????)

If you do not need image-rich Rich Presence, leave artwork uploads disabled.

### Request Format

ActivityPing sends an HTTP `POST` request to your configured uploader URL with:

- `Content-Type: application/json`
- `Authorization: Bearer <token>` when `discordArtworkWorkerToken` is configured
- a 10 second request timeout

Before sending, ActivityPing:

- decodes the original artwork
- resizes it to fit within `128x128`
- re-encodes it as JPEG
- wraps it as a data URL

Request JSON:

```json
{
  "base64": "data:image/jpeg;base64,...",
  "imageBase64": "data:image/jpeg;base64,...",
  "fileName": "etag.jpg",
  "expiresIn": 3600
}
```

Notes:

- `base64` and `imageBase64` currently contain the same value
- `fileName` is the generated cache key with a `.jpg` suffix
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
- stores the JPEG on disk
- returns a public URL in the format ActivityPing accepts
- serves saved files from `/files/<name>`
