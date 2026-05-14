"""ID generation for entity files.

Three configurable axes:

- format: ``word`` (adjective + noun) or ``uuid``
- word_style: ``pascal`` (PascalCase, e.g. ``CalmFox``), ``camel``
  (camelCase, e.g. ``calmFox``), or ``kebab`` (lowercase kebab,
  e.g. ``calm-fox``). Only applies to ``word`` format.
- slug: append a content-derived slug or not
- datetime_prefix: if True, prepend ``YYYYMMDD_HHMM`` before the word
  pair so the creation time is embedded in the ID.

The kind prefix (BACK, LOG, DEC, ...) is fixed by `processkit.KIND_PREFIXES`
and is not configurable.

For collision avoidance, callers pass an ``existing`` set of already-used
ID bodies (without prefix). If a generated body collides, the generator
appends a short hex suffix until unique.
"""
from __future__ import annotations

import random
import re
import uuid as _uuid
from collections import defaultdict
from datetime import datetime, timezone
from typing import Iterable, Sequence

from . import KIND_PREFIXES

# Positive, concrete words for memorable two-word IDs.
# Combinations: 120 * 120 = 14,400 base IDs per kind. Append a hex
# suffix on collision.
_ADJECTIVES = (
    "able active adept agile alert amber ample artful assured astute balanced"
    " bold brave bright brisk buoyant calm capable cheerful clear clever cool"
    " cordial crisp curious daring deft deep eager earnest fair faithful fast"
    " fine firm fit fluent focused fresh friendly gentle gifted golden graceful"
    " grand happy hardy helpful honest hopeful humble ideal jolly keen kind"
    " lively loyal lucid lucky mellow merry mighty mindful neat nimble noble"
    " open patient peaceful plucky polished prompt proud quick quiet radiant"
    " rapid ready refined reliable resolute robust royal sharp shiny silent"
    " skilled sleek smart smooth snappy snowy soft solid sound sparkling spry"
    " stable steady stout strong sunny sure swift tall tender thrifty tidy"
    " trusty tough true upbeat valiant vast vivid warm wild wise worthy zestful"
).split()

_NOUNS = (
    "anchor ant arch ash atlas beacon beam bird bison bloom blossom bridge"
    " brook butter cedar charm cliff clover comet compass coral crane crow"
    " daisy dawn deer dell dew dove eagle ember falcon fern field finch fjord"
    " flame flute forge fox frog garden garnet glade grove harbor hare harvest"
    " haven hawk hearth heron hill horizon ivy jay jewel kiln lake lantern lark"
    " leaf lily lotus lynx maple meadow melody mesa moon moss oak oasis orchard"
    " otter owl panda path peak pearl pine plum pond prairie quail quartz"
    " rabbit raven reef river robin rose sage sail sea seal shell shore sky"
    " spark spire spring spruce star stone stream summit sun swan tide tiger"
    " tower trail trout tulip vale valley willow wolf wren"
).split()

_NOUN_TAGS: dict[str, tuple[str, ...]] = {
    "architecture": (
        "arch bridge buttress column dome gate kiln mill pier spire tower"
        " vault wall wharf"
    ).split(),
    "cloud_weather": (
        "anvil aurora breeze cirrus cloud cumulus dawn dew drizzle fog frost"
        " gale halo haze mist nimbus rain shower squall storm stratus"
        " thunder"
    ).split(),
    "craft_tool": (
        "anvil awl chisel forge hammer lathe loom mallet plow press pulley"
        " quill saw shuttle spindle trowel wheel"
    ).split(),
    "fauna": (
        "ant badger beaver bison crane crow deer dove eagle falcon finch fox"
        " frog hare hawk heron jay lark lynx otter owl quail rabbit raven"
        " robin seal swan tiger trout wolf wren"
    ).split(),
    "flora": (
        "ash bloom blossom cedar clover daisy fern grove ivy leaf lily lotus"
        " maple moss oak orchard pine plum rose sage spruce tulip willow"
    ).split(),
    "landform": (
        "basin bluff canyon cliff dell dune field fjord glade hill horizon"
        " mesa meadow moraine oasis pass path peak prairie ridge shore spire"
        " summit trail vale valley"
    ).split(),
    "light_fire": (
        "beacon beam candle dawn ember flame flare forge hearth kiln lantern"
        " moon spark star sun torch"
    ).split(),
    "material": (
        "amber brass bronze clay cobalt copper fiber glass gold iron nickel"
        " pearl resin silk silver steel thread tin wool zinc"
    ).split(),
    "nautical": (
        "anchor barge canoe cutter dinghy harbor ketch launch raft sail skiff"
        " sloop schooner ship tide yawl"
    ).split(),
    "navigation": (
        "anchor atlas beacon compass harbor horizon lantern map marker path"
        " signal spire star trail waypoint"
    ).split(),
    "space": (
        "asteroid comet eclipse galaxy meteor moon nebula nova orbit planet"
        " pulsar quasar rocket satellite star sun"
    ).split(),
    "stone": (
        "agate basalt boulder flint garnet granite jasper jewel marble"
        " obsidian onyx pebble pearl quartz reef rock shale shell slate stone"
    ).split(),
    "water": (
        "brook current delta harbor lake oasis pond rain reef river sea shore"
        " spring stream tide waterfall wave"
    ).split(),
    "wood_tree": (
        "alder ash beech birch cedar elm grove hickory maple oak pine poplar"
        " spruce timber willow yew"
    ).split(),
}

