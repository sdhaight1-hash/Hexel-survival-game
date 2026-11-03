#!/usr/bin/env python3
"""Check the versioned flora research registry; no external packages needed.

This file is deliberately DATA validation, not an ecological balance review.
"""
from collections import Counter
import json
from pathlib import Path
import re
import sys
import unittest

REGISTRY = Path(__file__).resolve().parents[1] / "assets" / "flora" / "reference_registry.v1.json"
ID = re.compile(r"^flora\.(vanilla|wildcraft|florasphere_historical)\.[a-z_]+\.[a-z0-9_]+$")
WORLDGEN_FIELDS = {
    "temperature_c", "rainfall_fraction",
    "forestation_fraction", "source_elevation_fraction",
}
EXPECTED_GROUPS = {
    "tree.vanilla": 32,
    "fruit_tree.vanilla": 12,
    "berry.vanilla": 10,
    "flower.vanilla": 30,
    "crop.vanilla": 18,
    "fungus.vanilla": 45,
    "tree_shrub.wildcraft": 51,
    "shrub.florasphere_historical": 10,
}


def validate_interval(value, name):
    if value is None:
        return
    if not isinstance(value, list) or len(value) != 2:
        raise ValueError(f"{name}: expected null or [minimum, maximum]")
    a, b = value
    if not all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in value):
        raise ValueError(f"{name}: range must contain two numbers")
    if not (-100.0 <= a <= b <= 100.0):
        raise ValueError(f"{name}: inverted/implausible range")
    if name != "temperature_c" and not (0 <= a <= b <= 1):
        raise ValueError(f"{name}: expected normalized fraction 0..1")


def validate_registry(data):
    if data.get("schema_version") != 1:
        raise ValueError("Unsupported flora schema version")
    if not isinstance(data.get("registry_revision"), str) or not data["registry_revision"]:
        raise ValueError("Missing registry revision")
    rows = data.get("records")
    if not isinstance(rows, list) or len(rows) != 208:
        raise ValueError("Expected exactly 208 historical research rows, not unique taxa")
    identifiers = set()
    groups = Counter()
    for entry in rows:
        identifier = entry.get("id")
        if not isinstance(identifier, str) or not ID.fullmatch(identifier):
            raise ValueError(f"Malformed flora ID: {identifier!r}")
        if identifier in identifiers:
            raise ValueError(f"Duplicate flora ID: {identifier}")
        identifiers.add(identifier)
        if not isinstance(entry.get("display_name"), str) or not entry["display_name"]:
            raise ValueError(f"{identifier}: missing display name")
        origin = entry.get("origin")
        category = entry.get("category")
        if origin not in {"vanilla", "wildcraft", "florasphere_historical"}:
            raise ValueError(f"{identifier}: unknown source namespace")
        if not identifier.startswith(f"flora.{origin}.{category}."):
            raise ValueError(f"{identifier}: ID namespace inconsistent with origin/category")
        groups[f"{category}.{origin}"] += 1
        if entry.get("approval") != "research_only":
            raise ValueError(f"{identifier}: research cannot be auto-approved for worldgen")
        provenance = entry.get("provenance")
        if not isinstance(provenance, dict):
            raise ValueError(f"{identifier}: provenance required")
        if not isinstance(provenance.get("source_url"), str) or not provenance["source_url"].startswith("https://"):
            raise ValueError(f"{identifier}: source URL required")
        if not provenance.get("evidence_status") or not provenance.get("as_of"):
            raise ValueError(f"{identifier}: evidence status/date required")
        worldgen = entry.get("worldgen")
        if not isinstance(worldgen, dict) or set(worldgen) != WORLDGEN_FIELDS:
            raise ValueError(f"{identifier}: unknown/missing climate units")
        for key, value in worldgen.items():
            validate_interval(value, key)
        if origin != "vanilla" and any(v is not None for v in worldgen.values()):
            raise ValueError(f"{identifier}: mod numeric worldgen rules are not verified")
        for separate in ("survival", "establishment", "phenology", "harvest", "visuals"):
            if separate not in entry:
                raise ValueError(f"{identifier}: missing independent {separate} slot")
    if dict(groups) != EXPECTED_GROUPS:
        raise ValueError(f"Reference inventory category drift: {dict(groups)}")
    if data.get("counts_by_source") != EXPECTED_GROUPS:
        raise ValueError("Header inventory counts do not match records")
    problem = next(row for row in rows if row["id"] == "flora.vanilla.flower.cow_parsley_forest")
    if problem["worldgen"]["rainfall_fraction"] is not None:
        raise ValueError("Inverted flower rain range must remain unknown until verified")
    if problem["provenance"]["evidence_status"] != "source_inconsistent":
        raise ValueError("Source inconsistency flag for forest cow parsley was lost")
    return len(rows)


class TestFloraRegistry(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.registry = json.loads(REGISTRY.read_text(encoding="utf-8"))

    def test_registry_schema_and_provenance(self):
        self.assertEqual(validate_registry(self.registry), 208)

    def test_rejects_unapproved_import_becoming_gameplay_rule(self):
        import copy

        broken = copy.deepcopy(self.registry)
        broken["records"][0]["approval"] = "approved"
        with self.assertRaisesRegex(ValueError, "auto-approved"):
            validate_registry(broken)

    def test_rejects_inverted_or_unnormalized_climate_bands(self):
        for bad in ([0.5, 0.1], [95, 255]):
            with self.subTest(bad=bad):
                with self.assertRaises(ValueError):
                    validate_interval(bad, "rainfall_fraction")

    def test_rejects_ambiguous_absent_climate_fields(self):
        import copy

        broken = copy.deepcopy(self.registry)
        del broken["records"][0]["worldgen"]["rainfall_fraction"]
        with self.assertRaisesRegex(ValueError, "climate units"):
            validate_registry(broken)


if __name__ == "__main__":
    unittest.main(verbosity=2)
