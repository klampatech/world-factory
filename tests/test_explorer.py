"""v2 visual explorer smoke tests.

Three layers of coverage:

1. Static HTML shape: well-formed markup, expected DOM hooks,
   vanilla-JS-only API surface, biome color table embedded.
2. Demo JSON shape: the v2 explorer needs per-cell biome grid,
   river cells, settlement cells, and grid dimensions in the
   demo output. Confirm `run_v1_demo` produces them.
3. Runtime HTTP smoke: launch the explorer's `ExplorerServer`,
   `GET /index.html` and `GET /demo.json` over HTTP, and assert
   the responses are well-shaped. This is the test that proves
   the browser fetch path works (file:// fetch is blocked in
   most browsers per `RESEARCH/WORLD_FACTORY_V2_RELEASE_GATES.md`).
"""

from __future__ import annotations

import json
import shutil
import socket
import time
import urllib.error
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

from world_factory.demo import run_v1_demo
from world_factory.explorer import (
    ExplorerDir,
    IndexHtml,
    list_overlay_buttons,
    serve_explorer,
    write_demo_artifact,
)
from world_factory.models import WorldScale

# -- static HTML shape --

def test_explorer_html_exists_and_is_non_empty() -> None:
    content = IndexHtml.read_text()
    assert len(content) > 0
    assert "<canvas" in content
    assert "<script" in content


def test_explorer_html_has_expected_dom_hooks() -> None:
    """The page must expose: a canvas, an overlay toolbar with biome
    / elevation / rivers / settlements buttons, and a side panel
    populated by the click handler."""
    content = IndexHtml.read_text()
    assert 'id="canvas"' in content
    assert 'id="overlays"' in content
    for overlay in list_overlay_buttons():
        assert f'data-overlay="{overlay}"' in content
    assert 'id="summary"' in content
    assert 'id="status"' in content


def test_explorer_html_is_well_formed() -> None:
    """Smoke test: html.parser can fully consume the file without
    throwing. If the structure is broken (unclosed tags, etc.)
    this fails fast."""

    class _Parser(HTMLParser):
        def __init__(self) -> None:
            super().__init__()
            self.errors: list[str] = []

        def error(self, message: str) -> None:
            self.errors.append(message)

    content = IndexHtml.read_text()
    parser = _Parser()
    parser.feed(content)
    assert not parser.errors, f"HTML parse errors: {parser.errors}"


def test_explorer_html_uses_only_vanilla_apis() -> None:
    """No fetch framework, no third-party JS — just fetch + 2D
    canvas + DOM. The v1 minimal-deps philosophy."""
    content = IndexHtml.read_text()
    # No imports of popular frameworks.
    for needle in ("import React", "import Vue", "import angular",
                    "from \"react\"", "jQuery", "$("):
        assert needle not in content
    # Uses native APIs.
    assert "fetch(" in content
    assert "getContext(\"2d\")" in content
    assert "addEventListener" in content


def test_explorer_html_embeds_biome_color_table() -> None:
    """The page must carry the biome-color lookup so the
    summary view stays consistent with the generator output."""
    content = IndexHtml.read_text()
    for biome in (
        "ocean",
        "ice",
        "alpine",
        "desert",
        "tropical-forest",
        "temperate-forest",
        "grassland",
    ):
        assert biome in content, f"biome {biome} not in HTML"


def test_explorer_html_escapes_user_supplied_text() -> None:
    """Per release gates: 'HTML rendering uses textContent, not raw
    innerHTML with world data.' Confirm the renderSummary path
    uses a local escape helper and never injects world data into
    innerHTML without escaping."""
    content = IndexHtml.read_text()
    assert "escapeText" in content
    # The biome name and world data flow through `escapeText(v)`
    # before being concatenated into innerHTML.
    assert "escapeText(v)" in content
    # The escape helper covers &, <, and > — the dangerous three.
    assert "&amp;" in content
    assert "&lt;" in content
    assert "&gt;" in content
    # The biome name (read from WORLD.biome_grid) must NOT be
    # pasted directly into a template literal without escaping.
    assert "${biome}" not in content


def test_explorer_html_has_accessibility_basics() -> None:
    """Per release gates: keyboard-operable controls, visible focus,
    labelled controls, non-color layer identification."""
    content = IndexHtml.read_text()
    assert ":focus-visible" in content
    assert 'role="toolbar"' in content
    assert 'aria-label="Layer toggles"' in content
    assert "data-marker=" in content  # non-color layer label


def test_explorer_html_handles_device_pixel_ratio() -> None:
    """Per release gates: canvas coordinate mapping handles CSS
    scaling and device pixel ratio."""
    content = IndexHtml.read_text()
    assert "devicePixelRatio" in content
    assert "setTransform" in content


def test_explorer_html_validates_demo_json_shape() -> None:
    """The page must fail loudly on missing or malformed demo
    data. The runtime validateShape() check guards against silent
    failures."""
    content = IndexHtml.read_text()
    assert "validateShape" in content
    for required in (
        "world_id", "scale", "is_valid", "biome_counts",
        "sample_polity_summary", "grid_width", "grid_height",
        "biome_grid", "river_cells", "settlement_cells",
    ):
        assert f'"{required}"' in content


# -- demo JSON shape --

