#!/usr/bin/env python3
"""Verify the Inventory Explorer toolbar, holding tray, and opt-in routing."""

from __future__ import annotations

import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NEWVIEW = ROOT / "indra/newview"
XUI = NEWVIEW / "skins/default/xui/en"


def fail(message: str) -> None:
    raise RuntimeError(message)


def parse_xml(path: Path) -> ET.Element:
    try:
        return ET.parse(path).getroot()
    except ET.ParseError as error:
        fail(f"invalid XML in {path.relative_to(ROOT)}: {error}")


def require_fragments(path: Path, fragments: list[str]) -> None:
    source = path.read_text(encoding="utf-8")
    for fragment in fragments:
        if fragment not in source:
            fail(f"{path.relative_to(ROOT)} is missing {fragment!r}")


def main() -> int:
    xml_paths = [
        XUI / "panel_al_inventory_explorer.xml",
        XUI / "panel_al_inventory_explorer_collection_row.xml",
        XUI / "panel_al_inventory_explorer_holding_tray.xml",
        XUI / "panel_al_inventory_explorer_holding_item.xml",
        XUI / "panel_al_inventory_explorer_inspector.xml",
        XUI / "menu_al_inventory_explorer_sort.xml",
        XUI / "menu_viewer.xml",
    ]
    roots = {path: parse_xml(path) for path in xml_paths}

    main_panel = roots[XUI / "panel_al_inventory_explorer.xml"]
    controls = {element.get("name"): element for element in main_panel.iter()}
    expected_tags = {
        "tree_view_button": "button",
        "list_view_button": "button",
        "grid_view_button": "button",
        "sort_button": "menu_button",
        "actions_button": "button",
        "create_button": "button",
        "inventory_explorer_search_editor": "filter_editor",
        "inventory_holding_tray": "panel",
    }
    for name, expected_tag in expected_tags.items():
        control = controls.get(name)
        if control is None or control.tag != expected_tag:
            fail(f"{name} must be a {expected_tag}")

    if main_panel.get("background_opaque") != "true":
        fail("Inventory Explorer root must obscure controls in floaters behind it")
    if "right" not in controls["builtin_collection_selection"].get(
        "follows", ""
    ).split("|"):
        fail("built-in collection selection must resize with the rail")
    for name in (
        "all_collection_icon",
        "recent_collection_icon",
        "worn_collection_icon",
        "favorites_collection_icon",
    ):
        if "right" in controls[name].get("follows", "").split("|"):
            fail(f"{name} must retain its native size while the rail resizes")
    for name in (
        "all_collection_count",
        "recent_collection_count",
        "worn_collection_count",
        "favorites_collection_count",
    ):
        if int(controls[name].get("width", "0")) < 60:
            fail(f"{name} is too narrow for six-digit inventory counts")
    for name in (
        "all_collection_label",
        "recent_collection_label",
        "worn_collection_label",
        "favorites_collection_label",
    ):
        if "right" not in controls[name].get("follows", "").split("|"):
            fail(f"{name} must resize between its icon and count")

    holding_tray = controls["inventory_holding_tray"]
    expected_holding_bounds = {
        "left": "0",
        "right": "-1",
        "top": "0",
        "bottom": "-1",
    }
    for attribute, expected in expected_holding_bounds.items():
        if holding_tray.get(attribute) != expected:
            fail(f"inventory_holding_tray must fill its parent ({attribute}={expected})")

    collection_row_root = roots[XUI / "panel_al_inventory_explorer_collection_row.xml"]
    collection_row_controls = {
        element.get("name"): element for element in collection_row_root.iter()
    }
    if int(collection_row_controls["collection_count"].get("width", "0")) < 54:
        fail("custom-filter count is too narrow")
    if int(collection_row_controls["collection_label"].get("right", "0")) > -80:
        fail("custom-filter label overlaps its count and remove button")

    inspector_root = roots[XUI / "panel_al_inventory_explorer_inspector.xml"]
    inspector_controls = {
        element.get("name"): element for element in inspector_root.iter()
    }
    if inspector_controls["inspector_empty_message"].get("word_wrap") != "true":
        fail("empty inspector message must wrap instead of clipping")

    for path, root in roots.items():
        for element in root.iter():
            for attribute, value in element.attrib.items():
                if "color" in attribute and value.startswith("#"):
                    fail(f"literal color added in {path.relative_to(ROOT)}: {value}")

    require_fragments(NEWVIEW / "alpanelinventoryexplorer.cpp", [
        "setFilterSubString(mSearchString)",
        "showContextMenuForSelection()",
        "mGalleryPanel->showContextMenu",
        'createFromFile<LLMenuGL>(\n        "menu_inventory_add.xml"',
        "getRootViewModel().startDrag(drag_items)",
        "LLToolDragAndDrop::instance().hasMouseCapture()",
        "mRailPanel->getRect().getHeight() - 35 - 30 * row",
    ])
    explorer_source = (NEWVIEW / "alpanelinventoryexplorer.cpp").read_text(
        encoding="utf-8"
    )
    if "-30 * row - rect.mBottom" in explorer_source:
        fail("collection selection applies its row offset twice")
    require_fragments(NEWVIEW / "alpanelinventoryholdingtray.cpp", [
        "mItemIDs.push_back(item_id)",
        "ACCEPT_YES_MULTI",
        "gInventory.getObject(item_id)",
        "setEndDragCallback",
    ])
    holding_source = (NEWVIEW / "alpanelinventoryholdingtray.cpp").read_text(encoding="utf-8")
    if "changeItemParent" in holding_source or "changeCategoryParent" in holding_source:
        fail("holding tray bypasses the existing inventory drag handlers")

    require_fragments(NEWVIEW / "alfloaterinventoryexplorer.cpp", [
        'gSavedPerAccountSettings.getBOOL("InventoryUseExplorer")',
        '"inventory_explorer"',
        '"inventory"',
        "toggleInstanceOrBringToFront(getPreferredInventoryFloater())",
    ])
    require_fragments(NEWVIEW / "alviewermenu.cpp", [
        '"Inventory.PreferredVisible"',
        '"Inventory.TogglePreferred"',
    ])

    settings_path = NEWVIEW / "app_settings/settings_per_account_alchemy.xml"
    settings_root = parse_xml(settings_path)
    settings_map = settings_root.find("map")
    if settings_map is None:
        fail("per-account settings file has no root map")
    children = list(settings_map)
    try:
        setting_index = next(
            index for index, child in enumerate(children)
            if child.tag == "key" and child.text == "InventoryUseExplorer"
        )
    except StopIteration:
        fail("InventoryUseExplorer setting is missing")
    setting_map = children[setting_index + 1]
    setting_values = {
        child.text: setting_map[index + 1].text
        for index, child in enumerate(setting_map)
        if child.tag == "key"
    }
    if setting_values.get("Type") != "Boolean" or setting_values.get("Value") != "0":
        fail("InventoryUseExplorer must be a default-off per-account Boolean")

    viewer_menu = roots[XUI / "menu_viewer.xml"]
    inventory = next(
        (element for element in viewer_menu.iter() if element.get("name") == "Inventory"),
        None,
    )
    if inventory is None:
        fail("normal Inventory menu item is missing")
    callbacks = {child.get("function") for child in inventory}
    if callbacks != {"Inventory.PreferredVisible", "Inventory.TogglePreferred"}:
        fail(f"normal Inventory menu item has unexpected callbacks: {sorted(callbacks)}")

    commands = parse_xml(NEWVIEW / "app_settings/commands.xml")
    inventory_command = next(
        (element for element in commands if element.get("name") == "inventory"),
        None,
    )
    if inventory_command is None:
        fail("inventory toolbar command is missing")
    if inventory_command.get("execute_function") != "Inventory.TogglePreferred":
        fail("inventory toolbar command bypasses preferred-inventory routing")
    if inventory_command.get("is_running_function") != "Inventory.PreferredVisible":
        fail("inventory toolbar state bypasses preferred-inventory routing")

    cmake = (NEWVIEW / "CMakeLists.txt").read_text(encoding="utf-8")
    for filename in ("alpanelinventoryholdingtray.cpp", "alpanelinventoryholdingtray.h"):
        if filename not in cmake:
            fail(f"CMakeLists.txt is missing {filename}")

    print("Inventory Explorer remaining-slice wiring is consistent.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
