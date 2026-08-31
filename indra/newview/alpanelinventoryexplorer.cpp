/**
 * @file alpanelinventoryexplorer.cpp
 * @brief Responsive content shell for the opt-in Inventory Explorer floater
 *
 * $LicenseInfo:firstyear=2026&license=viewerlgpl$
 * Alchemy Viewer Source Code
 * Copyright (C) 2026, Alchemy Viewer Project.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation;
 * version 2.1 of the License only.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 * $/LicenseInfo$
 */

#include "llviewerprecompiledheaders.h"

#include "alpanelinventoryexplorer.h"

#include "alpanelinventoryholdingtray.h"
#include "alpanelinventoryinspector.h"

#include "llbutton.h"
#include "llcallbacklist.h"
#include "lldir.h"
#include "llfile.h"
#include "llflatlistview.h"
#include "llfiltereditor.h"
#include "llfolderviewmodelinventory.h"
#include "lliconctrl.h"
#include "llinventoryfilter.h"
#include "llinventoryfunctions.h"
#include "llinventorygallery.h"
#include "llinventorymodel.h"
#include "llinventorymodelbackgroundfetch.h"
#include "llinventorypanel.h"
#include "lllayoutstack.h"
#include "llmenugl.h"
#include "llpanelmaininventory.h"
#include "llsdserialize.h"
#include "lltextbox.h"
#include "lltooldraganddrop.h"
#include "lluicolortable.h"
#include "lluictrlfactory.h"
#include "llviewercontrol.h"
#include "llviewerinventory.h"
#include "llviewermenu.h"
#include "llwearabletype.h"
#include "llweb.h"

#include <algorithm>
#include <array>
#include <set>
#include <vector>

namespace
{
constexpr S32 COMPACT_BREAKPOINT = 700;
constexpr S32 COMPACT_ICON_LEFT = 15;
constexpr S32 WIDE_ICON_LEFT = 14;
constexpr const char* COLLECTIONS_FILENAME = "inventory_explorer_collections.xml";

constexpr std::array<const char*, 5> RAIL_ICONS{
    "all_collection_icon",
    "recent_collection_icon",
    "worn_collection_icon",
    "favorites_collection_icon",
    "add_filter_icon"
};

constexpr std::array<const char*, 10> WIDE_ONLY_CONTROLS{
    "all_collection_label",
    "all_collection_count",
    "recent_collection_label",
    "recent_collection_count",
    "worn_collection_label",
    "worn_collection_count",
    "favorites_collection_label",
    "favorites_collection_count",
    "my_filters_label",
    "add_filter_label"
};

constexpr U64 BODY_PART_MASK =
    (1ULL << LLWearableType::WT_SHAPE) |
    (1ULL << LLWearableType::WT_SKIN) |
    (1ULL << LLWearableType::WT_HAIR) |
    (1ULL << LLWearableType::WT_EYES);
constexpr U64 ALL_WEARABLE_MASK = (1ULL << LLWearableType::WT_COUNT) - 1;
constexpr U64 CLOTHING_MASK = ALL_WEARABLE_MASK & ~BODY_PART_MASK;

struct TypeFilterSpec
{
    const char* id;
    const char* label;
    const char* icon;
    U64 object_types;
    U64 wearable_types;
};

constexpr std::array<TypeFilterSpec, 10> TYPE_FILTER_SPECS{{
    { "clothing", "Clothing", "InvExplorer_Item_Clothing", 1ULL << LLInventoryType::IT_WEARABLE, CLOTHING_MASK },
    { "bodypart", "Body Parts", "InvExplorer_Item_BodyPart", 1ULL << LLInventoryType::IT_WEARABLE, BODY_PART_MASK },
    { "object", "Objects", "InvExplorer_Item_Object",
      (1ULL << LLInventoryType::IT_OBJECT) | (1ULL << LLInventoryType::IT_ATTACHMENT), ALL_WEARABLE_MASK },
    { "texture", "Textures", "InvExplorer_Item_Texture", 1ULL << LLInventoryType::IT_TEXTURE, ALL_WEARABLE_MASK },
    { "landmark", "Landmarks", "InvExplorer_Item_Landmark", 1ULL << LLInventoryType::IT_LANDMARK, ALL_WEARABLE_MASK },
    { "animation", "Animations", "InvExplorer_Item_Animation", 1ULL << LLInventoryType::IT_ANIMATION, ALL_WEARABLE_MASK },
    { "gesture", "Gestures", "InvExplorer_Item_Gesture", 1ULL << LLInventoryType::IT_GESTURE, ALL_WEARABLE_MASK },
    { "notecard", "Notecards", "InvExplorer_Item_Notecard", 1ULL << LLInventoryType::IT_NOTECARD, ALL_WEARABLE_MASK },
    { "sound", "Sounds", "InvExplorer_Item_Sound", 1ULL << LLInventoryType::IT_SOUND, ALL_WEARABLE_MASK },
    { "settings", "Settings", "InvExplorer_Item_Settings", 1ULL << LLInventoryType::IT_SETTINGS, ALL_WEARABLE_MASK }
}};

const TypeFilterSpec* findTypeFilterSpec(const std::string& id)
{
    const auto found = std::find_if(TYPE_FILTER_SPECS.begin(), TYPE_FILTER_SPECS.end(),
        [&id](const TypeFilterSpec& spec) { return id == spec.id; });
    return found == TYPE_FILTER_SPECS.end() ? nullptr : &*found;
}

LLInventoryFilter::Params makeTypeFilterParams(const TypeFilterSpec& spec)
{
    LLInventoryFilter::Params params;
    params.name = spec.id;
    params.filter_ops.object_types = spec.object_types;
    params.filter_ops.show_folder_state = LLInventoryFilter::SHOW_NON_EMPTY_FOLDERS;
    if (spec.object_types == (1ULL << LLInventoryType::IT_WEARABLE))
    {
        params.filter_ops.types = LLInventoryFilter::FILTERTYPE_OBJECT |
            LLInventoryFilter::FILTERTYPE_WEARABLE;
        params.filter_ops.wearable_types = spec.wearable_types;
    }
    return params;
}

bool matchesTypeFilter(const LLViewerInventoryItem& item, const TypeFilterSpec& spec)
{
    const LLInventoryType::EType inventory_type = item.getInventoryType();
    if (inventory_type < 0 || inventory_type >= 64 ||
        !(spec.object_types & (1ULL << inventory_type)))
    {
        return false;
    }

    if (inventory_type != LLInventoryType::IT_WEARABLE)
    {
        return true;
    }

    const LLWearableType::EType wearable_type = item.getWearableType();
    return wearable_type >= 0 && wearable_type < 64 &&
        (spec.wearable_types & (1ULL << wearable_type));
}

bool hasFavoriteOrTrashAncestor(const LLInventoryObject& object)
{
    LLUUID parent_id = object.getParentUUID();
    while (parent_id.notNull())
    {
        const LLViewerInventoryCategory* parent = gInventory.getCategory(parent_id);
        if (!parent)
        {
            return false;
        }
        if (parent->getPreferredType() == LLFolderType::FT_TRASH || parent->getIsFavorite())
        {
            return true;
        }
        parent_id = parent->getParentUUID();
    }
    return false;
}

std::string formatCount(S32 count)
{
    return llformat("%d", count);
}
}

