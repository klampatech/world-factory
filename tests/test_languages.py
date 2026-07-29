"""Phase 3b.4 language-layer invariants and determinism —
LanguageLayer on WorldModel, Language records parallel to cultures,
LanguageFamily edges with binary splits per root, phonotactic FSA
validation per `LANGUAGE_PHONOTACTIC_VALIDITY_RATIO`."""

from __future__ import annotations

from collections import Counter

from world_factory.constants import (
    LANGUAGE_ALGORITHM_VERSION,
    LANGUAGE_LEXICON_DERIVED_MAX_WORDS,
    LANGUAGE_LEXICON_DERIVED_MIN_WORDS,
    LANGUAGE_LEXICON_MIN_WORDS,
    LANGUAGE_PHONEMES,
    LANGUAGE_PHONOTACTIC_VALIDITY_RATIO,
    LANGUAGE_SYLLABLE_TEMPLATES,
)
from world_factory.generator import generate_world
from world_factory.language import (
    _compute_algorithm_version,
    _compute_language_algorithm_version,
    validate_languages_layer,
    validate_lexicon_phonotactic,
)
from world_factory.models import (
    Grammar,
    Language,
    LanguageLayer,
    Lexicon,
    PhonemeInventory,
    Phonology,
    SemanticCategory,
    WordOrder,
)
from world_factory.validation import validate_world


def _config(seed: int = 42, scale: object = None) -> object:
    from world_factory.models import WorldConfig, WorldScale
    return WorldConfig(seed=seed, scale=scale if scale is not None else WorldScale.LARGE)


def test_world_model_includes_languages_layer() -> None:
    """`WorldModel.languages` is a `LanguageLayer` aggregate."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    assert world.languages is not None
    assert isinstance(world.languages, LanguageLayer)


def test_roots_parallel_to_cultures() -> None:
    """One root language per culture (Q6 1:1 mapping)."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    roots = [lang for lang in world.languages.languages if lang.is_root]
    assert len(roots) == len(world.cultures.cultures), (
        f"roots {len(roots)} != cultures {len(world.cultures.cultures)}"
    )
    culture_ids = {culture.settlement_id for culture in world.cultures.cultures}
    for root in roots:
        assert root.culture_id in culture_ids, (
            f"root language {root.id} references missing culture {root.culture_id}"
        )


def test_per_culture_unique_root_id() -> None:
    """Each root language has a unique id parallel to its culture's id
    (simplest mapping)."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    root_ids_by_culture: dict[int, int] = {}
    for root in (lang for lang in world.languages.languages if lang.is_root):
        previous = root_ids_by_culture.get(root.culture_id)
        assert previous is None, (
            f"two roots for culture {root.culture_id}: {previous}, {root.id}"
        )
        root_ids_by_culture[root.culture_id] = root.id


def test_root_lexicon_size_meets_minimum() -> None:
    """Each root language has at least LANGUAGE_LEXICON_MIN_WORDS words."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    for root in (lang for lang in world.languages.languages if lang.is_root):
        assert len(root.lexicon.words) >= LANGUAGE_LEXICON_MIN_WORDS, (
            f"root {root.id} lexicon size {len(root.lexicon.words)} "
            f"< min {LANGUAGE_LEXICON_MIN_WORDS}"
        )


def test_root_lexicon_categorically_covered() -> None:
    """Each root language's lexicon covers all 7 SemanticCategory values
    (no zero-bucket categories)."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    for root in (lang for lang in world.languages.languages if lang.is_root):
        categories = {entry.semantic_category for entry in root.lexicon.words}
        assert categories == set(SemanticCategory), (
            f"root {root.id} missing categories: "
            f"{set(SemanticCategory) - categories}"
        )


def test_derived_lexicon_in_range() -> None:
    """Each derived language has lexicon size in
    [LANGUAGE_LEXICON_DERIVED_MIN_WORDS ..
    LANGUAGE_LEXICON_DERIVED_MAX_WORDS]."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    derived = [lang for lang in world.languages.languages if not lang.is_root]
    assert derived, "no derived languages at this seed/scale (expected binary splits per root)"
    for lang in derived:
        length = len(lang.lexicon.words)
        assert length >= LANGUAGE_LEXICON_DERIVED_MIN_WORDS
        assert length <= LANGUAGE_LEXICON_DERIVED_MAX_WORDS


