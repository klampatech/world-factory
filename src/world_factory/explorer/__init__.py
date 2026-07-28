"""Static HTML visual explorer for the world-factory v2 demo.

The explorer is a single self-contained HTML file that loads a
`demo.json` artifact and renders a 2D canvas with toggleable
overlays (biome / elevation / rivers / settlements) plus a
click-to-summary drilldown. Browser `fetch()` requires an HTTP
origin (file:// is blocked in most browsers), so the package
ships a `serve_explorer` helper that runs `python -m http.server`
on a chosen port. The CLI exposes this as `world-factory serve`.
"""

from __future__ import annotations

import http.server
import json
import socketserver
import threading
from collections.abc import Iterable
from pathlib import Path
from typing import Any

ExplorerDir = Path(__file__).resolve().parent
IndexHtml = ExplorerDir / "index.html"

_DEFAULT_BIND_HOST = "127.0.0.1"
_DEFAULT_PORT = 8765


def write_demo_artifact(  # pragma: no cover - dev helper, used by tests
    world_payload: dict[str, Any], destination: Path
) -> Path:
    """Write a `demo.json` artifact next to the HTML so the explorer
    page can fetch it via a relative URL."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        json.dumps(world_payload, allow_nan=False, sort_keys=True, indent=2) + "\n"
    )
    return destination


def find_free_port() -> int:
    """Bind to port 0 to ask the kernel for a free ephemeral port,
    then release it. Race-prone but adequate for the test harness
    that runs a single server at a time."""
    handler = http.server.BaseHTTPRequestHandler
    with socketserver.TCPServer((_DEFAULT_BIND_HOST, 0), handler) as probe:
        return probe.server_address[1]


class _ReusableThreadingHTTPServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    """Threaded HTTP server with address-reuse so quick restarts in
    tests do not hit TIME_WAIT."""

    allow_reuse_address = True
    daemon_threads = True


def _make_handler(directory: Path) -> type[http.server.SimpleHTTPRequestHandler]:
    """Build a SimpleHTTPRequestHandler rooted at `directory`."""
    rooted_directory = str(directory)

    class _RootedHandler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args: Any, **kwargs: Any) -> None:
            super().__init__(*args, directory=rooted_directory, **kwargs)

    return _RootedHandler


class ExplorerServer:
    """A running HTTP server that serves the explorer directory.
    `start()` binds, `stop()` releases the port. Holds a thread
    handle so callers can join or kill deterministically."""

    def __init__(self, directory: Path, port: int, host: str = _DEFAULT_BIND_HOST) -> None:
        self.directory = directory
        self.port = port
        self.host = host
        self._server: _ReusableThreadingHTTPServer | None = None
        self._thread: threading.Thread | None = None

    def start(self) -> None:
        handler_cls = _make_handler(self.directory)
        self._server = _ReusableThreadingHTTPServer(
            (self.host, self.port), handler_cls
        )
        self._thread = threading.Thread(
            target=self._server.serve_forever, name="explorer-http", daemon=True
        )
        self._thread.start()

    @property
    def base_url(self) -> str:
        return f"http://{self.host}:{self.port}"

    def stop(self) -> None:
        if self._server is not None:
            self._server.shutdown()
            self._server.server_close()
            self._server = None
        if self._thread is not None:
            self._thread.join(timeout=2.0)
            self._thread = None

    def __enter__(self) -> ExplorerServer:
        self.start()
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.stop()


def serve_explorer(
    directory: Path | None = None,
    port: int | None = None,
    host: str = _DEFAULT_BIND_HOST,
) -> ExplorerServer:
    """Build (but do not start) an `ExplorerServer`. Call `.start()`
    on the returned object to bind the port. Pass `directory=None`
    to default to the explorer package directory; pass `port=None`
    or `port=0` to bind a kernel-assigned ephemeral port."""
    if port is None or port == 0:
        port = find_free_port()
    return ExplorerServer(
        directory=(directory or ExplorerDir),
        port=port,
        host=host,
    )


def list_overlay_buttons() -> Iterable[str]:
    """Return the overlay-button names the explorer HTML wires up.
    Used by tests that want to confirm the four toggles survived
    any future HTML rewrite."""
    return ("biome", "elevation", "rivers", "settlements")


__all__ = [
    "ExplorerDir",
    "ExplorerServer",
    "IndexHtml",
    "_DEFAULT_BIND_HOST",
    "_DEFAULT_PORT",
    "find_free_port",
    "list_overlay_buttons",
    "serve_explorer",
    "write_demo_artifact",
]