static LLPanelInjector<ALPanelInventoryExplorer> t_panel_al_inventory_explorer(
    "panel_al_inventory_explorer");

ALPanelInventoryExplorer::ALPanelInventoryExplorer()
:   LLPanel()
{
    mCommitCallbackRegistrar.add("InventoryExplorer.AddTypeFilter",
        boost::bind(&ALPanelInventoryExplorer::onAddTypeFilter, this, _2));
    mEnableCallbackRegistrar.add("InventoryExplorer.CanAddTypeFilter",
        boost::bind(&ALPanelInventoryExplorer::canAddTypeFilter, this, _2));
    mCommitCallbackRegistrar.add("InventoryExplorer.Sort",
        boost::bind(&ALPanelInventoryExplorer::onSort, this, _2));
    mEnableCallbackRegistrar.add("InventoryExplorer.SortChecked",
        boost::bind(&ALPanelInventoryExplorer::isSortChecked, this, _2));
    mEnableCallbackRegistrar.add("InventoryExplorer.SortVisible",
        boost::bind(&ALPanelInventoryExplorer::isSortVisible, this, _2));
    mCommitCallbackRegistrar.add("Inventory.DoCreate",
        boost::bind(&ALPanelInventoryExplorer::onCreate, this, _2));
    mCommitCallbackRegistrar.add("Inventory.GearDefault.Custom.Action",
        boost::bind(&ALPanelInventoryExplorer::onCreateMenuAction, this, _2));
    mEnableCallbackRegistrar.add("Inventory.EnvironmentEnabled",
        [](LLUICtrl*, const LLSD&) { return LLPanelMainInventory::hasSettingsInventory(); });
    mEnableCallbackRegistrar.add("Inventory.MaterialsEnabled",
        [](LLUICtrl*, const LLSD&) { return LLPanelMainInventory::hasMaterialsInventory(); });
}

ALPanelInventoryExplorer::~ALPanelInventoryExplorer()
{
    gInventory.removeObserver(this);
    mListRootChangedConnection.disconnect();
    mGalleryRootChangedConnection.disconnect();
    mGallerySelectionConnection.disconnect();
}

