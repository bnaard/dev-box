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
from datetime import datetime, timezone
from typing import Iterable

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


_SLUG_RE = re.compile(r"[^a-z0-9]+")
_SUMMARY_WORD_RE = re.compile(r"[A-Za-z0-9]+")


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
    if style == "pascal":
        return f"{adj.capitalize()}{noun.capitalize()}"
    if style == "camel":
        return f"{adj}{noun.capitalize()}"
    return f"{adj}-{noun}"


def generate_id(
    kind: str,
    *,
    format: str = "word",
    word_style: str = "kebab",
    datetime_prefix: bool = False,
    datetime_str: str | None = None,
    slug_text: str | None = None,
    existing: Iterable[str] = (),
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
    used = {_strip_prefix(x, prefix) for x in existing}

    # Build datetime prefix component
    dt_part = ""
    if datetime_prefix:
        if datetime_str:
            dt_part = datetime_str
        else:
            dt_part = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M")

    for _ in range(50):
        body = _generate_body(format, rng, word_style)
        if dt_part:
            body = f"{dt_part}-{body}"
        if slug_text:
            slug = _slugify(slug_text)
            if slug:
                body = f"{body}-{slug}"
        if body not in used:
            return f"{prefix}-{body}"
    # Fallback: append a hex tag until unique
    while True:
        suffix = rng.randrange(16 ** 4)
        body = f"{_word_pair(rng, word_style)}-{suffix:04x}"
        if dt_part:
            body = f"{dt_part}-{body}"
        if slug_text:
            slug = _slugify(slug_text)
            if slug:
                body = f"{body}-{slug}"
        if body not in used:
            return f"{prefix}-{body}"


def _generate_body(
    format: str, rng: random.Random, word_style: str = "kebab"
) -> str:
    if format == "word":
        return _word_pair(rng, word_style)
    if format == "uuid":
        return str(_uuid.UUID(int=rng.getrandbits(128)))[:18]
    raise ValueError(f"unknown id format: {format!r}")


def _strip_prefix(value: str, prefix: str) -> str:
    if value.startswith(prefix + "-"):
        return value[len(prefix) + 1 :]
    return value
