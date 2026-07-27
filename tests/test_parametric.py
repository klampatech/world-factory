"""Parametric composition: knobs take effect coherently and decouple where intended."""

from world_factory.generator import generate_world
from world_factory.models import ClimateClass, WorldConfig, WorldScale


def test_seed_changes_world_identity() -> None:
    assert (
        generate_world(WorldConfig(seed=1)).metadata.world_id
        != generate_world(WorldConfig(seed=2)).metadata.world_id
    )


def test_scale_changes_grid_dimensions() -> None:
    small = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    medium = generate_world(WorldConfig(seed=42, scale=WorldScale.MEDIUM))
    assert small.geography.width < medium.geography.width
    assert small.geography.height < medium.geography.height


def test_climate_class_changes_temperature_baseline() -> None:
    cold = generate_world(WorldConfig(seed=42, climate_class=ClimateClass.COLD))
    hot = generate_world(WorldConfig(seed=42, climate_class=ClimateClass.HOT))
    cold_mean = sum(sum(row) for row in cold.climate.temperature_celsius) / (
        cold.geography.width * cold.geography.height
    )
    hot_mean = sum(sum(row) for row in hot.climate.temperature_celsius) / (
        hot.geography.width * hot.geography.height
    )
    assert hot_mean > cold_mean


def test_sentience_and_magic_do_not_affect_phase_zero_outputs() -> None:
    baseline = generate_world(WorldConfig(seed=42, sentience_enabled=True, magic_enabled=False))
    sentience_off = generate_world(
        WorldConfig(seed=42, sentience_enabled=False, magic_enabled=False)
    )
    magic_on = generate_world(WorldConfig(seed=42, sentience_enabled=True, magic_enabled=True))
    assert baseline.geography.elevation_meters == sentience_off.geography.elevation_meters
    assert baseline.geography.elevation_meters == magic_on.geography.elevation_meters
    assert baseline.climate.temperature_celsius == sentience_off.climate.temperature_celsius
    assert baseline.biomes.classifications == sentience_off.biomes.classifications


def test_world_id_changes_with_config_change() -> None:
    base = WorldConfig(seed=42)
    assert (
        generate_world(base).metadata.world_id
        != generate_world(base.model_copy(update={"scale": WorldScale.MEDIUM})).metadata.world_id
    )


def test_parametric_cross_product_stays_plausible() -> None:
    """The Phase 0 parametric-composition smoke test: all knob combos yield valid worlds."""
    from world_factory.validation import validate_world

    for scale in WorldScale:
        for climate in ClimateClass:
            for sentience in (True, False):
                for magic in (True, False):
                    config = WorldConfig(
                        seed=42,
                        scale=scale,
                        climate_class=climate,
                        sentience_enabled=sentience,
                        magic_enabled=magic,
                    )
                    report = validate_world(generate_world(config))
                    assert report.is_valid, (config, report.model_dump(mode="json"))