bool ALPanelInventoryExplorer::postBuild()
{
    if (!LLPanel::postBuild())
    {
        return false;
    }

    mLayoutStack = getChild<LLLayoutStack>("inventory_explorer_layout_stack");
    mRailPanel = getChild<LLLayoutPanel>("collections_rail_layout_panel");
    mInspectorPanel = getChild<LLLayoutPanel>("inspector_layout_panel");
    mContentLayoutStack = getChild<LLLayoutStack>("content_layout_stack");
    mHoldingTrayLayoutPanel = getChild<LLLayoutPanel>("holding_tray_layout_panel");

    mTreePanel = getChild<LLInventoryPanel>("all_items_tree");
    mListPanel = getChild<LLInventorySingleFolderPanel>("all_items_list");
    mGalleryPanel = getChild<LLInventoryGallery>("all_items_grid");
    mRecentPanel = getChild<LLInventoryPanel>("recent_collection_view");
    mWornPanel = getChild<LLInventoryPanel>("worn_collection_view");
    mFavoritesPanel = getChild<LLInventoryPanel>("favorites_collection_view");
    mTypeFilterPanel = getChild<LLInventoryPanel>("type_filter_collection_view");
    mTypeFilterList = getChild<LLFlatListView>("type_filter_list");

    mBackButton = getChild<LLButton>("back_button");
    mForwardButton = getChild<LLButton>("forward_button");
    mUpButton = getChild<LLButton>("up_button");
    mTreeViewButton = getChild<LLButton>("tree_view_button");
    mListViewButton = getChild<LLButton>("list_view_button");
    mGridViewButton = getChild<LLButton>("grid_view_button");
    mCreateButton = getChild<LLButton>("create_button");
    mSearchEditor = getChild<LLFilterEditor>("inventory_explorer_search_editor");
    mStatusText = getChild<LLTextBox>("status_text");
    mInspector = dynamic_cast<ALPanelInventoryInspector*>(getChildView("inventory_inspector"));
    mHoldingTray = dynamic_cast<ALPanelInventoryHoldingTray*>(getChildView("inventory_holding_tray"));
    if (!mInspector || !mHoldingTray)
    {
        LL_WARNS("InventoryExplorer") << "Unable to create an Inventory Explorer child panel" << LL_ENDL;
        return false;
    }
    mHoldingTray->setDragStartCallback(boost::bind(
        &ALPanelInventoryExplorer::startHoldingItemDrag, this, _1));

    mTreePanel->getFilter().markDefault();
    mTreePanel->setSelectCallback(boost::bind(
        &ALPanelInventoryExplorer::onInventorySelection, this, mTreePanel, _1, _2));
    mTreePanel->initializeViewBuilding();

    mListPanel->initFolderRoot();
    mListPanel->setSortOrder(gSavedSettings.getU32(LLInventoryPanel::DEFAULT_SORT_ORDER));
    mListPanel->getFilter().setFilterThumbnails(LLInventoryFilter::FILTER_INCLUDE_THUMBNAILS);
    mListPanel->getFilter().markDefault();
    mListPanel->setSelectCallback(boost::bind(
        &ALPanelInventoryExplorer::onInventorySelection, this, mListPanel, _1, _2));

    mGalleryPanel->setSortOrder(mListPanel->getSortOrder());
    mGalleryPanel->getFilter().setFilterThumbnails(LLInventoryFilter::FILTER_INCLUDE_THUMBNAILS);
    mGalleryPanel->getFilter().markDefault();
    mGalleryPanel->setRootFolder(mListPanel->getSingleFolderRoot());

    mListRootChangedConnection = mListPanel->setRootChangedCallback(
        boost::bind(&ALPanelInventoryExplorer::onRootChanged, this));
    mGalleryRootChangedConnection = mGalleryPanel->setRootChangedCallback(
        boost::bind(&ALPanelInventoryExplorer::onRootChanged, this));
    mGallerySelectionConnection = mGalleryPanel->setSelectionChangeCallback(
        boost::bind(&ALPanelInventoryExplorer::onGallerySelection, this, _1));

    mRecentPanel->setSinceLogoff(true);
    mRecentPanel->setSortOrder(LLInventoryFilter::SO_DATE);
    mRecentPanel->setShowFolderState(LLInventoryFilter::SHOW_NON_EMPTY_FOLDERS);
    LLInventoryFilter& recent_filter = mRecentPanel->getFilter();
    recent_filter.setFilterObjectTypes(recent_filter.getFilterObjectTypes() &
        ~(1ULL << LLInventoryType::IT_CATEGORY));
    recent_filter.setEmptyLookupMessage("InventoryNoMatchingRecentItems");
    recent_filter.markDefault();
    mRecentPanel->setSelectCallback(boost::bind(
        &ALPanelInventoryExplorer::onInventorySelection, this, mRecentPanel, _1, _2));
    mRecentPanel->initializeViewBuilding();

    U64 worn_types = (1ULL << LLInventoryType::IT_WEARABLE) |
        (1ULL << LLInventoryType::IT_ATTACHMENT) |
        (1ULL << LLInventoryType::IT_OBJECT);
    mWornPanel->setFilterTypes(worn_types);
    mWornPanel->setFilterWorn();
    mWornPanel->setShowFolderState(LLInventoryFilter::SHOW_NON_EMPTY_FOLDERS);
    mWornPanel->setFilterLinks(LLInventoryFilter::FILTERLINK_EXCLUDE_LINKS);
    LLInventoryFilter& worn_filter = mWornPanel->getFilter();
    worn_filter.setFilterCategoryTypes(worn_filter.getFilterCategoryTypes() |
        (1ULL << LLFolderType::FT_INBOX));
    worn_filter.markDefault();
    mWornPanel->setSelectCallback(boost::bind(
        &ALPanelInventoryExplorer::onInventorySelection, this, mWornPanel, _1, _2));
    mWornPanel->initializeViewBuilding();

    mFavoritesPanel->setSortOrder(gSavedSettings.getU32(LLInventoryPanel::DEFAULT_SORT_ORDER));
    LLInventoryFilter& favorites_filter = mFavoritesPanel->getFilter();
    favorites_filter.setEmptyLookupMessage("InventoryNoMatchingFavorites");
    favorites_filter.markDefault();
    mFavoritesPanel->setSelectCallback(boost::bind(
        &ALPanelInventoryExplorer::onInventorySelection, this, mFavoritesPanel, _1, _2));
    mFavoritesPanel->initializeViewBuilding();

    mTypeFilterPanel->setSortOrder(gSavedSettings.getU32(LLInventoryPanel::DEFAULT_SORT_ORDER));
    mTypeFilterPanel->setSelectCallback(boost::bind(
        &ALPanelInventoryExplorer::onInventorySelection, this, mTypeFilterPanel, _1, _2));
    mTypeFilterPanel->initializeViewBuilding();

    mBackButton->setCommitCallback(boost::bind(&ALPanelInventoryExplorer::onBack, this));
    mForwardButton->setCommitCallback(boost::bind(&ALPanelInventoryExplorer::onForward, this));
    mUpButton->setCommitCallback(boost::bind(&ALPanelInventoryExplorer::onUp, this));
    mTreeViewButton->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::setViewMode, this, EViewMode::TREE));
    mListViewButton->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::setViewMode, this, EViewMode::LIST));
    mGridViewButton->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::setViewMode, this, EViewMode::GRID));
    getChild<LLButton>("actions_button")->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::onActions, this));
    mCreateButton->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::onCreateButton, this));
    mSearchEditor->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::onSearch, this, _2));

    mCommitCallbackRegistrar.pushScope();
    mEnableCallbackRegistrar.pushScope();
    LLMenuGL* create_menu = LLUICtrlFactory::getInstance()->createFromFile<LLMenuGL>(
        "menu_inventory_add.xml", gMenuHolder,
        LLViewerMenuHolderGL::child_registry_t::instance());
    mEnableCallbackRegistrar.popScope();
    mCommitCallbackRegistrar.popScope();
    if (create_menu)
    {
        mCreateMenuHandle = create_menu->getHandle();
    }

    getChild<LLButton>("all_collection_button")->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::selectBuiltinCollection, this, EBuiltinCollection::ALL_ITEMS));
    getChild<LLButton>("recent_collection_button")->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::selectBuiltinCollection, this, EBuiltinCollection::RECENT));
    getChild<LLButton>("worn_collection_button")->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::selectBuiltinCollection, this, EBuiltinCollection::WORN));
    getChild<LLButton>("favorites_collection_button")->setCommitCallback(boost::bind(
        &ALPanelInventoryExplorer::selectBuiltinCollection, this, EBuiltinCollection::FAVORITES));

    mTypeFilterList->setAllowSelection(false);
    loadTypeFilters();
    rebuildTypeFilterRows();
    gInventory.addObserver(this);

    updateViewVisibility();
    updateNavigationButtons();
    updateStatusText();
    applyLayout(getRect().getWidth());
    updateHoldingTrayVisibility();
    scheduleCountRefresh();
    return true;
}

void ALPanelInventoryExplorer::draw()
{
    updateHoldingTrayVisibility();
    LLPanel::draw();
}

void ALPanelInventoryExplorer::changed(U32 mask)
{
    if (mask != LLInventoryObserver::NONE)
    {
        mInspector->refreshObject();
        scheduleCountRefresh();
    }
}

void ALPanelInventoryExplorer::reshape(S32 width, S32 height, bool called_from_parent)
{
    LLPanel::reshape(width, height, called_from_parent);
    if (mLayoutStack)
    {
        applyLayout(width);
    }
}

void ALPanelInventoryExplorer::applyLayout(S32 width)
{
    const ELayoutState next_state = width < COMPACT_BREAKPOINT
        ? ELayoutState::COMPACT
        : ELayoutState::WIDE;
    if (mLayoutState == next_state)
    {
        return;
    }

    const bool compact = next_state == ELayoutState::COMPACT;
    const S32 icon_left = compact ? COMPACT_ICON_LEFT : WIDE_ICON_LEFT;

    mLayoutStack->collapsePanel(mRailPanel, compact);
    mInspectorPanel->setVisible(!compact);

    for (const char* control_name : WIDE_ONLY_CONTROLS)
    {
        getChildView(control_name)->setVisible(!compact);
    }

    for (const char* control_name : RAIL_ICONS)
    {
        LLView* control = getChildView(control_name);
        control->setOrigin(icon_left, control->getRect().mBottom);
    }

    for (const auto& [type_id, row] : mTypeFilterRows)
    {
        row->getChildView("collection_label")->setVisible(!compact);
        row->getChildView("collection_count")->setVisible(!compact);
        row->getChildView("remove_button")->setVisible(!compact);
        LLView* icon = row->getChildView("collection_icon");
        icon->setOrigin(compact ? 9 : 8, icon->getRect().mBottom);
    }

    LLView* divider = getChildView("my_filters_divider");
    LLRect divider_rect = divider->getRect();
    divider_rect.mLeft = compact ? 9 : 87;
    divider_rect.mRight = compact ? 37 : 174;
    divider->setRect(divider_rect);

    mLayoutState = next_state;
    mLayoutStack->updateLayout();
}

