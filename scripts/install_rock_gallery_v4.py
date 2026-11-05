"""Install only the 69 original V4 rock PNG candidates for the opt-in gallery.

Usage: python scripts/install_rock_gallery_v4.py <downloaded V4 ZIP>
The ZIP is not production art; this script never touches existing P0/v2 assets.
"""
from pathlib import Path
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "assets" / "rock-gallery-v4"
PREFIX = "hexel_rocks_v4_pixel_geology/"


def install(archive: Path) -> None:
    with zipfile.ZipFile(archive) as package:
        pictures = [p for p in package.namelist() if p.endswith(".png")]
        if len(pictures) != 69:
            raise ValueError(f"Expected 69 V4 PNGs; got {len(pictures)}")
        for name in pictures:
            if not name.startswith(PREFIX):
                raise ValueError(f"Unexpected archive member: {name}")
            relative = Path(name[len(PREFIX):])
            if len(relative.parts) != 3 or relative.suffix.lower() != ".png":
                raise ValueError(f"Unexpected rock texture path: {relative}")
            family, rock, filename = relative.parts
            if family not in {"sedimentary", "metamorphic", "igneous_volcanic"}:
                raise ValueError(f"Unexpected family: {family}")
            if filename not in {f"{rock}_{face}.png" for face in ("top", "side", "bottom")}:
                raise ValueError(f"Unexpected rock face: {relative}")
            target = TARGET / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(package.read(name))
    print(f"Installed 69 PNGs under {TARGET}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python scripts/install_rock_gallery_v4.py <V4 ZIP>")
    install(Path(sys.argv[1]))
