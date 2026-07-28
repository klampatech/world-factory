"""Canonical, atomic persistence for versioned worlds."""

import json
import os
import tempfile
from pathlib import Path

from world_factory.models import WorldModel


def serialize_world(world: WorldModel) -> bytes:
    """Serialize a world to stable UTF-8 JSON bytes."""
    payload = json.dumps(
        world.model_dump(mode="json"),
        allow_nan=False,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    )
    return f"{payload}\n".encode()


def save_world(world: WorldModel, destination: Path) -> None:
    """Atomically persist a world without exposing partial output."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    file_descriptor, temporary_name = tempfile.mkstemp(dir=destination.parent, prefix=".world-")
    try:
        with os.fdopen(file_descriptor, "wb") as temporary_file:
            temporary_file.write(serialize_world(world))
            temporary_file.flush()
            os.fsync(temporary_file.fileno())
        os.replace(temporary_name, destination)
    except BaseException:
        Path(temporary_name).unlink(missing_ok=True)
        raise


def load_world(source: Path) -> WorldModel:
    """Load and strictly validate a persisted world at the trust boundary."""
    return WorldModel.model_validate_json(source.read_bytes(), strict=True)
