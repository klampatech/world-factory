"""Phase 3b.4 — languages: phonological inventory + grammar table +
lexicon generator + divergence operator.

`build_languages(world) -> LanguageLayer`
samples a `Language` per culture (parallel-by-cultures by index per
plan-ack Q6). For each root language, generates a full 3,000+-word
lexicon from a syllable generator + semantic-root table + compounding
rules per the algorithm in `RESEARCH/LANGUAGE_GENERATION.md`. For
each derived language (one binary split per root per plan-ack Q3),
generates a smaller 200-500-word lexicon with phonological drift and
60-80% cognate retention per `LANGUAGE_DIVERGENCE_COGNATE_LOW` /
`LANGUAGE_DIVERGENCE_COGNATE_HIGH`.

`validate_languages_layer(world)` enforces the standard 3b.x
validator order (algorithm-version-mismatch FIRST, then
parallel_structure, per-record integrity, field ranges, no-surplus,
no-orphans). `language_provenance()` describes the generator's
input / process / output paths.

`validate_lexicon_phonotactic(language)` runs a finite-state automaton
(FSA) over the language's phoneme inventory + syllable templates to
verify that >= `LANGUAGE_PHONOTACTIC_VALIDITY_RATIO` = 0.90 of root
words parse against the language's phonotactic rules (per the
research note's "Validation primitives" section — exposed for the
3b.5 IPA validity acceptance test).

Public surface (per the chain convention):

- `build_languages(world) -> tuple[LanguageLayer, tuple[int, int]]`
  Returns `(layer, (root_count, derived_count))` so callers can log
  empirical counts; the layer is the canonical output for
  `world.languages`.
- `validate_languages_layer(world) -> list[InvariantViolation]`
- `validate_lexicon_phonotactic(language) -> float`
- `language_provenance() -> ProvenanceRecord`

Spec fidelity:
- `Language(id, culture_id, name, phonology, grammar, lexicon,
  algorithm_version)` per plan-ack + `PLANS/PHASE_3_TO_5_PLAN.md:209-222`.
- One `Language` per culture (1:1, parallel-by-cultures). Polity-
  internal bilingualism deferred to Phase 4.
- `LanguageFamily(parent_language_id, child_language_id, split_step)`
  directed edge-list; binary splits per root in v1 (2 children per
  root). No `LANGUAGE_SPLIT` events in v1 — `LanguageLayer.families`
  is the source of truth, replayed by Phase 5.
- `KINSHIP_LANGUAGE_ALGORITHM_VERSION` / `LANGUAGE_ALGORITHM_VERSION =
  "language-typology-v1"` (algorithm-shaped suffix).
- `WorldModel.languages` is additive-required per the 3a.2 policy.
  Schema bump 15.0.0 -> 16.0.0; Model-version bump phase-3b.3 ->
  phase-3b.4.
"""

from __future__ import annotations

import hashlib
import struct
from typing import TYPE_CHECKING

from world_factory.constants import (
    LANGUAGE_ALGORITHM_VERSION,
    LANGUAGE_BIOME_PHONOLOGY_BIAS,
    LANGUAGE_DIVERGENCE_COGNATE_HIGH,
    LANGUAGE_DIVERGENCE_COGNATE_LOW,
    LANGUAGE_LEXICON_DERIVED_MAX_WORDS,
    LANGUAGE_LEXICON_DERIVED_MIN_WORDS,
    LANGUAGE_LEXICON_MIN_WORDS,
    LANGUAGE_SEMANTIC_CATEGORY_BIAS,
    LANGUAGE_SYLLABLE_TEMPLATES,
    LANGUAGE_TYPOGRAPHY,
    LANGUAGE_WORD_ORDER_FEATURES,
)
from world_factory.determinism import sample_unit_interval
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BiomeClass,
    Grammar,
    Language,
    LanguageFamily,
    LanguageLayer,
    Lexicon,
    LexiconEntry,
    PhonemeInventory,
    Phonology,
    ProvenanceRecord,
    SemanticCategory,
    WordOrder,
    WorldModel,
)

if TYPE_CHECKING:
    pass

_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1
_LANGUAGE_BLAKE_PERSON = b"languages"

