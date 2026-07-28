"""CLI smoke tests: generate + validate against the canonical demo seed."""

import json
import subprocess
import sys
from pathlib import Path


def test_generate_writes_valid_world(tmp_path: Path) -> None:
    out = tmp_path / "demo.json"
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "world_factory.cli",
            "generate",
            "--seed",
            "42",
            "--scale",
            "small",
            "--climate",
            "temperate",
            "--sentience",
            "--no-magic",
            "--out",
            str(out),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    assert out.is_file()
    payload = json.loads(out.read_text())
    assert payload["metadata"]["config"]["seed"] == 42


def test_validate_accepts_generated_world(tmp_path: Path) -> None:
    out = tmp_path / "demo.json"
    subprocess.run(
        [
            sys.executable,
            "-m",
            "world_factory.cli",
            "generate",
            "--seed",
            "42",
            "--out",
            str(out),
        ],
        check=True,
        capture_output=True,
    )
    result = subprocess.run(
        [sys.executable, "-m", "world_factory.cli", "validate", str(out)],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["data"]["is_valid"] is True


def test_generate_is_byte_identical_across_runs(tmp_path: Path) -> None:
    out_a = tmp_path / "a.json"
    out_b = tmp_path / "b.json"
    for out in (out_a, out_b):
        subprocess.run(
            [
                sys.executable,
                "-m",
                "world_factory.cli",
                "generate",
                "--seed",
                "42",
                "--out",
                str(out),
            ],
            check=True,
            capture_output=True,
        )
    assert out_a.read_bytes() == out_b.read_bytes()
