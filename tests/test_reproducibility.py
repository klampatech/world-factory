"""Reproducibility contract: same config yields byte-identical worlds."""

from pathlib import Path

from world_factory.generator import generate_world
from world_factory.models import WorldConfig
from world_factory.persistence import load_world, save_world, serialize_world


def _config(seed: int = 42) -> WorldConfig:
    return WorldConfig(seed=seed)


def test_same_seed_same_world_id() -> None:
    world_a = generate_world(_config(42))
    world_b = generate_world(_config(42))
    assert world_a.metadata.world_id == world_b.metadata.world_id


def test_same_seed_byte_identical_serialization(tmp_path: Path) -> None:
    first = generate_world(_config(42))
    second = generate_world(_config(42))
    path = tmp_path / "world.json"
    save_world(first, path)
    payload_first = path.read_bytes()
    save_world(second, path)
    payload_second = path.read_bytes()
    assert payload_first == payload_second


def test_serialization_is_deterministic() -> None:
    world = generate_world(_config(42))
    first = serialize_world(world)
    second = serialize_world(world)
    assert first == second


def test_different_seeds_yield_different_worlds() -> None:
    world_a = generate_world(_config(1))
    world_b = generate_world(_config(2))
    assert world_a.metadata.world_id != world_b.metadata.world_id
    assert world_a.geography.elevation_meters != world_b.geography.elevation_meters


def test_round_trip_preserves_identity(tmp_path: Path) -> None:
    world = generate_world(_config(42))
    path = tmp_path / "world.json"
    save_world(world, path)
    reloaded = load_world(path)
    assert reloaded == world
