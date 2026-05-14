import random

from processkit import ids


def test_generate_id_default_word_path_preserves_legacy_pool():
    generated = ids.generate_id("WorkItem", rng=random.Random(1))

    assert generated == "BACK-capable-oasis"


def test_generate_lexical_token_supports_high_volume_modes():
    double = ids.generate_lexical_token(
        random.Random(1),
        allocation_mode="double_adjective",
        word_style="pascal",
        palette_tags=("space",),
    )
    counted = ids.generate_lexical_token(
        random.Random(1),
        allocation_mode="counted",
        word_style="pascal",
        palette_tags=("water",),
    )

    assert double == "CapableProudEclipse"
    assert counted == "ElevenTrustyLakes"


def test_lexical_token_extraction_handles_three_token_ids():
    assert (
        ids.lexical_token_from_id("BACK-20260409_1449-CleanRapidRiver-title")
        == "CleanRapidRiver"
    )
    assert (
        ids.lexical_token_from_id("BACK-seven-rapid-rivers-title")
        == "seven-rapid-rivers"
    )


def test_managed_token_detection_skips_structural_names():
    assert ids.is_managed_lexical_token("CapableProudEclipse")
    assert not ids.is_managed_lexical_token("ModelSpec")


def test_capacity_report_and_ambiguity_detection():
    report = ids.vocabulary_capacity_report(
        palette_tags=("space",),
        allocation_mode="double_adjective",
        existing=(
            "BACK-20260409_1449-CleanRapidRiver-one",
            "DEC-20260409_1450-CleanRapidRiver-two",
        ),
    )

    assert report["capacity"] == 228480
    assert report["used_lexical_tokens"] == 1
    assert ids.detect_lexical_ambiguities(
        (
            "BACK-20260409_1449-CleanRapidRiver-one",
            "DEC-20260409_1450-CleanRapidRiver-two",
        )
    ) == {
        "CleanRapidRiver": [
            "BACK-20260409_1449-CleanRapidRiver-one",
            "DEC-20260409_1450-CleanRapidRiver-two",
        ]
    }


def test_generate_id_reserves_existing_lexical_tokens():
    generated = ids.generate_id(
        "WorkItem",
        allocation_mode="double_adjective",
        word_style="pascal",
        existing=("BACK-20260409_1449-CapableProudEclipse-one",),
        rng=random.Random(1),
    )

    assert generated != "BACK-CapableProudEclipse"
    assert ids.lexical_token_from_id(generated) != "CapableProudEclipse"


def test_configured_palette_kinds_are_public_and_stable():
    assert "WorkItem" in ids.configured_palette_kinds()
    assert ids.configured_palette_kinds() == tuple(
        sorted(ids.configured_palette_kinds())
    )