void ALPanelInventoryExplorer::selectBuiltinCollection(EBuiltinCollection collection)
{
    if (mActiveCollection == ActiveCollection(collection))
    {
        return;
    }

    mActiveCollection = collection;
    mSelectedItemID.setNull();
    updateInspectorSelection();
    updateViewVisibility();
    updateCollectionSelection();
    updateNavigationButtons();
    updateStatusText();
    scheduleCountRefresh();
}

void ALPanelInventoryExplorer::selectTypeCollection(const std::string& type_id)
{
    const TypeFilterSpec* spec = findTypeFilterSpec(type_id);
    if (!spec || std::find(mTypeFilterIDs.begin(), mTypeFilterIDs.end(), type_id) == mTypeFilterIDs.end())
    {
        return;
    }

    mActiveCollection = type_id;
    mSelectedItemID.setNull();
    updateInspectorSelection();
    mTypeFilterPanel->getFilter().fromParams(makeTypeFilterParams(*spec));
    mTypeFilterPanel->getFilter().markDefault();
    mTypeFilterPanel->setFilterSubString(mSearchString);
    updateViewVisibility();
    updateCollectionSelection();
    updateNavigationButtons();
    updateStatusText();
    scheduleCountRefresh();
}

void ALPanelInventoryExplorer::onAddTypeFilter(const LLSD& data)
{
    const std::string type_id = data.asString();
    if (!canAddTypeFilter(data))
    {
        return;
    }

    mTypeFilterIDs.push_back(type_id);
    saveTypeFilters();
    rebuildTypeFilterRows();
    selectTypeCollection(type_id);
    scheduleCountRefresh();
}

bool ALPanelInventoryExplorer::canAddTypeFilter(const LLSD& data) const
{
    const std::string type_id = data.asString();
    return findTypeFilterSpec(type_id) &&
        std::find(mTypeFilterIDs.begin(), mTypeFilterIDs.end(), type_id) == mTypeFilterIDs.end();
}

void ALPanelInventoryExplorer::removeTypeFilter(const std::string& type_id)
{
    const auto found = std::find(mTypeFilterIDs.begin(), mTypeFilterIDs.end(), type_id);
    if (found == mTypeFilterIDs.end())
    {
        return;
    }

    const bool removed_active = std::holds_alternative<std::string>(mActiveCollection) &&
        std::get<std::string>(mActiveCollection) == type_id;
    mTypeFilterIDs.erase(found);
    saveTypeFilters();
    rebuildTypeFilterRows();
    if (removed_active)
    {
        mActiveCollection = EBuiltinCollection::ALL_ITEMS;
        mSelectedItemID.setNull();
        updateInspectorSelection();
        updateViewVisibility();
        updateNavigationButtons();
        updateStatusText();
        scheduleCountRefresh();
    }
    updateCollectionSelection();
}

bool ALPanelInventoryExplorer::isAllItemsCollection() const
{
    const EBuiltinCollection* builtin = std::get_if<EBuiltinCollection>(&mActiveCollection);
    return builtin && *builtin == EBuiltinCollection::ALL_ITEMS;
}

LLInventoryPanel* ALPanelInventoryExplorer::getActiveInventoryPanel() const
{
    if (const EBuiltinCollection* builtin = std::get_if<EBuiltinCollection>(&mActiveCollection))
    {
        switch (*builtin)
        {
            case EBuiltinCollection::ALL_ITEMS: return mTreePanel;
            case EBuiltinCollection::RECENT: return mRecentPanel;
            case EBuiltinCollection::WORN: return mWornPanel;
            case EBuiltinCollection::FAVORITES: return mFavoritesPanel;
        }
    }
    return mTypeFilterPanel;
}

std::string ALPanelInventoryExplorer::getActiveCollectionLabel() const
{
    if (const EBuiltinCollection* builtin = std::get_if<EBuiltinCollection>(&mActiveCollection))
    {
        switch (*builtin)
        {
            case EBuiltinCollection::ALL_ITEMS: return "All Items";
            case EBuiltinCollection::RECENT: return "Recent";
            case EBuiltinCollection::WORN: return "Worn";
            case EBuiltinCollection::FAVORITES: return "Favorites";
        }
    }

    const TypeFilterSpec* spec = findTypeFilterSpec(std::get<std::string>(mActiveCollection));
    return spec ? spec->label : "All Items";
}

void ALPanelInventoryExplorer::updateCollectionSelection()
{
    LLView* builtin_selection = getChildView("builtin_collection_selection");
    builtin_selection->setVisible(std::holds_alternative<EBuiltinCollection>(mActiveCollection));
    if (const EBuiltinCollection* builtin = std::get_if<EBuiltinCollection>(&mActiveCollection))
    {
        S32 row = 0;
        switch (*builtin)
        {
            case EBuiltinCollection::ALL_ITEMS: row = 0; break;
            case EBuiltinCollection::RECENT: row = 1; break;
            case EBuiltinCollection::WORN: row = 2; break;
            case EBuiltinCollection::FAVORITES: row = 3; break;
        }
        LLRect rect = builtin_selection->getRect();
        rect.translate(0, -rect.mBottom + (mRailPanel->getRect().getHeight() - 35 - 30 * row));
        builtin_selection->setRect(rect);
    }

    constexpr std::array<const char*, 4> builtin_labels{
        "all_collection_label",
        "recent_collection_label",
        "worn_collection_label",
        "favorites_collection_label"
    };
    S32 active_builtin = -1;
    if (const EBuiltinCollection* builtin = std::get_if<EBuiltinCollection>(&mActiveCollection))
    {
        active_builtin = static_cast<S32>(*builtin);
    }
    for (S32 index = 0; index < static_cast<S32>(builtin_labels.size()); ++index)
    {
        getChild<LLTextBox>(builtin_labels[index])->setColor(
            LLUIColorTable::instance().getColor(index == active_builtin ? "White" : "LabelTextColor"));
    }

    for (const auto& [type_id, row] : mTypeFilterRows)
    {
        const bool selected = std::holds_alternative<std::string>(mActiveCollection) &&
            std::get<std::string>(mActiveCollection) == type_id;
        row->getChildView("selection_background")->setVisible(selected);
        row->getChild<LLTextBox>("collection_label")->setColor(
            selected ? LLUIColorTable::instance().getColor("White")
                     : LLUIColorTable::instance().getColor("LabelTextColor"));
    }
}

