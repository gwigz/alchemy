#!/usr/bin/env python3
"""Verify the Inventory Explorer collection manifest and LLUI wiring."""

from __future__ import annotations

import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "indra/newview/alpanelinventoryexplorer.cpp"
XUI = ROOT / "indra/newview/skins/default/xui/en"
MAIN_PANEL = XUI / "panel_al_inventory_explorer.xml"
ROW_PANEL = XUI / "panel_al_inventory_explorer_collection_row.xml"
ADD_MENU = XUI / "menu_al_inventory_explorer_add_filter.xml"

EXPECTED_TYPE_IDS = [
    "clothing",
    "bodypart",
    "object",
    "texture",
    "landmark",
    "animation",
    "gesture",
    "notecard",
    "sound",
    "settings",
]


def fail(message: str) -> None:
    raise RuntimeError(message)


def parse_xml(path: Path) -> ET.Element:
    try:
        return ET.parse(path).getroot()
    except ET.ParseError as error:
        fail(f"invalid XML in {path.relative_to(ROOT)}: {error}")


def names(root: ET.Element) -> set[str]:
    return {value for element in root.iter() if (value := element.get("name"))}


def main() -> int:
    source = SOURCE.read_text(encoding="utf-8")
    panel = parse_xml(MAIN_PANEL)
    row = parse_xml(ROW_PANEL)
    menu = parse_xml(ADD_MENU)

    manifest_match = re.search(
        r"TYPE_FILTER_SPECS\{\{(?P<body>.*?)\}\};", source, re.DOTALL
    )
    if not manifest_match:
        fail("TYPE_FILTER_SPECS manifest is missing")
    manifest_ids = re.findall(r'\{\s*"([a-z]+)"\s*,', manifest_match.group("body"))
    if manifest_ids != EXPECTED_TYPE_IDS:
        fail(f"unexpected type-filter manifest: {manifest_ids}")
    if len(manifest_ids) != len(set(manifest_ids)):
        fail("duplicate type-filter IDs in TYPE_FILTER_SPECS")

    menu_ids = [
        element.get("parameter", "")
        for element in menu.iter("on_click")
        if element.get("function") == "InventoryExplorer.AddTypeFilter"
    ]
    if menu_ids != manifest_ids:
        fail(f"add-filter menu does not match manifest: {menu_ids}")

    panel_names = names(panel)
    required_panel_names = {
        "recent_collection_view",
        "worn_collection_view",
        "favorites_collection_view",
        "type_filter_collection_view",
        "type_filter_list",
        "add_filter_button",
    }
    missing_panel_names = sorted(required_panel_names - panel_names)
    if missing_panel_names:
        fail(f"main panel is missing controls: {missing_panel_names}")

    tag_by_name = {
        element.get("name"): element.tag for element in panel.iter() if element.get("name")
    }
    expected_tags = {
        "recent_collection_view": "recent_inventory_panel",
        "favorites_collection_view": "favorites_inventory_panel",
        "worn_collection_view": "inventory_panel",
        "type_filter_collection_view": "inventory_panel",
    }
    for control_name, expected_tag in expected_tags.items():
        if tag_by_name.get(control_name) != expected_tag:
            fail(f"{control_name} must use <{expected_tag}>")

    required_row_names = {
        "selection_background",
        "collection_icon",
        "collection_label",
        "collection_count",
        "select_button",
        "remove_button",
    }
    missing_row_names = sorted(required_row_names - names(row))
    if missing_row_names:
        fail(f"collection row is missing controls: {missing_row_names}")

    required_source_fragments = [
        'COLLECTIONS_FILENAME = "inventory_explorer_collections.xml"',
        "LL_PATH_PER_SL_ACCOUNT",
        "mRecentPanel->setSinceLogoff(true)",
        "mWornPanel->setFilterWorn()",
        "hasFavoriteOrTrashAncestor",
        "doOnIdleOneTime",
    ]
    for fragment in required_source_fragments:
        if fragment not in source:
            fail(f"source wiring is missing: {fragment}")

    print("Inventory Explorer slice 3 wiring is consistent.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
