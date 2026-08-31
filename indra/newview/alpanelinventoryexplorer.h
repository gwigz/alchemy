/**
 * @file alpanelinventoryexplorer.h
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

#ifndef AL_PANELINVENTORYEXPLORER_H
#define AL_PANELINVENTORYEXPLORER_H

#include "llinventoryobserver.h"
#include "llpanel.h"
#include "lluuid.h"

#include <boost/signals2/connection.hpp>

#include <deque>
#include <list>
#include <map>
#include <string>
#include <variant>
#include <vector>

class LLButton;
class LLFilterEditor;
class LLFlatListView;
class LLFolderViewItem;
class LLInventoryGallery;
class LLInventoryPanel;
class LLInventorySingleFolderPanel;
class LLLayoutPanel;
class LLLayoutStack;
class LLTextBox;
class ALPanelInventoryInspector;
class ALPanelInventoryHoldingTray;

class ALPanelInventoryExplorer final : public LLPanel, public LLInventoryObserver
{
public:
    ALPanelInventoryExplorer();
    ~ALPanelInventoryExplorer() override;

    bool postBuild() override;
    void draw() override;
    void reshape(S32 width, S32 height, bool called_from_parent = true) override;
    void changed(U32 mask) override;

private:
    enum class ELayoutState
    {
        UNINITIALIZED,
        COMPACT,
        WIDE
    };

    enum class EViewMode
    {
        TREE,
        LIST,
        GRID
    };

    enum class EBuiltinCollection
    {
        ALL_ITEMS,
        RECENT,
        WORN,
        FAVORITES
    };

    using ActiveCollection = std::variant<EBuiltinCollection, std::string>;

    struct SingleFolderState
    {
        LLUUID root;
        std::list<LLUUID> backward;
        std::list<LLUUID> forward;
        U32 sort_order{ 0 };
    };

    void applyLayout(S32 width);
    void setViewMode(EViewMode mode);
    void updateViewVisibility();

    void selectBuiltinCollection(EBuiltinCollection collection);
    void selectTypeCollection(const std::string& type_id);
    void onAddTypeFilter(const LLSD& data);
    bool canAddTypeFilter(const LLSD& data) const;
    void removeTypeFilter(const std::string& type_id);
    bool isAllItemsCollection() const;
    LLInventoryPanel* getActiveInventoryPanel() const;
    std::string getActiveCollectionLabel() const;
    void updateCollectionSelection();

    void loadTypeFilters();
    void saveTypeFilters() const;
    void rebuildTypeFilterRows();
    void updateTypeFilterRow(LLPanel* row, const std::string& type_id);

    void scheduleCountRefresh();
    static void onIdleCountRefresh(LLHandle<LLPanel> panel_handle);
    void refreshCollectionCounts();

    SingleFolderState makeSingleFolderStateFromTree();
    SingleFolderState captureSingleFolderState() const;
    void applySingleFolderState(const SingleFolderState& state);
    LLUUID getCurrentSingleFolderRoot() const;

    void onInventorySelection(LLInventoryPanel* panel,
                              const std::deque<LLFolderViewItem*>& items,
                              bool user_action);
    void onGallerySelection(const LLUUID& item_id);
    void restoreSelection();
    void updateInspectorSelection();
    bool selectionBelongsToRoot(const LLUUID& item_id, const LLUUID& root_id) const;

    void onRootChanged();
    void onBack();
    void onForward();
    void onUp();
    void updateNavigationButtons();
    void updateStatusText();
    bool startHoldingItemDrag(const LLUUID& item_id);
    void updateHoldingTrayVisibility();

    void onSearch(const std::string& search_string);
    void applySearch();
    void onSort(const LLSD& data);
    bool isSortChecked(const LLSD& data) const;
    bool isSortVisible(const LLSD& data) const;
    U32 getActiveSortOrder() const;
    void setActiveSortOrder(U32 sort_order);
    void onActions();
    void onCreateButton();
    void onCreate(const LLSD& data);
    void onCreateMenuAction(const LLSD& data);
    void updateCreateMenu();
    LLUUID getCreateDestination() const;

    LLLayoutStack* mLayoutStack{ nullptr };
    LLLayoutPanel* mRailPanel{ nullptr };
    LLLayoutPanel* mInspectorPanel{ nullptr };
    LLLayoutStack* mContentLayoutStack{ nullptr };
    LLLayoutPanel* mHoldingTrayLayoutPanel{ nullptr };

    LLInventoryPanel* mTreePanel{ nullptr };
    LLInventorySingleFolderPanel* mListPanel{ nullptr };
    LLInventoryGallery* mGalleryPanel{ nullptr };
    LLInventoryPanel* mRecentPanel{ nullptr };
    LLInventoryPanel* mWornPanel{ nullptr };
    LLInventoryPanel* mFavoritesPanel{ nullptr };
    LLInventoryPanel* mTypeFilterPanel{ nullptr };
    LLFlatListView* mTypeFilterList{ nullptr };
    LLButton* mBackButton{ nullptr };
    LLButton* mForwardButton{ nullptr };
    LLButton* mUpButton{ nullptr };
    LLButton* mTreeViewButton{ nullptr };
    LLButton* mListViewButton{ nullptr };
    LLButton* mGridViewButton{ nullptr };
    LLButton* mCreateButton{ nullptr };
    LLFilterEditor* mSearchEditor{ nullptr };
    LLTextBox* mStatusText{ nullptr };
    ALPanelInventoryInspector* mInspector{ nullptr };
    ALPanelInventoryHoldingTray* mHoldingTray{ nullptr };

    ELayoutState mLayoutState{ ELayoutState::UNINITIALIZED };
    EViewMode mViewMode{ EViewMode::TREE };
    ActiveCollection mActiveCollection{ EBuiltinCollection::ALL_ITEMS };
    std::vector<std::string> mTypeFilterIDs;
    std::map<std::string, LLPanel*> mTypeFilterRows;
    LLUUID mSelectedItemID;
    std::string mSearchString;
    LLHandle<LLView> mCreateMenuHandle;
    bool mRestoringSelection{ false };
    bool mCountRefreshPending{ false };

    boost::signals2::connection mListRootChangedConnection;
    boost::signals2::connection mGalleryRootChangedConnection;
    boost::signals2::connection mGallerySelectionConnection;
};

#endif // AL_PANELINVENTORYEXPLORER_H