void ALPanelInventoryExplorer::loadTypeFilters()
{
    mTypeFilterIDs.clear();
    const std::string filename = gDirUtilp->getExpandedFilename(
        LL_PATH_PER_SL_ACCOUNT, COLLECTIONS_FILENAME);
    llifstream input(filename.c_str());
    LLSD saved;
    const bool loaded = input.is_open() &&
        LLSDParser::PARSE_FAILURE != LLSDSerialize::fromXML(saved, input) &&
        saved.has("type_filters") && saved["type_filters"].isArray();

    std::set<std::string> seen;
    if (loaded)
    {
        for (LLSD::array_const_iterator it = saved["type_filters"].beginArray();
             it != saved["type_filters"].endArray(); ++it)
        {
            const std::string type_id = it->asString();
            if (findTypeFilterSpec(type_id) && seen.insert(type_id).second)
            {
                mTypeFilterIDs.push_back(type_id);
            }
        }
    }
    else
    {
        mTypeFilterIDs = { "landmark", "texture" };
    }
}

void ALPanelInventoryExplorer::saveTypeFilters() const
{
    LLSD saved = LLSD::emptyMap();
    saved["version"] = 1;
    saved["type_filters"] = LLSD::emptyArray();
    for (const std::string& type_id : mTypeFilterIDs)
    {
        saved["type_filters"].append(type_id);
    }

    const std::string filename = gDirUtilp->getExpandedFilename(
        LL_PATH_PER_SL_ACCOUNT, COLLECTIONS_FILENAME);
    llofstream output(filename.c_str());
    if (!output.is_open() || !LLSDSerialize::toPrettyXML(saved, output))
    {
        LL_WARNS("InventoryExplorer") << "Unable to save type filters to " << filename << LL_ENDL;
    }
}

void ALPanelInventoryExplorer::rebuildTypeFilterRows()
{
    mTypeFilterList->clear();
    mTypeFilterRows.clear();
    for (const std::string& type_id : mTypeFilterIDs)
    {
        LLPanel* row = LLUICtrlFactory::getInstance()->createFromFile<LLPanel>(
            "panel_al_inventory_explorer_collection_row.xml", nullptr,
            LLPanel::child_registry_t::instance());
        if (!row)
        {
            LL_WARNS("InventoryExplorer") << "Unable to create row for " << type_id << LL_ENDL;
            continue;
        }
        updateTypeFilterRow(row, type_id);
        row->getChild<LLButton>("select_button")->setCommitCallback(boost::bind(
            &ALPanelInventoryExplorer::selectTypeCollection, this, type_id));
        row->getChild<LLButton>("remove_button")->setCommitCallback(boost::bind(
            &ALPanelInventoryExplorer::removeTypeFilter, this, type_id));
        mTypeFilterList->addItem(row, type_id, ADD_BOTTOM);
        mTypeFilterRows.emplace(type_id, row);
    }
    updateCollectionSelection();

    if (mLayoutState != ELayoutState::UNINITIALIZED)
    {
        const ELayoutState old_state = mLayoutState;
        mLayoutState = ELayoutState::UNINITIALIZED;
        applyLayout(old_state == ELayoutState::COMPACT ? COMPACT_BREAKPOINT - 1 : COMPACT_BREAKPOINT);
    }
}

void ALPanelInventoryExplorer::updateTypeFilterRow(LLPanel* row, const std::string& type_id)
{
    const TypeFilterSpec* spec = findTypeFilterSpec(type_id);
    if (!spec)
    {
        return;
    }
    row->getChild<LLIconCtrl>("collection_icon")->setValue(spec->icon);
    row->getChild<LLTextBox>("collection_label")->setText(std::string(spec->label));
    row->getChild<LLButton>("remove_button")->setToolTip(llformat("Remove %s filter", spec->label));
}

void ALPanelInventoryExplorer::scheduleCountRefresh()
{
    if (mCountRefreshPending || !mTypeFilterList)
    {
        return;
    }
    mCountRefreshPending = true;
    doOnIdleOneTime(boost::bind(
        &ALPanelInventoryExplorer::onIdleCountRefresh, getHandle()));
}

void ALPanelInventoryExplorer::onIdleCountRefresh(LLHandle<LLPanel> panel_handle)
{
    if (panel_handle.isDead())
    {
        return;
    }
    ALPanelInventoryExplorer* panel =
        static_cast<ALPanelInventoryExplorer*>(panel_handle.get());
    panel->mCountRefreshPending = false;
    panel->refreshCollectionCounts();
}

void ALPanelInventoryExplorer::refreshCollectionCounts()
{
    if (LLInventoryModelBackgroundFetch::instance().folderFetchActive())
    {
        constexpr std::array<const char*, 4> count_controls{
            "all_collection_count",
            "recent_collection_count",
            "worn_collection_count",
            "favorites_collection_count"
        };
        for (const char* control_name : count_controls)
        {
            getChild<LLTextBox>(control_name)->setText(std::string("..."));
        }
        getChild<LLTextBox>("active_collection_count")->setText(std::string("..."));
        for (const auto& [type_id, row] : mTypeFilterRows)
        {
            row->getChild<LLTextBox>("collection_count")->setText(std::string("..."));
        }
        scheduleCountRefresh();
        return;
    }

    LLInventoryModel::cat_array_t categories;
    LLInventoryModel::item_array_t items;
    gInventory.collectDescendents(gInventory.getRootFolderID(), categories, items, true);

    S32 recent_count = 0;
    S32 worn_count = 0;
    std::map<std::string, S32> type_counts;
    for (const TypeFilterSpec& spec : TYPE_FILTER_SPECS)
    {
        type_counts.emplace(spec.id, 0);
    }

    const time_t recent_min = mRecentPanel->getFilter().getMinDate();
    const time_t recent_max = mRecentPanel->getFilter().getMaxDate();
    for (const LLPointer<LLViewerInventoryItem>& item : items)
    {
        if (item->getCreationDate() >= recent_min && item->getCreationDate() <= recent_max)
        {
            ++recent_count;
        }
        const LLInventoryType::EType inventory_type = item->getInventoryType();
        const bool is_worn_collection_type =
            inventory_type == LLInventoryType::IT_WEARABLE ||
            inventory_type == LLInventoryType::IT_ATTACHMENT ||
            inventory_type == LLInventoryType::IT_OBJECT;
        if (!item->getIsLinkType() && is_worn_collection_type && get_is_item_worn(item.get()))
        {
            ++worn_count;
        }
        for (const TypeFilterSpec& spec : TYPE_FILTER_SPECS)
        {
            if (matchesTypeFilter(*item, spec))
            {
                ++type_counts[spec.id];
            }
        }
    }

    S32 favorites_count = 0;
    for (const LLPointer<LLViewerInventoryCategory>& category : categories)
    {
        if (category->getPreferredType() != LLFolderType::FT_TRASH &&
            category->getIsFavorite() && !hasFavoriteOrTrashAncestor(*category))
        {
            ++favorites_count;
        }
    }
    for (const LLPointer<LLViewerInventoryItem>& item : items)
    {
        if (item->getIsFavorite() && !hasFavoriteOrTrashAncestor(*item))
        {
            ++favorites_count;
        }
    }

    const std::string all_count = formatCount(gInventory.getItemCount());
    const std::string recent = formatCount(recent_count);
    const std::string worn = formatCount(worn_count);
    const std::string favorites = formatCount(favorites_count);
    getChild<LLTextBox>("all_collection_count")->setText(all_count);
    getChild<LLTextBox>("recent_collection_count")->setText(recent);
    getChild<LLTextBox>("worn_collection_count")->setText(worn);
    getChild<LLTextBox>("favorites_collection_count")->setText(favorites);

    for (const auto& [type_id, row] : mTypeFilterRows)
    {
        row->getChild<LLTextBox>("collection_count")->setText(formatCount(type_counts[type_id]));
    }

    std::string active_count = all_count;
    if (const EBuiltinCollection* builtin = std::get_if<EBuiltinCollection>(&mActiveCollection))
    {
        switch (*builtin)
        {
            case EBuiltinCollection::ALL_ITEMS: active_count = all_count; break;
            case EBuiltinCollection::RECENT: active_count = recent; break;
            case EBuiltinCollection::WORN: active_count = worn; break;
            case EBuiltinCollection::FAVORITES: active_count = favorites; break;
        }
    }
    else
    {
        active_count = formatCount(type_counts[std::get<std::string>(mActiveCollection)]);
    }
    getChild<LLTextBox>("active_collection_count")->setText(active_count);
}