_KIND_PALETTES: dict[str, tuple[str, ...]] = {
    "WorkItem": ("landform", "stone", "water", "navigation", "craft_tool"),
    "DecisionRecord": ("navigation", "architecture", "light_fire"),
    "Discussion": ("cloud_weather", "water", "flora"),
    "Note": ("cloud_weather", "flora", "water"),
    "Artifact": ("craft_tool", "material", "architecture"),
    "Gate": ("architecture", "navigation", "stone"),
    "Binding": ("navigation", "architecture", "material"),
    "Scope": ("landform", "navigation", "architecture"),
    "LogEntry": ("cloud_weather", "light_fire", "water", "space"),
}

_COUNT_PREFIXES = (
    "two three four five six seven eight nine ten eleven twelve"
).split()

_PLURAL_OVERRIDES = {
    "deer": "deer",
    "fish": "fish",
    "trout": "trout",
    "bison": "bison",
    "swan": "swans",
    "wolf": "wolves",
}

_BLOCKED_ID_WORDS = {
    "agent", "api", "backlog", "binding", "bug", "close", "closed", "commit",
    "context", "create", "decision", "delete", "done", "fix", "gate", "git",
    "issue", "link", "merge", "migrate", "model", "open", "process",
    "record", "release", "review", "route", "ship", "task", "team",
    "transition", "workitem",
}


_SLUG_RE = re.compile(r"[^a-z0-9]+")
_SUMMARY_WORD_RE = re.compile(r"[A-Za-z0-9]+")
_DATETIME_BODY_RE = re.compile(r"^\d{8}_\d{4}-(.+)$")
_UUID_BODY_RE = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}(?:-[0-9a-f]{4})?")
_PASCAL_TOKEN_RE = re.compile(r"^[A-Z][a-z]+(?:[A-Z][a-z]+){1,2}$")
_CAMEL_TOKEN_RE = re.compile(r"^[a-z]+(?:[A-Z][a-z]+){1,2}$")


def _slugify(text: str, max_words: int = 6) -> str:
    cleaned = _SLUG_RE.sub("-", text.lower()).strip("-")
    parts = [p for p in cleaned.split("-") if p]
    return "-".join(parts[:max_words])


def validate_slug_summary(
    text: str | None,
    *,
    min_words: int = 4,
    max_words: int = 6,
) -> list[str]:
    """Validate a caller-supplied semantic slug summary.

    MCP servers cannot reliably summarize long descriptions on their own.
    Callers may provide a compact human/LLM-authored summary instead; the
    server validates only the shape and then slugifies it deterministically.
    """
    if text is None:
        return []
    words = _SUMMARY_WORD_RE.findall(text)
    if len(words) < min_words or len(words) > max_words:
        return [
            (
                "slug_summary must contain "
                f"{min_words}-{max_words} words; got {len(words)}"
            )
        ]
    return []


def _word_pair(rng: random.Random, style: str = "kebab") -> str:
    adj = rng.choice(_ADJECTIVES)
    noun = rng.choice(_NOUNS)
    return _format_words((adj, noun), style)