def test_binary_split_per_root() -> None:
    """Per plan-ack Q3, each root produces exactly two derived children
    (binary splits)."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    children_per_root: Counter[int] = Counter()
    for family in world.languages.families:
        if family.parent_language_id is not None:
            children_per_root[family.parent_language_id] += 1
    for root in (lang for lang in world.languages.languages if lang.is_root):
        assert children_per_root[root.id] == 2, (
            f"root {root.id} has {children_per_root[root.id]} children; "
            f"expected 2 (binary split per plan-ack Q3)"
        )


def test_family_parental_coverage() -> None:
    """Every non-root language has a LanguageFamily edge referencing it as
    child; every family edge has parent + child both existing."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    language_ids = {lang.id for lang in world.languages.languages}
    family_children = {
        family.child_language_id for family in world.languages.families
    }
    for lang in world.languages.languages:
        if not lang.is_root:
            assert lang.id in family_children, (
                f"derived language {lang.id} has no LanguageFamily edge"
            )
    for family in world.languages.families:
        assert family.parent_language_id in language_ids, (
            f"family edge references unknown parent {family.parent_language_id}"
        )
        assert family.child_language_id in language_ids, (
            f"family edge references unknown child {family.child_language_id}"
        )


def test_family_split_step_zero() -> None:
    """v1 emits family splits at world-gen (step 0), not per-step."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    for family in world.languages.families:
        assert family.split_step == 0


def test_no_duplicate_family_child() -> None:
    """Each child appears in at most one `LanguageFamily` edge."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    children = [family.child_language_id for family in world.languages.families]
    assert len(children) == len(set(children)), (
        "duplicate child_language_id entries in LanguageFamily edges"
    )


def test_phonology_inventory_subset_of_language_phonemes() -> None:
    """All languages' phoneme inventories are subsets of the curated
    LANGUAGE_PHONEMES pool."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    allowed = set(LANGUAGE_PHONEMES)
    for lang in world.languages.languages:
        for phoneme in lang.phonology.inventory.consonants:
            assert phoneme in allowed, (
                f"language {lang.id} consonant {phoneme!r} not in LANGUAGE_PHONEMES"
            )
        for phoneme in lang.phonology.inventory.vowels:
            assert phoneme in allowed, (
                f"language {lang.id} vowel {phoneme!r} not in LANGUAGE_PHONEMES"
            )


def test_syllable_structures_subset_of_language_syllable_templates() -> None:
    """Each language's syllable structures are drawn from the
    LANGUAGE_SYLLABLE_TEMPLATES pool."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    allowed = set(LANGUAGE_SYLLABLE_TEMPLATES)
    for lang in world.languages.languages:
        for template in lang.phonology.syllable_structures:
            assert template in allowed, (
                f"language {lang.id} syllable template {template!r} not in pool"
            )


def test_grammar_word_order_is_str_enum() -> None:
    """Each language's `Grammar.word_order` is a valid `WordOrder` value."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    for lang in world.languages.languages:
        assert isinstance(lang.grammar.word_order, WordOrder)


def test_phonotactic_validity_passes_threshold() -> None:
    """Per plan-ack Q5 + the research note, >= 90% of root words parse
    under the language's phonotactic rules."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    for lang in (lang for lang in world.languages.languages if lang.is_root):
        ratio = validate_lexicon_phonotactic(lang)
        assert ratio >= LANGUAGE_PHONOTACTIC_VALIDITY_RATIO, (
            f"language {lang.id} phonotactic validity {ratio:.3f} "
            f"< threshold {LANGUAGE_PHONOTACTIC_VALIDITY_RATIO}"
        )


def test_phonotactic_returns_one_for_empty_lexicon() -> None:
    """`validate_lexicon_phonotactic` returns 1.0 for empty lexicons
    (vacuous truth; bound for derived languages in degenerate cases)."""
    language = Language(
        id=999,
        culture_id=999,
        name="empty",
        phonology=Phonology(
            inventory=PhonemeInventory(
                consonants=("p",), vowels=("a",), tone=False
            ),
            syllable_structures=("CV",),
            allowed_clusters=(),
            tone=False,
        ),
        grammar=Grammar(
            word_order=WordOrder.SVO,
            has_cases=False,
            has_gender=False,
            has_tense_aspect=False,
        ),
        lexicon=Lexicon(words=()),
        is_root=False,
        algorithm_version="deadbeef",
    )
    assert validate_lexicon_phonotactic(language) == 1.0


def test_algorithm_version_recomputed_matches_recorded() -> None:
    """The recorded `LanguageLayer.algorithm_version` matches a fresh
    blake2b of languages + families (algorithm-version-first invariant)."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    expected = _compute_algorithm_version(
        world.languages.languages, world.languages.families
    )
    assert world.languages.algorithm_version == expected


def test_per_language_algorithm_version_matches_recorded() -> None:
    """Each language's `algorithm_version` matches its phonology + grammar
    + lexicon hash."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    for lang in world.languages.languages:
        expected = _compute_language_algorithm_version(
            lang.name, lang.phonology, lang.grammar, lang.lexicon
        )
        assert lang.algorithm_version == expected, (
            f"language {lang.id} algorithm_version {lang.algorithm_version!r} "
            f"!= recomputed {expected!r}"
        )


