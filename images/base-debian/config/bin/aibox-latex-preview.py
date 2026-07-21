#!/usr/bin/env python3
"""Serve configured LaTeX PDFs through a small EmbedPDF web UI."""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import threading
import time
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit


EMBEDPDF_VERSION = "2.14.3"
SAFE_NAME = re.compile(r"^[A-Za-z0-9_-]+$")


@dataclass
class Document:
    name: str
    source: str
    pdf: Path
    revision: int = 0
    condition: threading.Condition = field(default_factory=threading.Condition)


def file_revision(path: Path) -> int:
    try:
        stat = path.stat()
    except OSError:
        return 0
    return stat.st_mtime_ns ^ (stat.st_size << 17)


def watch_pdf(document: Document) -> None:
    candidate = file_revision(document.pdf)
    stable_polls = 0
    while True:
        time.sleep(0.25)
        observed = file_revision(document.pdf)
        if observed == candidate:
            stable_polls += 1
        else:
            candidate = observed
            stable_polls = 0
        if stable_polls < 2:
            continue
        with document.condition:
            if candidate != document.revision:
                document.revision = candidate
                document.condition.notify_all()


def index_html(documents: list[Document]) -> str:
    links = "".join(
        f'<li><a href="/documents/{document.name}/"><strong>{html.escape(document.name)}</strong>'
        f"<span>{html.escape(document.source)}</span></a></li>"
        for document in documents
    )
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>LaTeX documents</title>
<style>
:root{{color-scheme:light dark;font-family:system-ui,sans-serif}}body{{margin:0;background:#f4f6f8;color:#18202a}}
main{{max-width:760px;margin:0 auto;padding:48px 24px}}h1{{font-size:28px;letter-spacing:0;margin:0 0 8px}}p{{margin:0 0 28px;color:#586474}}
ul{{list-style:none;margin:0;padding:0;border:1px solid #cbd2da;border-radius:8px;overflow:hidden;background:#fff}}
li+li{{border-top:1px solid #dce1e6}}a{{display:flex;align-items:center;justify-content:space-between;gap:24px;padding:18px 20px;color:inherit;text-decoration:none}}
a:hover,a:focus-visible{{background:#edf5ff;outline:none}}strong{{font-size:16px}}span{{color:#687585;font:13px ui-monospace,monospace;text-align:right;overflow-wrap:anywhere}}
@media(max-width:560px){{main{{padding:32px 16px}}a{{align-items:flex-start;flex-direction:column;gap:6px}}span{{text-align:left}}}}
@media(prefers-color-scheme:dark){{body{{background:#11161c;color:#edf2f7}}p,span{{color:#9da9b7}}ul{{background:#171d24;border-color:#3a4653}}li+li{{border-color:#303a45}}a:hover,a:focus-visible{{background:#1b3045}}}}
</style></head><body><main><h1>LaTeX documents</h1><p>Select a document to open its live PDF preview.</p><ul>{links}</ul></main></body></html>"""


def viewer_html(document: Document, base_path: str) -> str:
    safe_title = html.escape(document.name)
    pdf_url = f"{base_path}/document.pdf"
    events_url = f"{base_path}/events"
    storage_key = f"aibox-latex-{document.name}"
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{safe_title} - LaTeX preview</title>
<style>html,body,#viewer{{height:100%;margin:0}}#waiting{{font:14px system-ui;padding:16px}}#documents{{position:fixed;z-index:5;top:10px;left:10px;padding:7px 10px;border-radius:6px;background:#202833;color:#fff;font:13px system-ui;text-decoration:none}}</style></head>
<body><a id="documents" href="/">Documents</a><div id="waiting">Waiting for the first completed PDF build...</div><div id="viewer"></div>
<script type="module">
import EmbedPDF from 'https://cdn.jsdelivr.net/npm/@embedpdf/snippet@{EMBEDPDF_VERSION}/dist/embedpdf.js';
let revision = Date.now();
async function openPdf() {{
  const response = await fetch(`{pdf_url}?v=${{revision}}`, {{method:'HEAD', cache:'no-store'}});
  if (!response.ok) return;
  document.querySelector('#waiting').remove();
  const viewer = EmbedPDF.init({{
    type:'container', target:document.querySelector('#viewer'), theme:{{preference:'system'}},
    documentManager:{{initialDocuments:[{{url:`{pdf_url}?v=${{revision}}`, documentId:'live-preview'}}]}}
  }});
  if (!viewer) return;
  const registry = await viewer.registry;
  const scroll = registry.getPlugin('scroll')?.provides();
  const documentScroll = scroll?.forDocument('live-preview');
  const zoom = registry.getPlugin('zoom')?.provides()?.forDocument('live-preview');
  const savedPage = Number(sessionStorage.getItem('{storage_key}-page') || '1');
  const savedZoom = JSON.parse(sessionStorage.getItem('{storage_key}-zoom') || 'null');
  let restoringView = true;
  scroll?.onLayoutReady(event => {{
    if (event.documentId !== 'live-preview' || !event.isInitial) return;
    documentScroll?.scrollToPage({{pageNumber:savedPage, behavior:'instant'}});
    if (savedZoom !== null) zoom?.requestZoom(savedZoom);
    requestAnimationFrame(() => {{ restoringView = false; }});
  }});
  scroll?.onPageChange(event => {{
    if (event.documentId === 'live-preview' && !restoringView) sessionStorage.setItem('{storage_key}-page', String(event.pageNumber));
  }});
  zoom?.onStateChange(state => {{
    if (!restoringView) sessionStorage.setItem('{storage_key}-zoom', JSON.stringify(state.currentZoomLevel));
  }});
}}
await openPdf();
const events = new EventSource('{events_url}');
events.addEventListener('revision', event => {{
  const next = Number(event.data);
  if (next && next !== revision) {{ revision = next; location.reload(); }}
}});
</script></body></html>"""


class PreviewServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(self, address: tuple[str, int], documents: list[Document], preferred: str):
        super().__init__(address, PreviewHandler)
        self.documents = documents
        self.documents_by_name = {document.name: document for document in documents}
        self.preferred = preferred


class PreviewHandler(BaseHTTPRequestHandler):
    server: PreviewServer

    def do_GET(self) -> None:
        self._route(head_only=False)

    def do_HEAD(self) -> None:
        self._route(head_only=True)

    def log_message(self, message: str, *args: object) -> None:
        print(f"{self.address_string()} - {message % args}", flush=True)

    def _route(self, head_only: bool) -> None:
        path = urlsplit(self.path).path
        if path == "/health":
            self._send_bytes(b'{"status":"ok"}', "application/json", head_only)
            return
        if path == "/":
            if len(self.server.documents) == 1:
                body = viewer_html(self.server.documents[0], "")
            else:
                body = index_html(self.server.documents)
            self._send_bytes(body.encode(), "text/html; charset=utf-8", head_only)
            return

        if path in {"/document.pdf", "/events"}:
            document = self.server.documents_by_name[self.server.preferred]
            endpoint = path[1:]
        else:
            parts = path.strip("/").split("/")
            if len(parts) not in {2, 3} or parts[0] != "documents":
                self.send_error(404)
                return
            document = self.server.documents_by_name.get(parts[1])
            if document is None:
                self.send_error(404)
                return
            endpoint = parts[2] if len(parts) == 3 else ""

        if endpoint == "":
            body = viewer_html(document, f"/documents/{document.name}")
            self._send_bytes(body.encode(), "text/html; charset=utf-8", head_only)
        elif endpoint == "document.pdf":
            self._send_pdf(document, head_only)
        elif endpoint == "events" and not head_only:
            self._send_events(document)
        else:
            self.send_error(404)

    def _send_bytes(self, body: bytes, content_type: str, head_only: bool) -> None:
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        if not head_only:
            self.wfile.write(body)

    def _send_pdf(self, document: Document, head_only: bool) -> None:
        try:
            size = document.pdf.stat().st_size
            pdf = None if head_only else document.pdf.open("rb")
        except OSError:
            self.send_error(404, "PDF has not been built yet")
            return
        self.send_response(200)
        self.send_header("Content-Type", "application/pdf")
        self.send_header("Content-Length", str(size))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        if pdf is not None:
            with pdf:
                while chunk := pdf.read(64 * 1024):
                    self.wfile.write(chunk)

    def _send_events(self, document: Document) -> None:
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("X-Accel-Buffering", "no")
        self.end_headers()
        last = document.revision
        try:
            while True:
                with document.condition:
                    document.condition.wait_for(lambda: document.revision != last, timeout=15)
                    current = document.revision
                if current != last:
                    last = current
                    payload = f"event: revision\ndata: {last}\n\n"
                else:
                    payload = ": keepalive\n\n"
                self.wfile.write(payload.encode())
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            return


def parse_documents(raw: str) -> list[Document]:
    values = json.loads(raw)
    if not isinstance(values, list) or not values:
        raise ValueError("at least one LaTeX document is required")
    documents = []
    seen = set()
    for value in values:
        name = value["name"]
        if not isinstance(name, str) or not SAFE_NAME.fullmatch(name) or name in seen:
            raise ValueError(f"invalid or duplicate document name: {name!r}")
        seen.add(name)
        documents.append(
            Document(name=name, source=value["source"], pdf=Path(value["pdf"]))
        )
    return documents


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--documents-json", default=os.environ.get("AIBOX_LATEX_DOCUMENTS_JSON")
    )
    parser.add_argument(
        "--preferred", default=os.environ.get("AIBOX_LATEX_PREFERRED_DOCUMENT")
    )
    parser.add_argument("--bind", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8765)
    args = parser.parse_args()
    if not args.documents_json:
        parser.error("--documents-json or AIBOX_LATEX_DOCUMENTS_JSON is required")
    documents = parse_documents(args.documents_json)
    preferred = args.preferred or documents[0].name
    if preferred not in {document.name for document in documents}:
        parser.error(f"preferred document is not configured: {preferred}")
    for document in documents:
        document.revision = file_revision(document.pdf)
        threading.Thread(target=watch_pdf, args=(document,), daemon=True).start()
    server = PreviewServer((args.bind, args.port), documents, preferred)
    print(
        f"Serving {len(documents)} LaTeX document(s) on http://{args.bind}:{args.port}/",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