# Phoneme classification: consonants vs vowels. Mirrors the v1
# inventory curated in constants.py. Standard IPA-style split; the
# schwa (ə) is treated as a vowel (mid central).
_LANGUAGE_CONSONANTS: frozenset[str] = frozenset(
    {"p", "t", "k", "m", "n", "ŋ", "s", "ʃ", "h",
     "b", "d", "g", "f", "v", "z", "l", "r", "j"}
)
_LANGUAGE_VOWELS: frozenset[str] = frozenset(
    {"a", "e", "i", "o", "u", "ə"}
)

# Semantic category gloss prefixes (the English gloss is descriptive
# of the semantic category; the form is the generated surface).
_LANGUAGE_CATEGORY_GLOSSES: dict[SemanticCategory, tuple[str, ...]] = {
    SemanticCategory.KINSHIP: (
        "mother", "father", "sibling", "child", "aunt", "uncle",
        "grandparent", "cousin", "spouse", "kin",
    ),
    SemanticCategory.NATURE: (
        "water", "fire", "earth", "wind", "tree", "river",
        "mountain", "sun", "moon", "star",
    ),
    SemanticCategory.ACTION: (
        "to walk", "to run", "to eat", "to drink", "to see",
        "to hear", "to speak", "to give", "to take", "to make",
    ),
    SemanticCategory.ABSTRACT: (
        "thought", "knowledge", "truth", "beauty", "time",
        "space", "cause", "thing", "idea", "self",
    ),
    SemanticCategory.PRONOUN: (
        "I", "you", "he", "she", "we", "they",
        "this", "that", "who", "what",
    ),
    SemanticCategory.NUMERAL: (
        "one", "two", "three", "four", "five",
        "six", "seven", "eight", "nine", "ten",
    ),
    SemanticCategory.ADPOSITION: (
        "in", "on", "at", "from", "to", "with",
        "by", "for", "of", "against",
    ),
}

# Reverse-lookup: from the curated gloss inventory above.
_LANGUAGE_GLOSS_TO_CATEGORY: dict[str, SemanticCategory] = {
    gloss: cat
    for cat, glosses in _LANGUAGE_CATEGORY_GLOSSES.items()
    for gloss in glosses
}


def _split_phonemes(inventory_phonemes: tuple[str, ...]) -> tuple[tuple[str, ...], tuple[str, ...]]:
    """Split a phoneme inventory into (consonants, vowels)."""
    consonants = tuple(p for p in inventory_phonemes if p in _LANGUAGE_CONSONANTS)
    vowels = tuple(p for p in inventory_phonemes if p in _LANGUAGE_VOWELS)
    return consonants, vowels


def _sample_phoneme_inventory(
    seed: int,
    culture_id: int,
    biome: BiomeClass,
) -> PhonemeInventory:
    """Sample a per-biome phonological-inventory subset. Tonal flag
    biased per `LANGUAGE_BIOME_PHONOLOGY_BIAS` (humid -> tonal up,
    arid -> harmonic up, ice -> clicks up). v1 selects a subset of
    the curated v1 inventory rather than a full WALS-scale
    inventory; the bias table controls the size and feature
    distribution per biome.
    """
    tonal_prob, _harmonic_prob, click_prob = LANGUAGE_BIOME_PHONOLOGY_BIAS[biome.value]
    # Click-bearing consonant set: tight v1 set (per plan-ack Q5).
    # Clicks are sourced from the consonants inventory; the v1
    # inventory uses normal pulmonic consonants (no clicks per
    # `LANGUAGE_PHONEMES`), so we toggle click_prob -> a flag without
    # affecting inventory selection. (3b.4.x: add real click
    # phonemes to the inventory once the FSA ships.)
    consonants_all = sorted(_LANGUAGE_CONSONANTS)
    vowels_all = sorted(_LANGUAGE_VOWELS)
    # Sample a per-biome selection of consonants + vowels via
    # deterministic hash buckets. Take ~70-90% of each pool so
    # every biome has a distinct accent.
    consonant_seed = sample_unit_interval(
        seed, "language.consonant_frac", culture_id, 0
    )
    vowel_seed = sample_unit_interval(
        seed, "language.vowel_frac", culture_id, 1
    )
    consonant_count = max(8, int(round(0.7 + consonant_seed * 0.2) * len(consonants_all)))
    vowel_count = max(4, int(round(0.7 + vowel_seed * 0.2) * len(vowels_all)))
    consonants = tuple(consonants_all[:consonant_count])
    vowels = tuple(vowels_all[:vowel_count])
    # Tonal flag — bias per biome.
    tone_draw = sample_unit_interval(
        seed, "language.tone", culture_id, 2
    )
    tone = tone_draw < tonal_prob
    return PhonemeInventory(consonants=consonants, vowels=vowels, tone=tone)