void ALPanelInventoryExplorer::setViewMode(EViewMode mode)
{
    if (!isAllItemsCollection())
    {
        return;
    }

    if (mode == mViewMode)
    {
        updateViewVisibility();
        return;
    }

    SingleFolderState state;
    if (mViewMode == EViewMode::TREE)
    {
        state = makeSingleFolderStateFromTree();
    }
    else
    {
        state = captureSingleFolderState();
        if (mode == EViewMode::TREE && mSelectedItemID.isNull())
        {
            mSelectedItemID = state.root;
        }
    }

    mViewMode = mode;
    if (mViewMode != EViewMode::TREE)
    {
        applySingleFolderState(state);
        if (!selectionBelongsToRoot(mSelectedItemID, state.root))
        {
            mSelectedItemID.setNull();
        }
    }

    updateViewVisibility();
    restoreSelection();
    updateInspectorSelection();
    updateNavigationButtons();
    updateStatusText();
}

void ALPanelInventoryExplorer::updateViewVisibility()
{
    const bool all_items = isAllItemsCollection();
    const bool tree_visible = all_items && mViewMode == EViewMode::TREE;
    const bool list_visible = all_items && mViewMode == EViewMode::LIST;
    const bool grid_visible = all_items && mViewMode == EViewMode::GRID;

    mTreePanel->setVisible(tree_visible);
    mListPanel->setVisible(list_visible);
    mGalleryPanel->setVisible(grid_visible);
    mRecentPanel->setVisible(!all_items && getActiveInventoryPanel() == mRecentPanel);
    mWornPanel->setVisible(!all_items && getActiveInventoryPanel() == mWornPanel);
    mFavoritesPanel->setVisible(!all_items && getActiveInventoryPanel() == mFavoritesPanel);
    mTypeFilterPanel->setVisible(!all_items && getActiveInventoryPanel() == mTypeFilterPanel);

    getChild<LLTextBox>("active_collection_label")->setText(getActiveCollectionLabel());
    getChildView("active_collection_label")->setVisible(!all_items || tree_visible);
    getChildView("active_collection_count")->setVisible(!all_items || tree_visible);
    mBackButton->setVisible(all_items && !tree_visible);
    mForwardButton->setVisible(all_items && !tree_visible);
    mUpButton->setVisible(all_items && !tree_visible);

    mTreeViewButton->setToggleState(!all_items || tree_visible);
    mListViewButton->setToggleState(list_visible);
    mGridViewButton->setToggleState(grid_visible);
    mListViewButton->setEnabled(all_items);
    mGridViewButton->setEnabled(all_items);
}

ALPanelInventoryExplorer::SingleFolderState
ALPanelInventoryExplorer::makeSingleFolderStateFromTree()
{
    SingleFolderState state;
    state.root = gInventory.getRootFolderID();
    state.sort_order = mListPanel->getSortOrder();

    if (mSelectedItemID.isNull())
    {
        return state;
    }

    if (gInventory.getCategory(mSelectedItemID))
    {
        state.root = mSelectedItemID;
        mSelectedItemID.setNull();
    }
    else if (const LLViewerInventoryItem* item = gInventory.getItem(mSelectedItemID))
    {
        state.root = item->getParentUUID();
    }
    else
    {
        mSelectedItemID.setNull();
    }

    return state;
}

ALPanelInventoryExplorer::SingleFolderState
ALPanelInventoryExplorer::captureSingleFolderState() const
{
    SingleFolderState state;
    if (mViewMode == EViewMode::LIST)
    {
        state.root = mListPanel->getSingleFolderRoot();
        state.backward = mListPanel->getNavBackwardList();
        state.forward = mListPanel->getNavForwardList();
        state.sort_order = mListPanel->getSortOrder();
    }
    else if (mViewMode == EViewMode::GRID)
    {
        state.root = mGalleryPanel->getRootFolder();
        state.backward = mGalleryPanel->getNavBackwardList();
        state.forward = mGalleryPanel->getNavForwardList();
        state.sort_order = mGalleryPanel->getSortOrder();
    }

    return state;
}

void ALPanelInventoryExplorer::applySingleFolderState(const SingleFolderState& state)
{
    if (state.root.isNull())
    {
        return;
    }

    if (mViewMode == EViewMode::LIST)
    {
        mListPanel->changeFolderRoot(state.root);
        mListPanel->setNavBackwardList(state.backward);
        mListPanel->setNavForwardList(state.forward);
        mListPanel->setSortOrder(state.sort_order);
    }
    else if (mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->setRootFolder(state.root);
        mGalleryPanel->setNavBackwardList(state.backward);
        mGalleryPanel->setNavForwardList(state.forward);
        mGalleryPanel->setSortOrder(state.sort_order, true);
    }
}

LLUUID ALPanelInventoryExplorer::getCurrentSingleFolderRoot() const
{
    if (mViewMode == EViewMode::LIST)
    {
        return mListPanel->getSingleFolderRoot();
    }
    if (mViewMode == EViewMode::GRID)
    {
        return mGalleryPanel->getRootFolder();
    }
    return LLUUID::null;
}

