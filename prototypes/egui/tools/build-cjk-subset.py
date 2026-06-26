#!/usr/bin/env python3
"""Regenerate the CJK subset font (assets/fonts/SourceHanSansJP-Subset.ttf).

The app's Latin fonts (Inter etc.) carry no CJK glyphs, so a Source Han Sans subset sits at the
back of the fallback chain. Keeping it a subset keeps the WASM small, but it must cover every glyph
the UI can show -- the harvested locale strings AND the language-picker names defined in source.
Run this after harvesting new locale strings or adding a CJK language name, then rebuild.

Outline format matters: egui's font backend (ttf-parser) renders the static `glyf` outlines here,
not CFF2, so the variable source face is pinned to Regular and converted cubic -> quadratic.

Usage: tools/build-cjk-subset.sh   (bootstraps fonttools, then runs this)
Source font override: SHS_TTC=/path/to/SourceHanSans-VF.otf.ttc
"""

import glob
import os
import re
import sys

from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.subset import Options, Subsetter
from fontTools.ttLib import TTCollection, TTFont, newTable
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TTC = os.environ.get(
    "SHS_TTC",
    "/Applications/Alchemy Test.app/Contents/Resources/fonts/SourceHanSans-VF.otf.ttc",
)
OUT = os.path.join(ROOT, "assets/fonts/SourceHanSansJP-Subset.ttf")


def wanted_codepoints():
    cps = set()
    for path in glob.glob(os.path.join(ROOT, "i18n/**/*.ftl"), recursive=True):
        with open(path, encoding="utf-8") as f:
            cps.update(ord(ch) for ch in f.read())
    # Language-picker display names live in source, not the .ftl files.
    with open(os.path.join(ROOT, "src/i18n.rs"), encoding="utf-8") as f:
        cps.update(ord(ch) for ch in f.read())
    # Only the glyphs the CJK fallback needs to supply: anything past Latin/Cyrillic that the
    # text fonts already cover. The subsetter silently drops codepoints the face lacks.
    return sorted(c for c in cps if c >= 0x2000)


def main():
    if not os.path.exists(TTC):
        sys.exit(f"source font not found: {TTC}\nset SHS_TTC to a Source Han Sans VF .ttc")

    font = TTCollection(TTC).fonts[0]  # face 0 = Source Han Sans VF (JP)
    axes = {a.axisTag: a.defaultValue for a in font["fvar"].axes}
    axes["wght"] = 400.0  # Regular
    instantiateVariableFont(font, axes, inplace=True)

    opts = Options()
    opts.recalc_timestamp = False
    ss = Subsetter(options=opts)
    ss.populate(unicodes=wanted_codepoints())
    ss.subset(font)

    # CFF2 -> static glyf (ttf-parser won't render CFF2 outlines).
    glyph_set = font.getGlyphSet()
    glyf = newTable("glyf")
    glyf.glyphOrder = font.getGlyphOrder()
    glyf.glyphs = {}
    for name in glyf.glyphOrder:
        pen = TTGlyphPen(glyph_set)
        glyph_set[name].draw(Cu2QuPen(pen, 1.0, reverse_direction=True))
        glyf[name] = pen.glyph()
    font["glyf"] = glyf
    font["loca"] = newTable("loca")  # populated from glyf on compile

    maxp = font["maxp"]
    maxp.tableVersion = 0x00010000
    for attr, default in [
        ("maxZones", 1), ("maxTwilightPoints", 0), ("maxStorage", 0),
        ("maxFunctionDefs", 0), ("maxInstructionDefs", 0), ("maxStackElements", 0),
        ("maxSizeOfInstructions", 0), ("maxComponentElements", 0), ("maxComponentDepth", 0),
    ]:
        if not hasattr(maxp, attr):
            setattr(maxp, attr, default)
    for tag in ("CFF ", "CFF2", "VORG"):
        if tag in font:
            del font[tag]
    font["head"].glyphDataFormat = 0
    font.sfntVersion = "\x00\x01\x00\x00"

    font.save(OUT)
    print(f"wrote {OUT}  ({os.path.getsize(OUT)} bytes, {font['maxp'].numGlyphs} glyphs)")


if __name__ == "__main__":
    main()
