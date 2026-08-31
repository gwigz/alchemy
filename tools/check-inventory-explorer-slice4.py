#!/usr/bin/env python3
"""Verify Inventory Explorer inspector and context-menu wiring."""

from __future__ import annotations

import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NEWVIEW = ROOT / "indra/newview"
XUI = NEWVIEW / "skins/default/xui/en"
MAIN_PANEL = XUI / "panel_al_inventory_explorer.xml"
INSPECTOR_PANEL = XUI / "panel_al_inventory_explorer_inspector.xml"
CLASSIC_MENU = XUI / "menu_gallery_inventory.xml"
EXPLORER_MENU = XUI / "menu_al_inventory_explorer.xml"
EXPLORER_MENU_NAME = "menu_al_inventory_explorer.xml"


def fail(message: str) -> None:
    raise RuntimeError(message)


def parse_xml(path: Path) -> ET.Element:
    try:
        return ET.parse(path).getroot()
    except ET.ParseError as error:
        fail(f"invalid XML in {path.relative_to(ROOT)}: {error}")


def named_children(root: ET.Element) -> dict[str, ET.Element]:
    result: dict[str, ET.Element] = {}
    for child in root:
        name = child.get("name")
        if not name:
            continue
        if name in result:
            fail(f"duplicate direct menu name: {name}")
        result[name] = child
    return result


def callback_signature(element: ET.Element) -> tuple[tuple[str, str, str], ...]:
    return tuple(
        (child.tag, child.get("function", ""), child.get("parameter", ""))
        for child in element
        if child.tag in {"menu_item_call.on_click", "on_click"}
    )


def main() -> int:
    subprocess.run(
        [sys.executable, str(ROOT / "tools/generate-inventory-explorer-menu.py"), "--check"],
        cwd=ROOT,
        check=True,
    )

    main_panel = parse_xml(MAIN_PANEL)
    inspector_panel = parse_xml(INSPECTOR_PANEL)
    classic_menu = parse_xml(CLASSIC_MENU)
    explorer_menu = parse_xml(EXPLORER_MENU)

    inventory_view_names = {
        "all_items_tree",
        "all_items_list",
        "all_items_grid",
        "recent_collection_view",
        "worn_collection_view",
        "favorites_collection_view",
        "type_filter_collection_view",
    }
    views = {
        element.get("name"): element
        for element in main_panel.iter()
        if element.get("name") in inventory_view_names
    }
    if set(views) != inventory_view_names:
        fail(f"missing inventory views: {sorted(inventory_view_names - set(views))}")
    for name, element in views.items():
        if element.get("context_menu") != EXPLORER_MENU_NAME:
            fail(f"{name} does not opt into the Inventory Explorer menu")

    inspector = next(
        (element for element in main_panel.iter() if element.get("name") == "inventory_inspector"),
        None,
    )
    if inspector is None:
        fail("main panel does not contain inventory_inspector")
    if inspector.get("class") != "panel_al_inventory_inspector":
        fail("inventory_inspector does not use ALPanelInventoryInspector")

    inspector_names = {element.get("name") for element in inspector_panel.iter()}
    required_inspector_controls = {
        "empty_panel",
        "details_scroll",
        "details_panel",
        "item_thumbnail",
        "item_type_icon",
        "item_name",
        "item_type",
        "item_creator",
        "item_created",
        "item_permissions",
        "item_attachment",
        "item_state",
    }
    if missing := required_inspector_controls - inspector_names:
        fail(f"inspector is missing controls: {sorted(missing)}")

    classic_items = named_children(classic_menu)
    explorer_items = named_children(explorer_menu)
    if set(classic_items) != set(explorer_items):
        missing = sorted(set(classic_items) - set(explorer_items))
        extra = sorted(set(explorer_items) - set(classic_items))
        fail(f"menu actions changed while regrouping; missing={missing}, extra={extra}")
    for name, classic_item in classic_items.items():
        if callback_signature(classic_item) != callback_signature(explorer_items[name]):
            fail(f"callback changed while regrouping menu item: {name}")

    menu_order = list(explorer_items)
    group_markers = ["Open", "Rename", "Share", "Copy Separator", "Paste Separator", "create_new"]
    marker_positions = [menu_order.index(marker) for marker in group_markers]
    if marker_positions != sorted(marker_positions):
        fail(f"menu groups are out of order: {group_markers}")

    explorer_source = (NEWVIEW / "alpanelinventoryexplorer.cpp").read_text(encoding="utf-8")
    inspector_source = (NEWVIEW / "alpanelinventoryinspector.cpp").read_text(encoding="utf-8")
    required_source_fragments = {
        explorer_source: [
            "updateInspectorSelection()",
            "mInspector->refreshObject()",
            "onInventorySelection",
            "onGallerySelection",
        ],
        inspector_source: [
            "getThumbnailUUID()",
            "getCreatorUUID()",
            "allowOperationBy",
            "getAttachedPointName",
            "get_is_item_worn",
            "RlvActions::canShowName",
        ],
    }
    for source, fragments in required_source_fragments.items():
        for fragment in fragments:
            if fragment not in source:
                fail(f"source wiring is missing: {fragment}")

    inventory_panel_header = (NEWVIEW / "llinventorypanel.h").read_text(encoding="utf-8")
    gallery_source = (NEWVIEW / "llinventorygallery.cpp").read_text(encoding="utf-8")
    if 'context_menu("context_menu", "menu_inventory.xml")' not in inventory_panel_header:
        fail("LLInventoryPanel classic context-menu default changed or is missing")
    if 'context_menu("context_menu", "menu_gallery_inventory.xml")' not in gallery_source:
        fail("LLInventoryGallery classic context-menu default changed or is missing")

    print("Inventory Explorer slice 4 wiring is consistent.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (RuntimeError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