void ALPanelInventoryExplorer::onInventorySelection(
    LLInventoryPanel* panel,
    const std::deque<LLFolderViewItem*>& items,
    bool user_action)
{
    panel->onSelectionChange(items, user_action);
    if (mRestoringSelection)
    {
        return;
    }

    mSelectedItemID.setNull();
    if (!items.empty())
    {
        const LLFolderViewModelItemInventory* model_item =
            static_cast<const LLFolderViewModelItemInventory*>(items.back()->getViewModelItem());
        if (model_item)
        {
            mSelectedItemID = model_item->getUUID();
        }
    }
    updateInspectorSelection();
    updateStatusText();
}

void ALPanelInventoryExplorer::onGallerySelection(const LLUUID& item_id)
{
    if (!mRestoringSelection)
    {
        mSelectedItemID = item_id;
        updateInspectorSelection();
        updateStatusText();
    }
}

void ALPanelInventoryExplorer::updateInspectorSelection()
{
    mInspector->setObjectID(mSelectedItemID);
}

void ALPanelInventoryExplorer::restoreSelection()
{
    if (mSelectedItemID.isNull())
    {
        return;
    }

    mRestoringSelection = true;
    if (!isAllItemsCollection())
    {
        getActiveInventoryPanel()->setSelection(mSelectedItemID, TAKE_FOCUS_NO);
    }
    else if (mViewMode == EViewMode::TREE)
    {
        mTreePanel->setSelection(mSelectedItemID, TAKE_FOCUS_NO);
    }
    else if (mViewMode == EViewMode::LIST)
    {
        mListPanel->setSelection(mSelectedItemID, TAKE_FOCUS_NO);
    }
    else if (mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->changeItemSelection(mSelectedItemID, true);
    }
    mRestoringSelection = false;
}

bool ALPanelInventoryExplorer::selectionBelongsToRoot(
    const LLUUID& item_id,
    const LLUUID& root_id) const
{
    const LLInventoryObject* object = gInventory.getObject(item_id);
    return object && object->getParentUUID() == root_id;
}

void ALPanelInventoryExplorer::onRootChanged()
{
    const LLUUID root_id = getCurrentSingleFolderRoot();
    if (!selectionBelongsToRoot(mSelectedItemID, root_id))
    {
        mSelectedItemID.setNull();
    }
    updateInspectorSelection();
    updateNavigationButtons();
    updateStatusText();
}

void ALPanelInventoryExplorer::onBack()
{
    if (mViewMode == EViewMode::LIST)
    {
        mListPanel->onBackwardFolder();
    }
    else if (mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->onBackwardFolder();
    }
}

void ALPanelInventoryExplorer::onForward()
{
    if (mViewMode == EViewMode::LIST)
    {
        mListPanel->onForwardFolder();
    }
    else if (mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->onForwardFolder();
    }
}

void ALPanelInventoryExplorer::onUp()
{
    const LLViewerInventoryCategory* category =
        gInventory.getCategory(getCurrentSingleFolderRoot());
    if (!category || category->getParentUUID().isNull())
    {
        return;
    }

    if (mViewMode == EViewMode::LIST)
    {
        mListPanel->changeFolderRoot(category->getParentUUID());
    }
    else if (mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->setRootFolder(category->getParentUUID());
    }
}

void ALPanelInventoryExplorer::updateNavigationButtons()
{
    if (!isAllItemsCollection())
    {
        mBackButton->setEnabled(false);
        mForwardButton->setEnabled(false);
        mUpButton->setEnabled(false);
        return;
    }

    bool back_enabled = false;
    bool forward_enabled = false;
    if (mViewMode == EViewMode::LIST)
    {
        back_enabled = mListPanel->isBackwardAvailable();
        forward_enabled = mListPanel->isForwardAvailable();
    }
    else if (mViewMode == EViewMode::GRID)
    {
        back_enabled = mGalleryPanel->isBackwardAvailable();
        forward_enabled = mGalleryPanel->isForwardAvailable();
    }

    mBackButton->setEnabled(back_enabled);
    mForwardButton->setEnabled(forward_enabled);

    const LLViewerInventoryCategory* category =
        gInventory.getCategory(getCurrentSingleFolderRoot());
    mUpButton->setEnabled(category && category->getParentUUID().notNull());
}

void ALPanelInventoryExplorer::updateStatusText()
{
    if (!isAllItemsCollection() || mViewMode == EViewMode::TREE)
    {
        const LLInventoryObject* object = gInventory.getObject(mSelectedItemID);
        mStatusText->setText(object ? object->getName() : getActiveCollectionLabel());
        return;
    }

    std::vector<std::string> path;
    LLUUID folder_id = getCurrentSingleFolderRoot();
    while (folder_id.notNull())
    {
        const LLViewerInventoryCategory* category = gInventory.getCategory(folder_id);
        if (!category)
        {
            break;
        }

        path.push_back(get_localized_folder_name(folder_id));
        if (folder_id == gInventory.getRootFolderID())
        {
            break;
        }
        folder_id = category->getParentUUID();
    }

    std::reverse(path.begin(), path.end());
    std::string breadcrumb;
    for (const std::string& folder_name : path)
    {
        if (!breadcrumb.empty())
        {
            breadcrumb += " > ";
        }
        breadcrumb += folder_name;
    }
    mStatusText->setText(breadcrumb.empty() ? "All Items" : breadcrumb);
}

void ALPanelInventoryExplorer::onSearch(const std::string& search_string)
{
    if (!LLInventoryModelBackgroundFetch::instance().inventoryFetchStarted())
    {
        LLInventoryModelBackgroundFetch::instance().start();
    }

    mSearchString = search_string;
    applySearch();
}

void ALPanelInventoryExplorer::applySearch()
{
    // Keep every collection synchronized so switching views never clears the query.
    mTreePanel->setFilterSubString(mSearchString);
    mListPanel->setFilterSubString(mSearchString);
    mGalleryPanel->setFilterSubString(mSearchString);
    mRecentPanel->setFilterSubString(mSearchString);
    mWornPanel->setFilterSubString(mSearchString);
    mFavoritesPanel->setFilterSubString(mSearchString);
    mTypeFilterPanel->setFilterSubString(mSearchString);
}

U32 ALPanelInventoryExplorer::getActiveSortOrder() const
{
    if (isAllItemsCollection())
    {
        if (mViewMode == EViewMode::LIST)
        {
            return mListPanel->getSortOrder();
        }
        if (mViewMode == EViewMode::GRID)
        {
            return mGalleryPanel->getSortOrder();
        }
    }
    return getActiveInventoryPanel()->getSortOrder();
}

