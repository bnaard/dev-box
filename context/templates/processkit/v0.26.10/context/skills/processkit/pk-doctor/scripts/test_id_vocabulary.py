from pathlib import Path

from checks import id_vocabulary


def test_id_vocabulary_reports_capacity_and_ambiguity(monkeypatch):
    monkeypatch.setattr(id_vocabulary, "_sqlite_vec_available", lambda: True)
    monkeypatch.setattr(
        id_vocabulary,
        "_indexed_ids",
        lambda _repo_root: [
            "BACK-20260409_1449-CapableProudEclipse-one",
            "DEC-20260409_1450-CapableProudEclipse-two",
        ],
    )

    results = id_vocabulary.run({"repo_root": Path.cwd()})
    ids = {result.id for result in results}

    assert "id-vocabulary.configured-kind-capacity-ok" in ids
    assert "id-vocabulary.high-volume-capacity-ok" in ids
    assert "id-vocabulary.lexical-token-historical-ambiguity" in ids
    assert "id-vocabulary.semantic-index-ready" in ids

    ambiguity = next(
        result for result in results
        if result.id == "id-vocabulary.lexical-token-historical-ambiguity"
    )
    assert ambiguity.severity == "INFO"
    assert ambiguity.to_dict()["action_required"] is False