def _format_words(words: Sequence[str], style: str = "kebab") -> str:
    if style == "pascal":
        return "".join(w.capitalize() for w in words)
    if style == "camel":
        head, *tail = words
        return head + "".join(w.capitalize() for w in tail)
    return "-".join(words)


def _pluralize(noun: str) -> str:
    if noun in _PLURAL_OVERRIDES:
        return _PLURAL_OVERRIDES[noun]
    if noun.endswith("y") and noun[-2:-1] not in "aeiou":
        return noun[:-1] + "ies"
    if noun.endswith(("s", "x", "z", "ch", "sh")):
        return noun + "es"
    if noun.endswith("f"):
        return noun[:-1] + "ves"
    if noun.endswith("fe"):
        return noun[:-2] + "ves"
    return noun + "s"


def _unique_words(words: Iterable[str]) -> tuple[str, ...]:
    return tuple(dict.fromkeys(w for w in words if w))


def noun_pool(palette_tags: Iterable[str] | None = None) -> tuple[str, ...]:
    """Return the noun pool for a set of semantic palette tags.

    ``None`` preserves the historical 120-word default pool used by the
    generator. Passing tags returns the union of matching tagged pools.
    Unknown tags are ignored so project-level settings can be forward
    compatible with newer processkit vocabularies.
    """
    tags = tuple(palette_tags or ())
    if not tags:
        return tuple(_NOUNS)
    words: list[str] = []
    for tag in tags:
        words.extend(_NOUN_TAGS.get(tag, ()))
    return _unique_words(words) or tuple(_NOUNS)


def palette_for_kind(kind: str, intent_text: str | None = None) -> tuple[str, ...]:
    """Select a broad, stable palette for an entity kind and intent text."""
    tags = list(_KIND_PALETTES.get(kind, ()))
    ranked = rank_palette_tags(intent_text or "")
    for tag, _score in ranked:
        if tag not in tags:
            tags.append(tag)
        if len(tags) >= 5:
            break
    return tuple(tags)


def configured_palette_kinds() -> tuple[str, ...]:
    """Return entity kinds with explicit vocabulary palette configuration."""
    return tuple(sorted(_KIND_PALETTES))


def rank_palette_tags(intent_text: str) -> list[tuple[str, int]]:
    """Rank palette tags with a deterministic local lexical score.

    This is the provider-neutral baseline for RAG-assisted allocation. A
    future embedding-backed selector can replace the scoring without
    changing the allocator contract.
    """
    tokens = set(_tokenize(intent_text))
    if not tokens:
        return []
    scores: list[tuple[str, int]] = []
    for tag, words in _NOUN_TAGS.items():
        tag_tokens = set(tag.split("_")) | set(words)
        score = len(tokens & tag_tokens)
        if score:
            scores.append((tag, score))
    return sorted(scores, key=lambda item: (-item[1], item[0]))


def vocabulary_capacity_report(
    *,
    palette_tags: Iterable[str] | None = None,
    allocation_mode: str = "pair",
    existing: Iterable[str] = (),
) -> dict[str, object]:
    """Return capacity and occupancy details for an ID vocabulary palette."""
    adjectives = tuple(_ADJECTIVES)
    nouns = noun_pool(palette_tags)
    countable_nouns = tuple(n for n in nouns if n not in {"quartz", "copper"})
    mode = allocation_mode
    if mode == "auto":
        mode = "pair"
    if mode == "pair":
        capacity = len(adjectives) * len(nouns)
    elif mode == "double_adjective":
        capacity = len(adjectives) * max(len(adjectives) - 1, 0) * len(nouns)
    elif mode == "counted":
        capacity = len(_COUNT_PREFIXES) * len(adjectives) * len(countable_nouns)
    else:
        raise ValueError(f"unknown allocation mode: {allocation_mode!r}")
    tokens = [t for t in (lexical_token_from_id(x) for x in existing) if t]
    used = len(set(tokens))
    return {
        "allocation_mode": mode,
        "palette_tags": list(palette_tags or ()),
        "adjectives": len(adjectives),
        "nouns": len(nouns),
        "count_prefixes": len(_COUNT_PREFIXES),
        "capacity": capacity,
        "used_lexical_tokens": used,
        "occupancy": (used / capacity) if capacity else 0.0,
    }


