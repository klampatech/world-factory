"""Stateless deterministic sampling for reproducible world generation."""

import hashlib
import struct

from world_factory.constants import DETERMINISTIC_ALGORITHM_VERSION

_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1
_UNIT_INTERVAL_DENOMINATOR = float(1 << 64)


def sample_unit_interval(seed: int, namespace: str, *coordinates: int) -> float:
    """Return a stable value in [0, 1) for a seed, namespace, and coordinates."""
    digest = hashlib.blake2b(digest_size=8, person=b"worldfac")
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(namespace.encode("utf-8"))
    for coordinate in coordinates:
        digest.update(struct.pack(">q", coordinate))
    return int.from_bytes(digest.digest(), "big") / _UNIT_INTERVAL_DENOMINATOR


def deterministic_algorithm_version() -> str:
    """Return the identifier persisted with deterministic outputs."""
    return DETERMINISTIC_ALGORITHM_VERSION
