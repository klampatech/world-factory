"""Persistence: atomic save and strict load at the trust boundary."""

import json
from pathlib import Path

import pytest
from pydantic import ValidationError

from world_factory import persistence
from world_factory.generator import generate_world
from world_factory.models import WorldConfig
from world_factory.persistence import load_world, save_world


def test_save_writes_valid_json(tmp_path: Path) -> None:
    world = generate_world(WorldConfig(seed=42))
    path = tmp_path / "subdir" / "world.json"
    save_world(world, path)
    assert path.is_file()
    payload = json.loads(path.read_text())
    assert payload["metadata"]["world_id"] == world.metadata.world_id


def test_load_round_trip_preserves_identity(tmp_path: Path) -> None:
    world = generate_world(WorldConfig(seed=42))
    path = tmp_path / "world.json"
    save_world(world, path)
    reloaded = load_world(path)
    assert reloaded == world


def test_load_rejects_unknown_fields(tmp_path: Path) -> None:
    path = tmp_path / "world.json"
    path.write_text(
        '{"metadata": {"world_id": "deadbeefdeadbeef01",'
        ' "schema_version": "1.0.0", "model_version": "phase-0.1",'
        ' "config": {"seed": 42}}, "surprise": true}'
    )
    with pytest.raises(ValidationError):
        load_world(path)


def test_save_is_atomic_on_failure(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """If a write fails mid-flight, the destination must not exist."""
    world = generate_world(WorldConfig(seed=42))
    path = tmp_path / "world.json"

    def _failing_replace(src: str, dst: str) -> None:
        raise OSError("simulated fs failure")

    monkeypatch.setattr(persistence.os, "replace", _failing_replace)
    with pytest.raises(OSError):
        save_world(world, path)
    assert not path.exists(), "partial file should have been removed"
    leftovers = list(tmp_path.glob(".world-*"))
    assert leftovers == [], f"temp file should have been cleaned up: {leftovers}"