def lexical_token_from_id(value: str) -> str | None:
    """Extract the human shorthand token from a processkit ID.

    Examples: ``BACK-20260409_1449-CalmFox-title`` -> ``CalmFox`` and
    ``BACK-calm-fox-title`` -> ``calm-fox``.
    """
    if not isinstance(value, str) or "-" not in value:
        return None
    _prefix, body = value.split("-", 1)
    match = _DATETIME_BODY_RE.match(body)
    if match:
        body = match.group(1)
    if _UUID_BODY_RE.match(body):
        return None
    head = body.split("-", 1)[0]
    if _PASCAL_TOKEN_RE.match(head) or _CAMEL_TOKEN_RE.match(head):
        return head
    parts = body.split("-")
    if len(parts) >= 3 and (
        parts[0] in _COUNT_PREFIXES
        or (parts[0] in _ADJECTIVES and parts[1] in _ADJECTIVES)
    ):
        return "-".join(parts[:3])
    if len(parts) >= 2:
        return "-".join(parts[:2])
    return None


def lexical_ambiguities(existing: Iterable[str]) -> dict[str, list[str]]:
    """Return lexical shorthand tokens that appear in more than one ID."""
    seen: dict[str, list[str]] = defaultdict(list)
    for value in existing:
        token = lexical_token_from_id(value)
        if token:
            seen[token].append(value)
    return {token: ids for token, ids in seen.items() if len(ids) > 1}


def detect_lexical_ambiguities(existing: Iterable[str]) -> dict[str, list[str]]:
    """Compatibility alias for doctor/check surfaces."""
    return lexical_ambiguities(existing)


def blocked_words_in_token(token: str) -> list[str]:
    words = set(_tokenize(token))
    return sorted(words & _BLOCKED_ID_WORDS)


def token_components(token: str) -> tuple[str, ...]:
    """Split a lexical token into lowercase word components."""
    return tuple(_tokenize(token))


def is_managed_lexical_token(token: str) -> bool:
    """Return True when a token matches processkit's generated word pools."""
    parts = token_components(token)
    nouns = set(noun_pool())
    tagged_nouns = set().union(*(set(words) for words in _NOUN_TAGS.values()))
    all_nouns = nouns | tagged_nouns
    plurals = {_pluralize(noun) for noun in all_nouns}
    if len(parts) == 2:
        return parts[0] in _ADJECTIVES and parts[1] in all_nouns
    if len(parts) == 3 and parts[0] in _ADJECTIVES and parts[1] in _ADJECTIVES:
        return parts[2] in all_nouns
    if len(parts) == 3 and parts[0] in _COUNT_PREFIXES:
        return parts[1] in _ADJECTIVES and parts[2] in plurals
    return False


def vocabulary_tags() -> dict[str, list[str]]:
    """Return tagged noun pools for status surfaces and doctor checks."""
    return {tag: list(words) for tag, words in sorted(_NOUN_TAGS.items())}


def generate_lexical_token(
    rng: random.Random,
    *,
    word_style: str = "kebab",
    allocation_mode: str = "pair",
    palette_tags: Iterable[str] | None = None,
) -> str:
    """Generate an unprefixed human-readable token from a configured mode."""
    adjectives = tuple(_ADJECTIVES)
    nouns = noun_pool(palette_tags)
    mode = allocation_mode
    if mode == "auto":
        mode = "pair"
    if mode == "pair":
        return _format_words((rng.choice(adjectives), rng.choice(nouns)), word_style)
    if mode == "double_adjective":
        first = rng.choice(adjectives)
        second = rng.choice(adjectives)
        while second == first and len(adjectives) > 1:
            second = rng.choice(adjectives)
        return _format_words((first, second, rng.choice(nouns)), word_style)
    if mode == "counted":
        noun = rng.choice(nouns)
        return _format_words(
            (rng.choice(_COUNT_PREFIXES), rng.choice(adjectives), _pluralize(noun)),
            word_style,
        )
    raise ValueError(f"unknown allocation mode: {allocation_mode!r}")


