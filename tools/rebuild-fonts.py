#!/usr/bin/env python3
"""Rebuild embedded Roboto weights from the original Fontsource WOFF2 subsets."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile
from fontTools.merge import Merger
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("source", type=Path, help="Original ui/assets/fonts directory")
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
(root / "target").mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(dir=root / "target") as directory:
    stage = Path(directory)
    subsets = ["latin", "latin-ext", "cyrillic"]
    for subset in subsets:
        path = stage / f"roboto-{subset}.woff2"
        shutil.copyfile(args.source / f"roboto-{subset}-wght-normal.woff2", path)
        subprocess.run(["woff2_decompress", str(path)], check=True)
    for weight, name in [(400, "Regular"), (500, "Medium"), (600, "Semibold"), (700, "Bold")]:
        paths = []
        for subset in subsets:
            font = instantiateVariableFont(
                TTFont(stage / f"roboto-{subset}.ttf"), {"wght": weight}, inplace=True
            )
            path = stage / f"{subset}-{name}.ttf"
            font.save(path)
            paths.append(str(path))
        Merger().merge(paths).save(root / "assets/fonts" / f"Roboto-{name}.ttf")
