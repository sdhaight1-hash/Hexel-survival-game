#!/usr/bin/env python3
"""Check source texture inventory and explicit face-set references (stdlib only)."""
import json
from pathlib import Path
import re
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
TEXTURES = ROOT / "assets" / "textures"
MANIFEST = TEXTURES / "manifest.json"
CATALOG = ROOT / "assets" / "materials" / "catalog.v1.json"
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def existing_texture(relative_path):
    if not isinstance(relative_path, str):
        raise ValueError("Texture path must be text")
    candidate = Path(relative_path)
    if candidate.is_absolute() or ".." in candidate.parts or candidate.suffix != ".png":
        raise ValueError(f"Invalid texture path: {relative_path}")
    path = (TEXTURES / candidate).resolve()
    if not path.is_relative_to(TEXTURES.resolve()) or not path.is_file():
        raise ValueError(f"Missing texture: {relative_path}")
    return path


def check_32px_rgba(path):
    # IHDR header validates physical dimensions without installing Pillow.
    with path.open("rb") as f:
        header = f.read(26)
    if len(header) != 26 or header[:8] != PNG_SIGNATURE or header[12:16] != b"IHDR":
        raise ValueError(f"Not a PNG: {path}")
    w, h = struct.unpack(">II", header[16:24])
    if (w, h, header[24], header[25]) != (32, 32, 8, 6):
        raise ValueError(f"Expected 32x32 RGBA8 PNG: {path}")


def validate():
    source = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if not isinstance(source, list) or not source:
        raise ValueError("Empty texture inventory")
    known = set()
    names = set()
    for entry in source:
        name = entry["name"]
        if name in names:
            raise ValueError(f"Repeated source name: {name}")
        names.add(name)
        for key in ("crop_path", "tile32_path"):
            prefixed = entry[key]
            prefix = "assets/textures/"
            if not prefixed.startswith(prefix):
                raise ValueError(f"Outside texture root: {prefixed}")
            relative = prefixed[len(prefix):]
            path = existing_texture(relative)
            if key == "tile32_path":
                check_32px_rgba(path)
                known.add(relative)

    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    if catalog.get("version") != 1:
        raise ValueError("Unsupported material catalog version")
    ids = set()
    for item in catalog["materials"]:
        identifier = item["id"]
        if identifier in ids or not re.fullmatch(r"(soil|rock|ore|mineral)\.[a-z0-9_]+", identifier):
            raise ValueError(f"Invalid/repeated material ID: {identifier}")
        ids.add(identifier)
        if item["kind"] not in {"soil", "rock", "ore", "mineral"}:
            raise ValueError(f"Unknown material class: {identifier}")
        if item["status"] not in {"legacy-preview", "production"}:
            raise ValueError(f"Missing material quality status: {identifier}")
        if set(item["faces"]) != {"top", "side", "bottom"}:
            raise ValueError(f"Missing or extra face role in: {identifier}")
        for relative in item["faces"].values():
            if relative not in known:
                raise ValueError(f"{identifier} references an unregistered texture: {relative}")
            check_32px_rgba(existing_texture(relative))
    for cover in catalog.get("cover_previews", []):
        if cover["status"] != "legacy-preview":
            raise ValueError("Opaque cover references cannot be production overlays")
        if cover["texture"] not in known:
            raise ValueError(f"Unregistered ground-cover swatch: {cover['texture']}")
        check_32px_rgba(existing_texture(cover["texture"]))
    print(f"Validated {len(source)} 32px source textures and {len(ids)} face sets")


if __name__ == "__main__":
    try:
        validate()
    except (ValueError, KeyError, OSError, json.JSONDecodeError) as error:
        print(f"Material catalog error: {error}", file=sys.stderr)
        sys.exit(1)
