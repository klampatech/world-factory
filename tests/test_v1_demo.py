"""v1 demo walkthrough tests.

The v1 demo exercises the full pipeline end-to-end:
generate -> validate -> summary statistics -> sample polity -> sample
bioregion -> query surface round-trip. These tests confirm the
walkthrough runs and the output is well-shaped.
"""

import json
import subprocess
import sys

from world_factory.demo import V1DemoReport, run_v1_demo
from world_factory.models import WorldScale


def test_v1_demo_runs_at_large_scale() -> None:
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert isinstance(report, V1DemoReport)
    assert report.is_valid
    assert report.seed == 42
    assert report.scale == "large"
    assert report.schema_version == "18.0.0"
    assert report.total_cells == 256 * 128
    assert report.ocean_cells + report.land_cells == report.total_cells
    assert report.surface_water_fraction > 0.0


def test_v1_demo_biome_counts_sum_to_total_cells() -> None:
    """`biome_counts` enumerates every cell, including OCEAN, so
    its sum should equal `total_cells` directly (not total - ocean)."""
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    biome_cells = sum(entry["cells"] for entry in report.biome_counts)
    assert biome_cells == report.total_cells


def test_v1_demo_settlements_have_population() -> None:
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert report.settlement_count > 0
    assert report.total_population > 0
    assert report.sample_polity_summary.settlements
    assert any(s.population > 0 for s in report.sample_polity_summary.settlements)


def test_v1_demo_sample_bioregion_is_3x3() -> None:
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert len(report.sample_bioregion_summaries) == 9


def test_v1_demo_query_surface_validates() -> None:
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert report.query_surface_validates


def test_v1_demo_to_dict_is_json_serializable() -> None:
    report = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    payload = report.to_dict()
    encoded = json.dumps(payload, allow_nan=False)
    decoded = json.loads(encoded)
    assert decoded["world_id"] == report.world_id
    assert decoded["settlement_count"] == report.settlement_count


def test_v1_demo_deterministic_across_runs() -> None:
    a = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    b = run_v1_demo(seed=42, scale=WorldScale.LARGE)
    assert a.world_id == b.world_id
    assert a.settlement_count == b.settlement_count
    a_total_pop = sum(s.population for s in a.sample_polity_summary.settlements)
    b_total_pop = sum(s.population for s in b.sample_polity_summary.settlements)
    assert a_total_pop == b_total_pop


def test_v1_demo_cli_runs_end_to_end(tmp_path) -> None:
    """Drive the CLI as a subprocess to confirm the public surface
    is functional and the output is valid JSON."""
    out_file = tmp_path / "demo.json"
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "world_factory.cli",
            "demo",
            "--seed",
            "42",
            "--scale",
            "large",
            "--out",
            str(out_file),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    payload = json.loads(out_file.read_text())
    assert payload["data"]["is_valid"] is True
    assert payload["data"]["world_id"]


def test_v1_demo_different_seeds_produce_different_worlds() -> None:
    a = run_v1_demo(seed=1, scale=WorldScale.SMALL)
    b = run_v1_demo(seed=2, scale=WorldScale.SMALL)
    assert a.world_id != b.world_id


def test_v1_demo_small_scale_runs() -> None:
    report = run_v1_demo(seed=42, scale=WorldScale.SMALL)
    assert report.is_valid
    assert report.total_cells == 24 * 12