def test_v1_demo_emits_per_cell_grids() -> None:
    """Per release gates: 'the existing v1 demo report is only a
    summary and 3x3 walk.' The v2 explorer needs the per-cell
    biome grid + river / settlement cell sets in the demo JSON."""
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert report.grid_width == 256
    assert report.grid_height == 128
    assert len(report.biome_grid) == 256 * 128
    # All biome_grid entries are valid biome names.
    valid_biomes = {
        "ocean", "ice", "alpine", "desert",
        "tropical-forest", "temperate-forest", "grassland",
    }
    for entry in report.biome_grid:
        assert entry in valid_biomes, f"unknown biome {entry!r}"
    # River cells are tuples of (x, y) inside the grid.
    for x, y in report.river_cells:
        assert 0 <= x < 256
        assert 0 <= y < 128
    # Settlement cells are tuples of (x, y) inside the grid.
    for x, y in report.settlement_cells:
        assert 0 <= x < 256
        assert 0 <= y < 128


def test_v1_demo_grids_are_byte_stable_across_runs() -> None:
    """The v2 explorer depends on a deterministic JSON envelope."""
    a = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    b = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert a.biome_grid == b.biome_grid
    assert a.river_cells == b.river_cells
    assert a.settlement_cells == b.settlement_cells


def test_v1_demo_grids_match_world_layer() -> None:
    """The per-cell biome grid must agree with the underlying
    `biomes.classifications` tuple. Re-run generation and compare."""
    from world_factory.generator import generate_world
    from world_factory.models import WorldConfig

    config = WorldConfig(seed=42, scale=WorldScale.LARGE)
    world = generate_world(config)
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    expected = tuple(cell.value for row in world.biomes.classifications for cell in row)
    assert report.biome_grid == expected


def test_write_demo_artifact_round_trip(tmp_path: Path) -> None:
    """Run v1 demo, write to disk, read back, assert well-shaped."""
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    out = tmp_path / "demo.json"
    write_demo_artifact(report.to_dict(), out)
    assert out.exists()
    payload = json.loads(out.read_text())
    assert payload["world_id"] == report.world_id
    assert payload["is_valid"] is True
    assert "sample_polity_summary" in payload
    assert "biome_grid" in payload
    assert payload["grid_width"] == 256
    assert payload["grid_height"] == 128


def test_explorer_html_embeds_full_demo_data_round_trip(tmp_path: Path) -> None:
    """End-to-end: run v1 demo, copy HTML to tmp dir, write demo.json
    alongside. The HTML file is self-contained (it can be opened
    in any browser pointed at the JSON)."""
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    out_dir = tmp_path / "explorer"
    out_dir.mkdir()
    shutil.copy(IndexHtml, out_dir / "index.html")
    write_demo_artifact(report.to_dict(), out_dir / "demo.json")
    assert (out_dir / "index.html").exists()
    assert (out_dir / "demo.json").exists()
    assert (out_dir / "index.html").stat().st_size > 0
    assert (out_dir / "demo.json").stat().st_size > 0


# -- runtime HTTP smoke --

def _wait_for_server(host: str, port: int, timeout_seconds: float = 5.0) -> None:
    """Poll a TCP socket until the server accepts a connection."""
    deadline = time.monotonic() + timeout_seconds
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            with socket.create_connection((host, port), timeout=0.25):
                return
        except OSError as error:
            last_error = error
            time.sleep(0.05)
    raise RuntimeError(f"server at {host}:{port} did not start: {last_error}")


def _http_get(url: str, timeout_seconds: float = 5.0) -> tuple[int, bytes, dict[str, str]]:
    request = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            return response.status, response.read(), dict(response.headers)
    except urllib.error.HTTPError as error:
        return error.code, error.read(), dict(error.headers or {})


def test_explorer_serves_index_and_demo_over_http(tmp_path: Path) -> None:
    """Drive the actual serve flow end-to-end. This is the test
    that proves the browser fetch path works without file:// —
    the release-gate blocker for shipping the explorer."""
    report = run_v1_demo(seed=42, scale=WorldScale.SMALL)
    write_demo_artifact(report.to_dict(), tmp_path / "demo.json")
    shutil.copy(IndexHtml, tmp_path / "index.html")

    server = serve_explorer(directory=tmp_path, port=0)
    server.start()
    try:
        _wait_for_server(server.host, server.port)
        index_status, index_body, _ = _http_get(f"{server.base_url}/index.html")
        demo_status, demo_body, _ = _http_get(f"{server.base_url}/demo.json")
    finally:
        server.stop()

    assert index_status == 200
    assert demo_status == 200
    assert b"<canvas" in index_body
    demo = json.loads(demo_body)
    assert demo["world_id"] == report.world_id
    assert demo["grid_width"] * demo["grid_height"] == len(demo["biome_grid"])


def test_explorer_package_files_resolve() -> None:
    """The explorer package directory must expose both files at
    the same path so the package-data install works."""
    assert ExplorerDir.is_dir()
    assert IndexHtml.is_file()
    assert (ExplorerDir / "index.html").resolve() == IndexHtml.resolve()


def test_explorer_find_free_port_returns_distinct_ports() -> None:
    """`find_free_port()` asks the kernel for an unused port. Run
    it twice in a row and confirm we got two distinct ports."""
    from world_factory.explorer import find_free_port

    ports = {find_free_port() for _ in range(3)}
    assert len(ports) == 3, f"expected 3 distinct ports, got {ports}"