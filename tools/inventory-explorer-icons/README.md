# Inventory Explorer icon generator

This tool renders the Inventory Explorer Lucide icon pack as white, transparent 16 x 16
RGBA PNGs. LLUI applies the final colors through `LLUIColor` values.

The manifest is `manifest.json`. Each entry maps a viewer texture name to one SVG
in the sibling `viewer/fonts/lucide/icons` checkout and declares the icon size
and output stroke width inside the 16 x 16 canvas. Optional `offsetX` and
`offsetY` values move an awkward glyph in quarter-pixel steps. The generator
rejects offsets larger than two pixels.

```sh
bun install
bun run generate
bun run check
```

`generate` writes the PNGs to
`indra/newview/skins/default/textures/inventory_explorer`, refreshes the generated block
in `textures.xml`, copies the Lucide license, and writes `contact-sheet.png`.
`check` regenerates everything in memory and fails if any checked-in output differs.