def generate_id(
    kind: str,
    *,
    format: str = "word",
    word_style: str = "kebab",
    datetime_prefix: bool = False,
    datetime_str: str | None = None,
    slug_text: str | None = None,
    existing: Iterable[str] = (),
    allocation_mode: str = "pair",
    palette_tags: Iterable[str] | None = None,
    intent_text: str | None = None,
    reserved_lexical_tokens: Iterable[str] = (),
    rng: random.Random | None = None,
) -> str:
    """Generate a fresh ID for an entity of the given kind.

    Parameters
    ----------
    kind:
        The primitive kind (e.g. "WorkItem"). Determines the prefix.
    format:
        ``word`` or ``uuid``.
    word_style:
        ``pascal`` (e.g. ``CalmFox``), ``camel`` (e.g. ``calmFox``),
        or ``kebab`` (e.g. ``calm-fox``).
        Only applies when ``format="word"``.
    datetime_prefix:
        If True, prepend a ``YYYYMMDD_HHMM`` timestamp before the word
        pair. Use ``datetime_str`` to supply a specific value; if omitted
        the current UTC time is used.
    datetime_str:
        A pre-formatted datetime string to embed (e.g. ``"20260409_1449"``).
        Only used when ``datetime_prefix=True``.
    slug_text:
        If provided, a slug derived from this text is appended (e.g. the
        title of a WorkItem).
    existing:
        Already-used IDs (with or without prefix). The generator will not
        return one of these.
    rng:
        Optional random source for deterministic tests.
    """
    if kind not in KIND_PREFIXES:
        raise ValueError(f"unknown kind: {kind!r}")
    prefix = KIND_PREFIXES[kind]
    rng = rng or random.Random()
    existing = tuple(existing)
    used = {_strip_prefix(x, prefix) for x in existing}
    reserved_tokens = set(reserved_lexical_tokens)
    reserved_tokens.update(
        t for t in (lexical_token_from_id(x) for x in existing) if t
    )
    tags = tuple(
        palette_tags
        or (palette_for_kind(kind, intent_text) if intent_text else ())
    )

    # Build datetime prefix component
    dt_part = ""
    if datetime_prefix:
        if datetime_str:
            dt_part = datetime_str
        else:
            dt_part = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M")

    for _ in range(50):
        body = _generate_body(
            format,
            rng,
            word_style,
            allocation_mode=allocation_mode,
            palette_tags=tags,
        )
        lexical_body = body
        if dt_part:
            body = f"{dt_part}-{body}"
        if slug_text:
            slug = _slugify(slug_text)
            if slug:
                body = f"{body}-{slug}"
        if body not in used and (
            not reserved_tokens or lexical_body not in reserved_tokens
        ):
            return f"{prefix}-{body}"
    # Fallback: append a hex tag until unique
    while True:
        suffix = rng.randrange(16 ** 4)
        lexical_body = generate_lexical_token(
            rng,
            word_style=word_style,
            allocation_mode=allocation_mode,
            palette_tags=tags,
        )
        body = f"{lexical_body}-{suffix:04x}"
        if dt_part:
            body = f"{dt_part}-{body}"
        if slug_text:
            slug = _slugify(slug_text)
            if slug:
                body = f"{body}-{slug}"
        if body not in used and (
            not reserved_tokens or lexical_body not in reserved_tokens
        ):
            return f"{prefix}-{body}"


def _generate_body(
    format: str,
    rng: random.Random,
    word_style: str = "kebab",
    *,
    allocation_mode: str = "pair",
    palette_tags: Iterable[str] | None = None,
) -> str:
    if format == "word":
        if allocation_mode == "pair" and palette_tags is None:
            return _word_pair(rng, word_style)
        return generate_lexical_token(
            rng,
            word_style=word_style,
            allocation_mode=allocation_mode,
            palette_tags=palette_tags,
        )
    if format == "uuid":
        return str(_uuid.UUID(int=rng.getrandbits(128)))[:18]
    raise ValueError(f"unknown id format: {format!r}")


def _strip_prefix(value: str, prefix: str) -> str:
    if value.startswith(prefix + "-"):
        return value[len(prefix) + 1 :]
    return value


def _tokenize(text: str) -> list[str]:
    tokens: list[str] = []
    for chunk in re.findall(r"[A-Za-z0-9]+", text):
        parts = re.sub(r"(?<!^)(?=[A-Z])", " ", chunk).split()
        tokens.extend(part.lower() for part in parts if part)
    return tokens
