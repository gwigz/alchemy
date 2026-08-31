#!/usr/bin/env python3
"""Generate Inventory Explorer's grouped menu from the gallery inventory menu."""

from __future__ import annotations

import argparse
import copy
import xml.etree.ElementTree as ET
from pathlib import Path


GROUPS = ("primary", "organize", "share", "clipboard", "destructive", "create")

GROUP_ORDER = {
    "primary": ("Open", "Open Original", "Landmark Open"),
    "organize": (
        "open_in_current_window",
        "open_in_new_window",
        "Open Folder Separator",
        "Rename",
        "Add to Favorites",
        "Remove from Favorites",
        "thumbnail",
        "Find Original",
        "Find Links",
        "Subfolder Separator",
        "New folder from selected",
        "Ungroup folder items",
    ),
    "share": ("Share", "Properties", "Copy Asset UUID", "Copy Inventory UUID"),
    "clipboard": ("Copy Separator", "Cut", "Copy", "Paste", "Paste As Link", "Replace Links"),
    "destructive": (
        "Paste Separator",
        "Delete",
        "Delete System Folder",
        "Purge Item",
        "Restore Item",
        "Empty Trash",
        "Empty Lost And Found",
    ),
    "create": ("create_new", "upload_options", "upload_def"),
}

ORGANIZE = {
    "Find Original",
    "Find Links",
    "Rename",
    "thumbnail",
    "open_in_current_window",
    "open_in_new_window",
    "New folder from selected",
    "Ungroup folder items",
    "Add to Favorites",
    "Remove from Favorites",
    "Subfolder Separator",
    "Open Folder Separator",
}
SHARE = {"Share", "Properties", "Copy Asset UUID", "Copy Inventory UUID"}
CLIPBOARD = {"Copy Separator", "Cut", "Copy", "Paste", "Paste As Link", "Replace Links"}
DESTRUCTIVE = {
    "Paste Separator",
    "Delete",
    "Delete System Folder",
    "Purge Item",
    "Restore Item",
    "Empty Trash",
    "Empty Lost And Found",
}
CREATE = {"create_new", "upload_options", "upload_def"}


def group_for(name: str) -> str:
    if name in ORGANIZE:
        return "organize"
    if name in SHARE:
        return "share"
    if name in CLIPBOARD:
        return "clipboard"
    if name in DESTRUCTIVE:
        return "destructive"
    if name in CREATE:
        return "create"
    return "primary"


def generate(source: Path, output: Path) -> None:
    source_root = ET.parse(source).getroot()
    groups: dict[str, list[ET.Element]] = {name: [] for name in GROUPS}

    for child in source_root:
        name = child.get("name")
        if child.tag == "menu_item_separator" and not name:
            continue
        if not name:
            raise ValueError(f"Direct menu child <{child.tag}> has no name")
        groups[group_for(name)].append(copy.deepcopy(child))

    for group, children in groups.items():
        preferred = {name: index for index, name in enumerate(GROUP_ORDER[group])}
        children.sort(key=lambda child: preferred.get(child.get("name"), len(preferred)))

    root = ET.Element(
        "context_menu",
        {"layout": "topleft", "name": "Inventory Explorer"},
    )
    for group in GROUPS:
        for child in groups[group]:
            root.append(child)

    generated_names = [child.get("name") for child in root]
    if len(generated_names) != len(set(generated_names)):
        duplicates = sorted({name for name in generated_names if generated_names.count(name) > 1})
        raise ValueError(f"Duplicate direct menu names: {', '.join(duplicates)}")

    tree = ET.ElementTree(root)
    ET.indent(tree, space="  ")
    output.parent.mkdir(parents=True, exist_ok=True)
    tree.write(output, encoding="utf-8", xml_declaration=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("indra/newview/skins/default/xui/en/menu_gallery_inventory.xml"),
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("indra/newview/skins/default/xui/en/menu_al_inventory_explorer.xml"),
    )
    args = parser.parse_args()

    if args.check:
        temporary = args.output.with_suffix(".xml.generated")
        generate(args.source, temporary)
        try:
            if not args.output.exists() or args.output.read_bytes() != temporary.read_bytes():
                raise SystemExit(f"{args.output} is stale; run {Path(__file__).name}")
        finally:
            temporary.unlink(missing_ok=True)
        return

    generate(args.source, args.output)


if __name__ == "__main__":
    main()