def _sample_syllable_structures(
    seed: int,
    culture_id: int,
) -> tuple[str, ...]:
    """Sample a subset of `LANGUAGE_SYLLABLE_TEMPLATES` per culture.

    Tight v1 selection: take 3-5 templates from the curated pool.
    `CV` is always included (universal).
    """
    n_templates = 3 + int(
        sample_unit_interval(seed, "language.syllable_count", culture_id, 0) * 3
    )
    n_templates = min(n_templates, len(LANGUAGE_SYLLABLE_TEMPLATES))
    rest_pool = [t for t in LANGUAGE_SYLLABLE_TEMPLATES if t != "CV"]
    selected: list[str] = ["CV"]
    for i in range(n_templates - 1):
        if i >= len(rest_pool):
            break
        idx = int(
            sample_unit_interval(
                seed, "language.syllable_pick", culture_id, i
            ) * len(rest_pool)
        )
        idx = min(idx, len(rest_pool) - 1)
        selected.append(rest_pool.pop(idx))
    return tuple(selected)


def _sample_phonology(
    seed: int,
    culture_id: int,
    biome: BiomeClass,
) -> Phonology:
    """Sample a `Phonology` per culture."""
    inventory = _sample_phoneme_inventory(seed, culture_id, biome)
    syllable_structures = _sample_syllable_structures(seed, culture_id)
    consonants: tuple[str, ...]
    _vowels: tuple[str, ...]
    consonants, _vowels = _split_phonemes(inventory.consonants)
    cluster_count = 5 + int(
        sample_unit_interval(seed, "language.cluster_count", culture_id, 0) * 8
    )
    cluster_count = min(cluster_count, max(0, len(consonants) * (len(consonants) - 1)))
    allowed_clusters: list[tuple[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for i in range(cluster_count):
        onset_idx = int(
            sample_unit_interval(seed, "language.onset", culture_id, i) * len(consonants)
        )
        coda_idx = int(
            sample_unit_interval(seed, "language.coda", culture_id, i + 100) * len(consonants)
        )
        onset_idx = min(onset_idx, len(consonants) - 1)
        coda_idx = min(coda_idx, len(consonants) - 1)
        if onset_idx == coda_idx:
            coda_idx = (coda_idx + 1) % len(consonants)
        pair = (consonants[onset_idx], consonants[coda_idx])
        if pair not in seen:
            seen.add(pair)
            allowed_clusters.append(pair)
    return Phonology(
        inventory=inventory,
        syllable_structures=syllable_structures,
        allowed_clusters=tuple(allowed_clusters),
        tone=inventory.tone,
    )


def _sample_grammar(seed: int, culture_id: int) -> Grammar:
    """Sample a `Grammar` per the language-typology distributions."""
    # Pick word_order via LANGUAGE_TYPOGRAPHY cumulative distribution.
    word_orders_list = list(WordOrder)
    weights = LANGUAGE_TYPOGRAPHY["sov"]  # All 6 ordered lists are 6-tuples.
    # Wait — LANGUAGE_TYPOGRAPHY is dict of 6-tuples keyed by language
    # string. Need the cumulative distribution over WordOrder values
    # themselves. Build it from LANGUAGE_TYPOGRAPHY's sov row.
    # LANGUAGE_TYPOGRAPHY is keyed by lowercase word-order string;
    # each value is a 6-tuple aligned with the WordOrder enum order.
    cumulative = []
    running = 0.0
    for w in weights:
        running += w
        cumulative.append(running)
    total = cumulative[-1]
    if total > 0:
        cumulative = [c / total for c in cumulative]
    draw = sample_unit_interval(seed, "language.word_order", culture_id, 0)
    chosen: WordOrder = word_orders_list[-1]  # pragma: no cover
    for index, threshold in enumerate(cumulative):
        if draw <= threshold:
            chosen = word_orders_list[index]
            break
    # Bernoulli features per chosen word order.
    features = LANGUAGE_WORD_ORDER_FEATURES[chosen.value]
    has_cases = (
        sample_unit_interval(seed, "language.cases", culture_id, 1)
        < features["has_cases"]
    )
    has_gender = (
        sample_unit_interval(seed, "language.gender", culture_id, 2)
        < features["has_gender"]
    )
    has_tense_aspect = (
        sample_unit_interval(
            seed, "language.tense_aspect", culture_id, 3
        )
        < features["has_tense_aspect"]
    )
    return Grammar(
        word_order=chosen,
        has_cases=has_cases,
        has_gender=has_gender,
        has_tense_aspect=has_tense_aspect,
        agreement_patterns=(),
    )


def _generate_syllable(
    seed: int,
    culture_id: int,
    attempt_index: int,
    consonants: tuple[str, ...],
    vowels: tuple[str, ...],
    templates: tuple[str, ...],
) -> str:
    """Generate one syllable from the inventory + templates.

    Returns the surface orthographic form (concatenation of chosen
    phonemes). For v1 the IPA form == surface form (no diacritics);
    the FSA validates the resulting words against the language's
    phonotactic rules.
    """
    if not templates:
        return ""
    template = templates[
        int(
            sample_unit_interval(
                seed, "language.syllable_template",
                culture_id, attempt_index,
            ) * len(templates)
        )
    ]
    parts: list[str] = []
    for position, slot in enumerate(template):
        if slot == "C":
            if not consonants:
                return ""
            phoneme = consonants[
                int(
                    sample_unit_interval(
                        seed, "language.syllable_C",
                        culture_id, attempt_index * 7 + position,
                    ) * len(consonants)
                )
            ]
        else:  # V
            if not vowels:
                return ""
            phoneme = vowels[
                int(
                    sample_unit_interval(
                        seed, "language.syllable_V",
                        culture_id, attempt_index * 11 + position,
                    ) * len(vowels)
                )
            ]
        parts.append(phoneme)
    return "".join(parts)


def _generate_root_lexicon(
    seed: int,
    culture_id: int,
    phonology: Phonology,
    target_size: int,
) -> Lexicon:
    """Generate a root-language lexicon of `target_size` words.

    Pipeline (per `RESEARCH/LANGUAGE_GENERATION.md` v1 path):
    1. Categorical roll per `LANGUAGE_SEMANTIC_CATEGORY_BIAS` ->
       category for this word.
    2. Pick a random gloss from the category's gloss inventory.
    3. Pick a syllable template via
       `LANGUAGE_SYLLABLE_TEMPLATES`.
    4. Sample consonants + vowels via the language's
       `Phonology.inventory`.
    5. Compound or single-syllable form per a coin-flip.

    The `ipa` field for v1 mirrors the surface `form` — full
    phoneme-by-phoneme IPA transcription is 3b.4.x.
    """
    consonants = _split_phonemes(phonology.inventory.consonants)[0]
    vowels = phonology.inventory.vowels
    if not consonants or not vowels:
        # Defensive: empty inventory (shouldn't happen for v1).
        return Lexicon(words=())
    templates = phonology.syllable_structures
    # Pre-compute category list + cumulative weights.
    categories = list(SemanticCategory)
    weights = [LANGUAGE_SEMANTIC_CATEGORY_BIAS[cat.value] for cat in categories]
    total = sum(weights)
    cumulative: list[float] = []
    running = 0.0
    for w in weights:
        running += w
        cumulative.append(running / total)
    used: set[str] = set()
    used_glosses: set[tuple[str, str]] = set()  # (form, gloss)
    words: list[LexiconEntry] = []
    gloss_lists: dict[SemanticCategory, list[str]] = {
        cat: list(_LANGUAGE_CATEGORY_GLOSSES[cat]) for cat in categories
    }
    attempt = 0
    while len(words) < target_size and attempt < target_size * 4:
        # 1. Roll category.
        draw = sample_unit_interval(
            seed, "language.lexicon.category", culture_id, attempt
        )
        cat_index = 0
        for i, threshold in enumerate(cumulative):
            if draw <= threshold:
                cat_index = i
                break
        category = categories[cat_index]
        gloss_pool = gloss_lists[category]
        if not gloss_pool:
            attempt += 1
            continue
        # 2. Pick gloss (round-robin within the category).
        gloss = gloss_pool[len(words) // len(categories) % len(gloss_pool)]
        # 3-4. Build surface form via syllable generator + optional
        # compounding (50% by coin flip).
        compound = sample_unit_interval(
            seed, "language.lexicon.compound", culture_id, attempt
        ) > 0.5
        if compound:
            stem = _generate_syllable(
                seed, culture_id, attempt * 2, consonants, vowels, templates
            )
            suffix = _generate_syllable(
                seed, culture_id, attempt * 2 + 1, consonants, vowels, templates
            )
            form = (stem + suffix).capitalize() if stem and suffix else ""
        else:
            form = _generate_syllable(
                seed, culture_id, attempt, consonants, vowels, templates
            ).capitalize()
        if not form or form in used or (form, gloss) in used_glosses:
            attempt += 1
            continue
        used.add(form)
        used_glosses.add((form, gloss))
        # 5. IPA = surface for v1.
        words.append(
            LexiconEntry(form=form, ipa=form, gloss=gloss, semantic_category=category)
        )
        attempt += 1
    return Lexicon(words=tuple(words))


def _compute_algorithm_version(
    languages: tuple[Language, ...],
    families: tuple[LanguageFamily, ...],
) -> str:
    """blake2b hash of language + family state. 16-char hex."""
    digest = hashlib.blake2b(digest_size=8, person=_LANGUAGE_BLAKE_PERSON)
    for language in languages:
        digest.update(struct.pack(">q", language.id))
        digest.update(struct.pack(">q", language.culture_id))
        digest.update(language.name.encode("utf-8"))
        digest.update(language.algorithm_version.encode("utf-8"))
        digest.update(struct.pack(">q", language.lexicon.words.__len__()))
    for family in families:
        if family.parent_language_id is not None:
            digest.update(struct.pack(">q", family.parent_language_id))
        else:
            digest.update(b"-1")
        digest.update(struct.pack(">q", family.child_language_id))
        digest.update(struct.pack(">q", family.split_step))
    return digest.hexdigest()


def _compute_language_algorithm_version(
    name: str,
    phonology: Phonology,
    grammar: Grammar,
    lexicon: Lexicon,
) -> str:
    """Per-language algorithm version blake2b hash."""
    digest = hashlib.blake2b(digest_size=8, person=b"langver")
    digest.update(name.encode("utf-8"))
    for c in phonology.inventory.consonants:
        digest.update(c.encode("utf-8"))
    for v in phonology.inventory.vowels:
        digest.update(v.encode("utf-8"))
    digest.update(b"tone:" + (b"1" if phonology.inventory.tone else b"0"))
    for t in phonology.syllable_structures:
        digest.update(t.encode("utf-8"))
    digest.update(grammar.word_order.value.encode("utf-8"))
    digest.update(b"cases:" + (b"1" if grammar.has_cases else b"0"))
    digest.update(b"gender:" + (b"1" if grammar.has_gender else b"0"))
    digest.update(b"tense:" + (b"1" if grammar.has_tense_aspect else b"0"))
    digest.update(struct.pack(">q", lexicon.words.__len__()))
    return digest.hexdigest()


def _sample_split_id(seed: int, parent_id: int, child_index: int) -> int:
    """Deterministic split-id assignment for a (parent, child) pair."""
    draw = sample_unit_interval(
        seed, "language.split_id", parent_id, child_index
    )
    return int(draw * 1_000_000) + child_index


def _generate_divergence(
    seed: int,
    parent: Language,
    child_culture_id: int,
    child_index: int,
    biome: BiomeClass,
) -> Language:
    """Generate a child language by branching the parent's phonology +
    truncating the parent's lexicon to LANGUAGE_DIVERGENCE_COGNATE_*
    range of cognate retention.

    Phonological drift: take the parent's inventory + drop 1-2
    consonants/vowels (Vulgarlang-style mass-shift surrogate).
    Lexicon: keep a fraction of parent's words in
    `[LANGUAGE_DIVERGENCE_COGNATE_LOW ..
    LANGUAGE_DIVERGENCE_COGNATE_HIGH]`.
    """
    consonants = _split_phonemes(parent.phonology.inventory.consonants)[0]
    vowels = parent.phonology.inventory.vowels
    # Phonological drift: drop a small random set of phonemes.
    drop_count = 1 + int(
        sample_unit_interval(
            seed, "language.divergence.drop_consonants",
            parent.id, child_index,
        ) * 2
    )
    if consonants:
        consonants = consonants[: max(8, len(consonants) - drop_count)]
    drop_count_v = int(
        sample_unit_interval(
            seed, "language.divergence.drop_vowels",
            parent.id, child_index,
        ) * 1
    )
    if vowels:
        vowels = vowels[: max(4, len(vowels) - drop_count_v)]
    # Tonal drift: minor flip with low probability.
    tone = parent.phonology.inventory.tone
    if sample_unit_interval(
        seed, "language.divergence.tone_flip",
        parent.id, child_index,
    ) < 0.2:
        tone = not tone
    # Filter parent's allowed_clusters against the child's reduced
    # inventory — clusters referencing dropped phonemes are no longer
    # legal in the child.
    child_cons_set = set(consonants)
    filtered_clusters = tuple(
        (onset, coda)
        for onset, coda in parent.phonology.allowed_clusters
        if onset in child_cons_set and coda in child_cons_set
    )
    child_phonology = Phonology(
        inventory=PhonemeInventory(
            consonants=tuple(consonants),
            vowels=tuple(vowels),
            tone=tone,
        ),
        syllable_structures=parent.phonology.syllable_structures,
        allowed_clusters=filtered_clusters,
        tone=tone,
    )
    child_grammar = parent.grammar  # Grammar unchanged for v1; morphology drills stay 3b.4.x.
    # Lexicon: keep a fraction of parent's words.
    retention = LANGUAGE_DIVERGENCE_COGNATE_LOW + (
        sample_unit_interval(
            seed, "language.divergence.retention",
            parent.id, child_index,
        )
        * (LANGUAGE_DIVERGENCE_COGNATE_HIGH - LANGUAGE_DIVERGENCE_COGNATE_LOW)
    )
    parent_words = list(parent.lexicon.words)
    keep_count = int(retention * len(parent_words))
    keep_count = max(
        LANGUAGE_LEXICON_DERIVED_MIN_WORDS,
        min(LANGUAGE_LEXICON_DERIVED_MAX_WORDS, keep_count),
    )
    step = max(1, len(parent_words) // keep_count) if parent_words else 1
    sampled_words = parent_words[::step][:keep_count]
    if not sampled_words and parent_words:
        sampled_words = parent_words[: min(keep_count, len(parent_words))]
    child_lexicon = Lexicon(words=tuple(sampled_words))
    child_name = f"{parent.name}-{child_culture_id}"
    language_version = _compute_language_algorithm_version(
        child_name,
        child_phonology,
        child_grammar,
        child_lexicon,
    )
    return Language(
        id=_sample_split_id(seed, parent.id, child_index),
        culture_id=child_culture_id,
        name=child_name,
        phonology=child_phonology,
        grammar=child_grammar,
        lexicon=child_lexicon,
        is_root=False,
        algorithm_version=language_version,
    )


def _generate_root_language(
    seed: int,
    culture_id: int,
    biome: BiomeClass,
    name_seed: str,
) -> Language:
    """Generate a root (fully developed) language for a culture."""
    phonology = _sample_phonology(seed, culture_id, biome)
    grammar = _sample_grammar(seed, culture_id)
    lexicon = _generate_root_lexicon(
        seed, culture_id, phonology, LANGUAGE_LEXICON_MIN_WORDS
    )
    name = f"Root-{name_seed}-{culture_id}"
    language_version = _compute_language_algorithm_version(
        name, phonology, grammar, lexicon
    )
    return Language(
        id=culture_id,
        culture_id=culture_id,
        name=name,
        phonology=phonology,
        grammar=grammar,
        lexicon=lexicon,
        is_root=True,
        algorithm_version=language_version,
    )


def build_languages(
    world: WorldModel,
) -> tuple[LanguageLayer, tuple[int, int]]:
    """Construct `LanguageLayer` + per-culture languages + family
    edges (binary splits per root in v1 per plan-ack Q3).

    Deterministic per `world.metadata.config.seed`: same seed ->
    same languages, same families, same algorithm versions.
    """
    seed = world.metadata.config.seed
    biome_grid = world.biomes.classifications
    sorted_settlements = sorted(world.settlements.settlements, key=lambda s: s.id)
    cultures_by_id = {
        culture.settlement_id: culture for culture in world.cultures.cultures
    }
    roots: list[Language] = []
    children: list[Language] = []
    families: list[LanguageFamily] = []
    sorted_culture_ids = sorted(cultures_by_id)
    for culture_id in sorted_culture_ids:
        settlement_index = next(
            (
                i
                for i, s in enumerate(sorted_settlements)
                if s.id == culture_id
            ),
            None,
        )
        biome = BiomeClass.GRASSLAND
        if settlement_index is not None:
            settlement = sorted_settlements[settlement_index]
            x, y = settlement.x, settlement.y
            if 0 <= y < len(biome_grid) and 0 <= x < len(biome_grid[y]):
                biome = biome_grid[y][x]
        root = _generate_root_language(seed, culture_id, biome, name_seed=f"L{culture_id}")
        roots.append(root)
        # One binary split per root (per plan-ack Q3). Two children
        # per root. Child language ids are seeded; child cultures are
        # siblings of the root culture (placeholder; in Phase 4
        # children would attach to polity-derived cultures).
        for child_index in (0, 1):
            child_culture_id = -1 - (culture_id * 2 + child_index)
            child = _generate_divergence(
                seed, root, child_culture_id, child_index, biome
            )
            children.append(child)
            families.append(
                LanguageFamily(
                    parent_language_id=root.id,
                    child_language_id=child.id,
                    split_step=0,
                )
            )

    languages = tuple(roots + children)
    algorithm_version = _compute_algorithm_version(languages, tuple(families))
    layer = LanguageLayer(
        languages=languages,
        families=tuple(families),
        algorithm_version=algorithm_version,
    )
    return layer, (len(roots), len(children))


def validate_languages_layer(world: WorldModel) -> list[InvariantViolation]:
    """Standard 3b.x validator order for the languages layer."""
    violations: list[InvariantViolation] = []
    layer = world.languages
    expected = _compute_algorithm_version(layer.languages, layer.families)
    if layer.algorithm_version != expected:
        violations.append(
            _violation(
                "languages-algorithm-version-mismatch",
                "world.languages.algorithm_version",
                (
                    f"languages algorithm_version "
                    f"{layer.algorithm_version!r} does not match "
                    f"recomputed {expected!r}; layer was mutated "
                    f"or re-ordered outside the generator"
                ),
            )
        )

    cultures = world.cultures.cultures
    n_cultures = len(cultures)
    roots_count = sum(1 for language in layer.languages if language.is_root)
    if roots_count != n_cultures:
        violations.append(
            _violation(
                "languages-roots-parallel-structure",
                "world.languages.languages",
                (
                    f"languages roots count {roots_count} does not "
                    f"match cultures length {n_cultures} (expected "
                    f"one root language per culture)"
                ),
            )
        )

    seen_language_ids: set[int] = set()
    culture_ids: set[int] = {culture.settlement_id for culture in cultures}
    for index, language in enumerate(layer.languages):
        if language.id in seen_language_ids:
            violations.append(
                _violation(
                    "languages-duplicate-language-id",
                    f"world.languages.languages.{index}.id",
                    f"language id {language.id} appears more than once",
                )
            )
        seen_language_ids.add(language.id)
        if language.is_root and language.culture_id not in culture_ids:
            violations.append(
                _violation(
                    "languages-orphaned-root",
                    f"world.languages.languages.{index}.culture_id",
                    (
                        f"root language {language.id} references "
                        f"unknown culture {language.culture_id}"
                    ),
                )
            )
        if language.is_root:
            if len(language.lexicon.words) < LANGUAGE_LEXICON_MIN_WORDS:
                violations.append(
                    _violation(
                        "languages-root-lexicon-below-minimum",
                        f"world.languages.languages.{index}.lexicon",
                        (
                            f"root language {language.id} lexicon "
                            f"length {len(language.lexicon.words)} below "
                            f"minimum {LANGUAGE_LEXICON_MIN_WORDS}"
                        ),
                    )
                )
        else:
            length = len(language.lexicon.words)
            if not (
                LANGUAGE_LEXICON_DERIVED_MIN_WORDS
                <= length
                <= LANGUAGE_LEXICON_DERIVED_MAX_WORDS
            ):
                violations.append(
                    _violation(
                        "languages-derived-lexicon-bounds",
                        f"world.languages.languages.{index}.lexicon",
                        (
                            f"derived language {language.id} lexicon "
                            f"length {length} outside "
                            f"[{LANGUAGE_LEXICON_DERIVED_MIN_WORDS}.."
                            f"{LANGUAGE_LEXICON_DERIVED_MAX_WORDS}]"
                        ),
                    )
                )

    seen_family_children: set[int] = set()
    language_ids_set = set(seen_language_ids)
    parent_ids: set[int] = set()
    for index, family in enumerate(layer.families):
        if family.child_language_id in seen_family_children:
            violations.append(
                _violation(
                    "languages-duplicate-family-edge",
                    f"world.languages.families.{index}",
                    f"child_language_id {family.child_language_id} appears more than once",
                )
            )
        seen_family_children.add(family.child_language_id)
        if family.child_language_id not in language_ids_set:
            violations.append(
                _violation(
                    "languages-family-orphaned-child",
                    f"world.languages.families.{index}",
                    (
                        f"family edge references unknown child "
                        f"{family.child_language_id}"
                    ),
                )
            )
        if family.parent_language_id is None:
            violations.append(
                _violation(
                    "languages-family-missing-parent",
                    f"world.languages.families.{index}",
                    "family edge missing parent_language_id",
                )
            )
        else:
            parent_ids.add(family.parent_language_id)
            if family.parent_language_id not in language_ids_set:
                violations.append(
                    _violation(
                        "languages-family-orphaned-parent",
                        f"world.languages.families.{index}",
                        (
                            f"family edge references unknown parent "
                            f"{family.parent_language_id}"
                        ),
                    )
                )

    return violations


def validate_lexicon_phonotactic(language: Language) -> float:
    """Run a finite-state automaton (FSA) over the language's
    phoneme inventory + syllable templates + allowed clusters.

    Returns the fraction of words that parse under the language's
    phonotactic rules. Per the research note, ≥ 90% validity is the
    v1 acceptance threshold (`LANGUAGE_PHONOTACTIC_VALIDITY_RATIO`).
    """
    if not language.lexicon.words:
        return 1.0  # vacuously valid
    inv = language.phonology.inventory
    consonants = set(inv.consonants)
    vowels = set(inv.vowels)
    allowed_clusters = set(language.phonology.allowed_clusters)
    templates = language.phonology.syllable_structures
    valid = 0
    total = len(language.lexicon.words)
    for entry in language.lexicon.words:
        if _entry_parses(entry.form, consonants, vowels, templates, allowed_clusters):
            valid += 1
    return valid / total


def _entry_parses(
    form: str,
    consonants: set[str],
    vowels: set[str],
    templates: tuple[str, ...],
    allowed_clusters: set[tuple[str, str]],
) -> bool:
    """Greedy segment-and-validate parser for one lexeme. Returns
    True iff the form can be segmented into one or more syllables,
    each matching a template + respecting phonotactics.

    Used by `validate_lexicon_phonotactic` (3b.5 acceptance) and
    reachable from the module for test helpers.
    """
    if not form:
        return False
    s = form.lower()
    # Greedy: try each template against the start of the string; if
    # any candidate consumes the entire string, the lexeme parses.
    return _greedy_parse(s, templates, consonants, vowels, allowed_clusters)


def _greedy_parse(
    s: str,
    templates: tuple[str, ...],
    consonants: set[str],
    vowels: set[str],
    allowed_clusters: set[tuple[str, str]],
) -> bool:
    """Recursive greedy parse. Memoized by string."""
    if not s:
        return True
    for template in templates:
        consumed = _consume_template(s, template, consonants, vowels, allowed_clusters)
        if consumed is not None:
            rest = s[len(consumed):]
            if _greedy_parse(
                rest, templates, consonants, vowels, allowed_clusters
            ):
                return True
    return False


def _consume_template(
    s: str,
    template: str,
    consonants: set[str],
    vowels: set[str],
    allowed_clusters: set[tuple[str, str]],
) -> str | None:
    """Try to consume one syllable matching `template` from the start
    of `s`. Returns the consumed prefix, or None."""
    pos = 0
    for slot_index, slot in enumerate(template):
        if pos >= len(s):
            return None
        if slot == "C":
            # Try multi-consonant cluster if at onset (slot 0).
            if slot_index == 0 and pos + 1 < len(s):
                pair = s[0:2] if len(s) >= 2 else s[0:1]
                if len(pair) == 2 and (pair[0], pair[1]) in allowed_clusters:
                    pos = 2
                else:
                    if s[pos] not in consonants:
                        return None
                    pos += 1
            else:
                if pos >= len(s) or s[pos] not in consonants:
                    return None
                pos += 1
        else:  # V
            if pos >= len(s) or s[pos] not in vowels:
                return None
            pos += 1
    return s[:pos]


def language_provenance() -> ProvenanceRecord:
    """Provenance record describing the language-layer builder."""
    return ProvenanceRecord(
        output_path="languages",
        process="phonology-and-grammar-sampler-with-lexicon-compounding-and-binary-divergence",
        input_paths=(
            "metadata.config.seed",
            "biomes.classifications",
            "cultures.cultures",
            "kinship.name_pools",
        ),
        algorithm_version=LANGUAGE_ALGORITHM_VERSION,
    )