void ALPanelInventoryExplorer::onSort(const LLSD& data)
{
    U32 sort_order = getActiveSortOrder();
    const std::string sort_type = data.asString();
    if (sort_type == "name")
    {
        sort_order &= ~LLInventoryFilter::SO_DATE;
    }
    else if (sort_type == "date")
    {
        sort_order |= LLInventoryFilter::SO_DATE;
    }
    else if (sort_type == "foldersalwaysbyname")
    {
        sort_order ^= LLInventoryFilter::SO_FOLDERS_BY_NAME;
    }
    else if (sort_type == "systemfolderstotop")
    {
        sort_order ^= LLInventoryFilter::SO_SYSTEM_FOLDERS_TO_TOP;
    }
    setActiveSortOrder(sort_order);
}

bool ALPanelInventoryExplorer::isSortChecked(const LLSD& data) const
{
    const U32 sort_order = getActiveSortOrder();
    const std::string sort_type = data.asString();
    if (sort_type == "name")
    {
        return !(sort_order & LLInventoryFilter::SO_DATE);
    }
    if (sort_type == "date")
    {
        return sort_order & LLInventoryFilter::SO_DATE;
    }
    if (sort_type == "foldersalwaysbyname")
    {
        return sort_order & LLInventoryFilter::SO_FOLDERS_BY_NAME;
    }
    if (sort_type == "systemfolderstotop")
    {
        return sort_order & LLInventoryFilter::SO_SYSTEM_FOLDERS_TO_TOP;
    }
    return false;
}

bool ALPanelInventoryExplorer::isSortVisible(const LLSD& data) const
{
    return data.asString() != "systemfolderstotop" ||
        !isAllItemsCollection() || mViewMode == EViewMode::TREE;
}

void ALPanelInventoryExplorer::setActiveSortOrder(U32 sort_order)
{
    if (isAllItemsCollection())
    {
        mTreePanel->setSortOrder(sort_order);
        mListPanel->setSortOrder(sort_order);
        mGalleryPanel->setSortOrder(sort_order, true);
        gSavedSettings.setU32(LLInventoryPanel::DEFAULT_SORT_ORDER, sort_order);
        return;
    }

    LLInventoryPanel* panel = getActiveInventoryPanel();
    panel->setSortOrder(sort_order);
    gSavedSettings.setU32(panel == mRecentPanel
        ? LLInventoryPanel::RECENTITEMS_SORT_ORDER
        : LLInventoryPanel::DEFAULT_SORT_ORDER, sort_order);
}

void ALPanelInventoryExplorer::onActions()
{
    if (isAllItemsCollection() && mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->showContextMenu(getChild<LLUICtrl>("actions_button"), 0, 0,
            mGalleryPanel->getFirstSelectedItemID());
        return;
    }

    LLInventoryPanel* panel = getActiveInventoryPanel();
    if (isAllItemsCollection() && mViewMode == EViewMode::LIST)
    {
        panel = mListPanel;
    }
    panel->showContextMenuForSelection();
}

void ALPanelInventoryExplorer::onCreateButton()
{
    updateCreateMenu();
    LLMenuGL* menu = static_cast<LLMenuGL*>(mCreateMenuHandle.get());
    if (!menu)
    {
        return;
    }

    menu->buildDrawLabels();
    menu->updateParent(LLMenuGL::sMenuContainer);
    S32 menu_x = 0;
    S32 menu_y = 0;
    mCreateButton->localPointToOtherView(0, 0, &menu_x, &menu_y, this);
    LLMenuGL::showPopup(this, menu, menu_x, menu_y);
}

void ALPanelInventoryExplorer::onCreate(const LLSD& data)
{
    if (!is_add_allowed(getCreateDestination()))
    {
        return;
    }

    if (isAllItemsCollection() && mViewMode == EViewMode::GRID)
    {
        mGalleryPanel->doCreate(getCreateDestination(), data);
    }
    else if (isAllItemsCollection() && mViewMode == EViewMode::LIST)
    {
        mListPanel->doCreate(data);
    }
    else
    {
        getActiveInventoryPanel()->doCreate(data);
    }
}

void ALPanelInventoryExplorer::onCreateMenuAction(const LLSD& data)
{
    if (data.asString() == "shop")
    {
        LLWeb::loadURL(gSavedSettings.getString("MarketplaceURL"));
    }
}

void ALPanelInventoryExplorer::updateCreateMenu()
{
    LLMenuGL* menu = static_cast<LLMenuGL*>(mCreateMenuHandle.get());
    if (!menu)
    {
        return;
    }

    const bool enabled = is_add_allowed(getCreateDestination());
    const bool folder_enabled = enabled && mActiveCollection != ActiveCollection(EBuiltinCollection::RECENT);
    menu->getChild<LLMenuItemGL>("New Folder")->setEnabled(folder_enabled);
    menu->getChild<LLMenuItemGL>("New Script")->setEnabled(enabled);
    menu->getChild<LLMenuItemGL>("New Note")->setEnabled(enabled);
    menu->getChild<LLMenuItemGL>("New Gesture")->setEnabled(enabled);
    menu->setItemEnabled("New Clothes", enabled);
    menu->setItemEnabled("New Body Parts", enabled);
    menu->setItemEnabled("New Settings", enabled);
}

LLUUID ALPanelInventoryExplorer::getCreateDestination() const
{
    if (const LLViewerInventoryCategory* selected = gInventory.getCategory(mSelectedItemID))
    {
        return selected->getUUID();
    }
    if (isAllItemsCollection() && mViewMode != EViewMode::TREE)
    {
        return getCurrentSingleFolderRoot();
    }
    if (const LLInventoryObject* selected = gInventory.getObject(mSelectedItemID))
    {
        return selected->getParentUUID();
    }
    return gInventory.getRootFolderID();
}

bool ALPanelInventoryExplorer::startHoldingItemDrag(const LLUUID& item_id)
{
    const std::array<LLInventoryPanel*, 6> panels{
        mTreePanel, mListPanel, mRecentPanel, mWornPanel, mFavoritesPanel, mTypeFilterPanel
    };
    for (LLInventoryPanel* panel : panels)
    {
        LLFolderViewItem* item = panel->getItemByID(item_id);
        if (!item || !item->getViewModelItem())
        {
            continue;
        }

        std::vector<LLFolderViewModelItem*> drag_items{ item->getViewModelItem() };
        return panel->getRootViewModel().startDrag(drag_items);
    }
    return false;
}

void ALPanelInventoryExplorer::updateHoldingTrayVisibility()
{
    if (!mHoldingTray || !mHoldingTrayLayoutPanel || !mContentLayoutStack)
    {
        return;
    }

    const bool visible = mHoldingTray->hasItems() ||
        LLToolDragAndDrop::instance().hasMouseCapture();
    if (mHoldingTrayLayoutPanel->getVisible() == visible)
    {
        return;
    }

    mHoldingTrayLayoutPanel->setVisible(visible);
    mContentLayoutStack->collapsePanel(mHoldingTrayLayoutPanel, !visible);
    mContentLayoutStack->updateLayout();
}
