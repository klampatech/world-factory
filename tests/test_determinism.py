"""Stateless deterministic sampler: stable across order and partition."""

from world_factory.determinism import sample_unit_interval


def test_same_inputs_same_output() -> None:
    assert sample_unit_interval(42, "elevation", 0, 1) == sample_unit_interval(
        42, "elevation", 0, 1
    )


def test_different_seeds_yield_different_outputs() -> None:
    assert sample_unit_interval(1, "elevation", 0, 1) != sample_unit_interval(2, "elevation", 0, 1)


def test_different_namespaces_yield_different_outputs() -> None:
    assert sample_unit_interval(42, "elevation", 0, 1) != sample_unit_interval(
        42, "precipitation", 0, 1
    )


def test_output_in_unit_interval() -> None:
    for seed in (0, 1, 42, 1 << 32):
        for namespace in ("elevation", "precipitation", "climate"):
            value = sample_unit_interval(seed, namespace, 3, 7)
            assert 0.0 <= value < 1.0


def test_different_coordinates_yield_different_outputs() -> None:
    assert sample_unit_interval(42, "elevation", 0, 1) != sample_unit_interval(
        42, "elevation", 1, 0
    )
