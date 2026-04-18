import base64
import json
import mimetypes
import os
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlparse


HOST = os.environ.get("HOST", "127.0.0.1")
PORT = int(os.environ.get("PORT", "8787"))
STORAGE_DIR = Path(os.environ.get("STORAGE_DIR", "./uploads")).resolve()
PUBLIC_BASE_URL = os.environ.get("PUBLIC_BASE_URL", f"http://{HOST}:{PORT}").rstrip("/")
UPLOAD_TOKEN = os.environ.get("UPLOAD_TOKEN", "").strip()


def json_response(handler: BaseHTTPRequestHandler, status: int, payload: dict) -> None:
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def file_response(handler: BaseHTTPRequestHandler, path: Path) -> None:
    if not path.is_file():
        json_response(handler, HTTPStatus.NOT_FOUND, {"ok": False, "error": "file not found"})
        return

    body = path.read_bytes()
    mime, _ = mimetypes.guess_type(str(path))
    handler.send_response(HTTPStatus.OK)
    handler.send_header("Content-Type", mime or "application/octet-stream")
    handler.send_header("Content-Length", str(len(body)))
    handler.send_header("Cache-Control", "public, max-age=3600")
    handler.end_headers()
    handler.wfile.write(body)


def parse_data_url(value: str) -> bytes:
    if not value:
        raise ValueError("missing base64 payload")

    if "," in value:
        _, encoded = value.split(",", 1)
    else:
        encoded = value

    try:
        return base64.b64decode(encoded, validate=True)
    except Exception as exc:
        raise ValueError(f"invalid base64 payload: {exc}") from exc


def safe_file_name(value: str) -> str:
    name = os.path.basename((value or "").strip())
    if not name:
        raise ValueError("missing fileName")
    return name


class ArtworkUploaderHandler(BaseHTTPRequestHandler):
    server_version = "ActivityPingUploader/1.0"

    def do_POST(self) -> None:
        if self.path != "/upload":
            json_response(self, HTTPStatus.NOT_FOUND, {"ok": False, "error": "not found"})
            return

        if UPLOAD_TOKEN:
            auth = self.headers.get("Authorization", "")
            expected = f"Bearer {UPLOAD_TOKEN}"
            if auth != expected:
                json_response(self, HTTPStatus.UNAUTHORIZED, {"ok": False, "error": "invalid token"})
                return

        content_length = self.headers.get("Content-Length", "0").strip()
        if not content_length.isdigit():
            json_response(self, HTTPStatus.BAD_REQUEST, {"ok": False, "error": "invalid content length"})
            return

        raw_body = self.rfile.read(int(content_length))
        try:
            payload = json.loads(raw_body.decode("utf-8"))
        except Exception as exc:
            json_response(self, HTTPStatus.BAD_REQUEST, {"ok": False, "error": f"invalid json: {exc}"})
            return

        try:
            image_value = payload.get("imageBase64") or payload.get("base64") or ""
            file_name = safe_file_name(payload.get("fileName", ""))
            image_bytes = parse_data_url(image_value)
        except ValueError as exc:
            json_response(self, HTTPStatus.BAD_REQUEST, {"ok": False, "error": str(exc)})
            return

        STORAGE_DIR.mkdir(parents=True, exist_ok=True)
        output_path = STORAGE_DIR / file_name
        output_path.write_bytes(image_bytes)

        public_url = f"{PUBLIC_BASE_URL}/files/{file_name}"
        json_response(
            self,
            HTTPStatus.OK,
            {
                "ok": True,
                "url": public_url,
            },
        )

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if not parsed.path.startswith("/files/"):
            json_response(self, HTTPStatus.NOT_FOUND, {"ok": False, "error": "not found"})
            return

        file_name = unquote(parsed.path.removeprefix("/files/"))
        if not file_name or "/" in file_name or "\\" in file_name:
            json_response(self, HTTPStatus.BAD_REQUEST, {"ok": False, "error": "invalid file name"})
            return

        file_response(self, STORAGE_DIR / file_name)

    def log_message(self, format: str, *args) -> None:
        print(f"{self.address_string()} - {format % args}")


if __name__ == "__main__":
    STORAGE_DIR.mkdir(parents=True, exist_ok=True)
    server = ThreadingHTTPServer((HOST, PORT), ArtworkUploaderHandler)
    print(f"Listening on http://{HOST}:{PORT}")
    print(f"Serving files from {STORAGE_DIR}")
    print("POST /upload")
    print("GET  /files/<name>")
    server.serve_forever()
