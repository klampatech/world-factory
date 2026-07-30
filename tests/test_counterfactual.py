"""Phase 5.1 counterfactual operation — operation, not stored layer.

`run_counterfactual(world, intervention)` returns a
`CounterfactualRun` record. v1 ships `RemoveEventMutation` only.
Acceptance gate 5.4: counterfactual reproducibility — same
`(seed, intervention)` -> byte-identical `CounterfactualRun`.
"""

from __future__ import annotations

import pytest

from world_factory.constants import COUNTERFACTUAL_ALGORITHM_VERSION
from world_factory.counterfactual import (
    Intervention,
    counterfactual_provenance,
    run_counterfactual,
    validate_counterfactual_run,
)
from world_factory.generator import generate_world
from world_factory.models import (
    CounterfactualRun,
    RemoveEventMutation,
    WorldConfig,
    WorldScale,
)


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_counterfactual_run_returns_record() -> None:
    """`run_counterfactual` returns a `CounterfactualRun` record."""
    world = generate_world(_config())
    target = world.events.events[10]
    mutation = RemoveEventMutation(event_id=target.id)
    intervention: Intervention = (target.id, mutation)
    run = run_counterfactual(world, intervention)
    assert isinstance(run, CounterfactualRun)


def test_counterfactual_alt_events_excludes_target() -> None:
    """Alternate timeline does not include the removed event."""
    world = generate_world(_config())
    target = world.events.events[10]
    intervention: Intervention = (target.id, RemoveEventMutation(event_id=target.id))
    run = run_counterfactual(world, intervention)
    target_ids = {event.id for event in run.alternate_events}
    assert target.id not in target_ids, (
        f"alternate timeline includes the removed event {target.id!r}"
    )


def test_counterfactual_5_4_acceptance_byte_equal() -> None:
    """5.4 acceptance gate: counterfactual reproducibility. Same
    `(seed, intervention)` -> byte-identical `CounterfactualRun`."""
    world_a = generate_world(_config())
    world_b = generate_world(_config())
    target = world_a.events.events[10]
    intervention: Intervention = (
        target.id,
        RemoveEventMutation(event_id=target.id),
    )
    run_a = run_counterfactual(world_a, intervention)
    run_b = run_counterfactual(world_b, intervention)
    assert (
        run_a.model_dump() == run_b.model_dump()
    ), "same seed produced different counterfactual runs"


def test_counterfactual_rejects_unknown_event() -> None:
    """`run_counterfactual` raises `ValueError` for an event id not
    in the union of `world.events.events` and
    `world.polities.events`."""
    world = generate_world(_config())
    bogus_id = "ffffffffffffffff"
    with pytest.raises(ValueError):
        run_counterfactual(
            world,
            (bogus_id, RemoveEventMutation(event_id=bogus_id)),
        )


def test_counterfactual_rejects_mismatched_intervention() -> None:
    """`run_counterfactual` raises `ValueError` when the
    intervention tuple's event_id does not match
    `mutation.event_id`."""
    world = generate_world(_config())
    target = world.events.events[10]
    with pytest.raises(ValueError):
        run_counterfactual(
            world,
            (target.id, RemoveEventMutation(event_id="ffffffffffffffff")),
        )


def test_counterfactual_divergence_metrics_present() -> None:
    """`divergence_metrics` carries `n_diverged`, `pct_diverged`,
    `n_orphans`."""
    world = generate_world(_config())
    target = world.events.events[10]
    intervention: Intervention = (
        target.id,
        RemoveEventMutation(event_id=target.id),
    )
    run = run_counterfactual(world, intervention)
    assert "n_diverged" in run.divergence_metrics
    assert "pct_diverged" in run.divergence_metrics
    assert "n_orphans" in run.divergence_metrics


def test_counterfactual_validate_run_clean() -> None:
    """`validate_counterfactual_run` returns no violations for a
    freshly computed run."""
    world = generate_world(_config())
    target = world.events.events[10]
    intervention: Intervention = (
        target.id,
        RemoveEventMutation(event_id=target.id),
    )
    run = run_counterfactual(world, intervention)
    violations = validate_counterfactual_run(run)
    assert not violations, f"unexpected violations: {[v.code for v in violations]}"


def test_counterfactual_algorithm_version_pinned() -> None:
    """Algorithm version is exposed via
    `COUNTERFACTUAL_ALGORITHM_VERSION` and matches the run's
    `algorithm_version` after a fresh computation."""
    world = generate_world(_config())
    target = world.events.events[10]
    intervention: Intervention = (
        target.id,
        RemoveEventMutation(event_id=target.id),
    )
    run = run_counterfactual(world, intervention)
    from world_factory.counterfactual import _compute_algorithm_version
    assert run.algorithm_version == _compute_algorithm_version(
        run.alternate_events
    )
    assert run.algorithm_version.startswith("counter") or len(
        run.algorithm_version
    ) == 16, (
        f"counterfactual algorithm_version {run.algorithm_version!r} "
        f"not a 16-char blake2b hex"
    )


def test_counterfactual_provenance_recorded() -> None:
    """Provenance record describes the counterfactual operation."""
    record = counterfactual_provenance()
    assert record.output_path == "counterfactual_run"
    assert record.algorithm_version == COUNTERFACTUAL_ALGORITHM_VERSION


def test_counterfactual_base_world_id_set() -> None:
    """`base_world_id` is the canonical world's id (for
    traceability)."""
    world = generate_world(_config())
    target = world.events.events[10]
    intervention: Intervention = (
        target.id,
        RemoveEventMutation(event_id=target.id),
    )
    run = run_counterfactual(world, intervention)
    assert run.base_world_id == world.metadata.world_id