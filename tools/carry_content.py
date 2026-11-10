#!/usr/bin/env python3
"""Carry Modular Routers' assets out of the pack jar, and check the carry.

Only the files this slice declares are taken: the router's block model, states
and textures, and the blank/Mk1 module items. Everything else the jar carries
(the other 21 modules, 10 upgrades, 11 augments, 5 filters, the Patchouli book
and the GUI textures) is left out — `FINDINGS.md` lists it and `notes.md` records
it as a gap.

    python3 tools/carry_content.py            # write the files
    python3 tools/carry_content.py --check    # fail when a carried file differs
"""
import hashlib
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLONE = ROOT.parents[1]
JAR = CLONE / "ports/_shared/licence-cache/modular-routers-13.2.7+mc1.21.1.jar"
NS = "modularrouters"
CONTENT = ROOT / "content"

TEXTURES = [
    "block/modular_router_back.png",
    "block/modular_router_bottom.png",
    "block/modular_router_front.png",
    "block/modular_router_front_active.png",
    "block/modular_router_front_active.png.mcmeta",
    "block/modular_router_other.png",
    "block/modular_router_side.png",
    "block/modular_router_top.png",
    "item/blank_module.png",
    "item/module/module_layer0.png",
    "item/module/module_layer1.png",
    "item/module/puller_module_1.png",
    "item/module/sender_module_1.png",
]
MODELS = [
    "block/modular_router.json",
    "block/modular_router_off.json",
    "block/modular_router_on.json",
    "item/blank_module.json",
    "item/modular_router.json",
    "item/puller_module_1.json",
    "item/sender_module_1.json",
]
BLOCKSTATES = ["modular_router.json"]
TAGS = [
    "data/minecraft/tags/block/mineable/pickaxe.json",
    "data/minecraft/tags/block/needs_iron_tool.json",
]


def carried() -> dict[Path, bytes]:
    files: dict[Path, bytes] = {}
    with zipfile.ZipFile(JAR) as jar:
        for name in TEXTURES:
            files[CONTENT / "assets" / NS / "textures" / name] = jar.read(
                f"assets/{NS}/textures/{name}"
            )
        for name in MODELS:
            files[CONTENT / "assets" / NS / "models" / name] = jar.read(
                f"assets/{NS}/models/{name}"
            )
        for name in BLOCKSTATES:
            files[CONTENT / "assets" / NS / "blockstates" / name] = jar.read(
                f"assets/{NS}/blockstates/{name}"
            )
        for name in TAGS:
            files[CONTENT / name] = jar.read(name)
    return files


def main() -> int:
    check = "--check" in sys.argv[1:]
    if not JAR.is_file():
        raise SystemExit(f"missing jar: {JAR}")
    files = carried()
    problems = 0
    for path, data in sorted(files.items()):
        relative = path.relative_to(ROOT)
        if check:
            if not path.is_file() or path.read_bytes() != data:
                print(f"differs from the jar: {relative}")
                problems += 1
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    if check:
        print(f"{len(files) - problems}/{len(files)} carried files match the jar")
        return 1 if problems else 0
    for path in sorted(files):
        print(f"{path.relative_to(ROOT)}  {hashlib.sha256(path.read_bytes()).hexdigest()[:16]}")
    print(f"{len(files)} files carried from {JAR.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