def test_validator_catches_algorithm_version_mismatch() -> None:
    """Mutating `algorithm_version` triggers a violation."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    tampered = world.model_copy(
        update={
            "languages": world.languages.model_copy(
                update={"algorithm_version": "deadbeefdeadbeef"}
            )
        }
    )
    violations = validate_languages_layer(tampered)
    assert any(v.code == "languages-algorithm-version-mismatch" for v in violations)


def test_validator_catches_orphaned_root() -> None:
    """A root language referencing a missing culture triggers a violation."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    roots = [lang for lang in world.languages.languages if lang.is_root]
    bogus = roots[0].model_copy(update={"culture_id": -9999})
    bad_languages = (bogus,) + tuple(
        lang for lang in world.languages.languages if lang.id != bogus.id
    )
    bad_layer = world.languages.model_copy(
        update={
            "languages": bad_languages,
            "algorithm_version": _compute_algorithm_version(
                bad_languages, world.languages.families
            ),
        }
    )
    bad_world = world.model_copy(update={"languages": bad_layer})
    violations = validate_languages_layer(bad_world)
    assert any(v.code == "languages-orphaned-root" for v in violations)


def test_validator_catches_root_lexicon_below_minimum() -> None:
    """A root language with too-few lexicon words triggers a violation."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    roots = [lang for lang in world.languages.languages if lang.is_root]
    short_lex = Lexicon(words=roots[0].lexicon.words[:10])
    bogus = roots[0].model_copy(update={"lexicon": short_lex})
    bad_languages = (bogus,) + tuple(
        lang for lang in world.languages.languages if lang.id != bogus.id
    )
    bad_layer = world.languages.model_copy(
        update={
            "languages": bad_languages,
            "algorithm_version": _compute_algorithm_version(
                bad_languages, world.languages.families
            ),
        }
    )
    bad_world = world.model_copy(update={"languages": bad_layer})
    violations = validate_languages_layer(bad_world)
    assert any(v.code == "languages-root-lexicon-below-minimum" for v in violations)


def test_validator_catches_orphaned_family_parent() -> None:
    """A `LanguageFamily` referencing a missing parent_language_id
    triggers a violation."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    if not world.languages.families:
        return
    bogus_family = world.languages.families[0].model_copy(
        update={"parent_language_id": -9999}
    )
    bad_families = (bogus_family,) + world.languages.families[1:]
    bad_layer = world.languages.model_copy(
        update={
            "families": bad_families,
            "algorithm_version": _compute_algorithm_version(
                world.languages.languages, bad_families
            ),
        }
    )
    bad_world = world.model_copy(update={"languages": bad_layer})
    violations = validate_languages_layer(bad_world)
    assert any(v.code == "languages-family-orphaned-parent" for v in violations)


def test_validate_world_clean_at_seed_42() -> None:
    """End-to-end validation at LARGE seed=42 produces a clean report
    including the new languages validator."""
    from world_factory.models import WorldScale
    world = generate_world(_config(scale=WorldScale.LARGE))
    report = validate_world(world)
    assert report.is_valid, (
        f"validate_world reported violations: "
        f"{[(v.code, v.path, v.message) for v in report.violations]}"
    )


def test_world_id_stable_across_3b_4() -> None:
    """`world_id` for `--seed 42` is unchanged from the chain (no new
    WorldConfig fields)."""
    from world_factory.models import WorldScale
    world_a = generate_world(_config(seed=42, scale=WorldScale.LARGE))
    world_b = generate_world(_config(seed=42, scale=WorldScale.LARGE))
    assert world_a.metadata.world_id == world_b.metadata.world_id, (
        f"world_id drifted: {world_a.metadata.world_id} != {world_b.metadata.world_id}"
    )
    assert (
        world_a.languages.languages[0].lexicon.words[0].form
        == world_b.languages.languages[0].lexicon.words[0].form
    ), "language generation is non-deterministic across runs at the same seed"


def test_language_algorithm_version_constant() -> None:
    """`LANGUAGE_ALGORITHM_VERSION` carries an algorithm-shaped suffix,
    not a phase number (per chain convention)."""
    assert LANGUAGE_ALGORITHM_VERSION == "language-typology-v1"
    assert "-" in LANGUAGE_ALGORITHM_VERSION
    assert LANGUAGE_ALGORITHM_VERSION.endswith("-v1")
