/**
 * @file alpanelinventoryholdingtray.cpp
 * @brief Per-floater inventory references for the Inventory Explorer
 *
 * $LicenseInfo:firstyear=2026&license=viewerlgpl$
 * Alchemy Viewer Source Code
 * Copyright (C) 2026, Alchemy Viewer Project.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation;
 * version 2.1 of the License only.
 * $/LicenseInfo$
 */

#include "llviewerprecompiledheaders.h"

#include "alpanelinventoryholdingtray.h"

#include "llbutton.h"
#include "llcallbacklist.h"
#include "llflatlistview.h"
#include "lliconctrl.h"
#include "llinventoryicon.h"
#include "llinventorymodel.h"
#include "lltextbox.h"
#include "lltooldraganddrop.h"
#include "llviewerinventory.h"

#include <algorithm>
#include <utility>

namespace
{
class ALPanelInventoryHoldingItem final : public LLPanel
{
public:
    ALPanelInventoryHoldingItem(const LLUUID& item_id,
                                std::function<void()> remove_callback,
                                std::function<void()> drag_callback,
                                std::function<void(LLUICtrl*, S32, S32)> context_menu_callback)
    :   LLPanel(),
        mItemID(item_id),
        mDragCallback(std::move(drag_callback)),
        mContextMenuCallback(std::move(context_menu_callback))
    {
        buildFromFile("panel_al_inventory_explorer_holding_item.xml");
        getChild<LLButton>("remove_button")->setCommitCallback(
            [callback = std::move(remove_callback)](LLUICtrl*, const LLSD&) { callback(); });
        refreshItem();
    }

    void refreshItem()
    {
        const LLInventoryObject* object = gInventory.getObject(mItemID);
        if (!object)
        {
            return;
        }

        getChild<LLTextBox>("item_name")->setText(object->getName());
        if (const LLViewerInventoryItem* item = gInventory.getItem(mItemID))
        {
            getChild<LLIconCtrl>("item_icon")->setValue(LLInventoryIcon::getIconName(
                item->getType(), item->getInventoryType(), item->getFlags(), false));
        }
        else
        {
            getChild<LLIconCtrl>("item_icon")->setValue("InvExplorer_Item_Folder");
        }
    }

    bool handleMouseDown(S32 x, S32 y, MASK mask) override
    {
        LLButton* remove_button = getChild<LLButton>("remove_button");
        if (remove_button->getRect().pointInRect(x, y))
        {
            return LLPanel::handleMouseDown(x, y, mask);
        }

        gFocusMgr.setMouseCapture(this);
        S32 screen_x = 0;
        S32 screen_y = 0;
        localPointToScreen(x, y, &screen_x, &screen_y);
        LLToolDragAndDrop::instance().setDragStart(screen_x, screen_y);
        return true;
    }

    bool handleMouseUp(S32 x, S32 y, MASK mask) override
    {
        if (hasMouseCapture())
        {
            gFocusMgr.setMouseCapture(nullptr);
            return true;
        }
        return LLPanel::handleMouseUp(x, y, mask);
    }

    bool handleRightMouseDown(S32 x, S32 y, MASK mask) override
    {
        if (mContextMenuCallback)
        {
            mContextMenuCallback(this, x, y);
            return true;
        }
        return LLPanel::handleRightMouseDown(x, y, mask);
    }

    bool handleHover(S32 x, S32 y, MASK mask) override
    {
        if (hasMouseCapture())
        {
            S32 screen_x = 0;
            S32 screen_y = 0;
            localPointToScreen(x, y, &screen_x, &screen_y);
            if (LLToolDragAndDrop::instance().isOverThreshold(screen_x, screen_y))
            {
                mDragCallback();
                return LLToolDragAndDrop::instance().handleHover(x, y, mask);
            }
        }
        return LLPanel::handleHover(x, y, mask);
    }

private:
    LLUUID mItemID;
    std::function<void()> mDragCallback;
    std::function<void(LLUICtrl*, S32, S32)> mContextMenuCallback;
};

LLUUID getCargoID(EDragAndDropType cargo_type, void* cargo_data)
{
    if (!cargo_data)
    {
        return LLUUID::null;
    }

    const LLInventoryObject* object = nullptr;
    switch (cargo_type)
    {
        case DAD_CATEGORY:
            object = static_cast<LLInventoryCategory*>(cargo_data);
            break;
        case DAD_TEXTURE:
        case DAD_SOUND:
        case DAD_CALLINGCARD:
        case DAD_LANDMARK:
        case DAD_SCRIPT:
        case DAD_CLOTHING:
        case DAD_OBJECT:
        case DAD_NOTECARD:
        case DAD_BODYPART:
        case DAD_ANIMATION:
        case DAD_GESTURE:
        case DAD_LINK:
        case DAD_MESH:
        case DAD_SETTINGS:
        case DAD_MATERIAL:
            object = static_cast<LLInventoryItem*>(cargo_data);
            break;
        default:
            return LLUUID::null;
    }
    return object ? object->getUUID() : LLUUID::null;
}
}

static LLPanelInjector<ALPanelInventoryHoldingTray> t_panel_al_inventory_holding_tray(
    "panel_al_inventory_holding_tray");

ALPanelInventoryHoldingTray::ALPanelInventoryHoldingTray()
:   LLPanel()
{
}

ALPanelInventoryHoldingTray::~ALPanelInventoryHoldingTray()
{
    mEndDragConnection.disconnect();
    gInventory.removeObserver(this);
}

bool ALPanelInventoryHoldingTray::postBuild()
{
    if (!LLPanel::postBuild())
    {
        return false;
    }

    mItemsList = getChild<LLFlatListView>("holding_items");
    mItemsList->setAllowSelection(false);
    getChild<LLButton>("clear_button")->setCommitCallback(boost::bind(
        &ALPanelInventoryHoldingTray::clearItems, this));
    gInventory.addObserver(this);
    return true;
}

bool ALPanelInventoryHoldingTray::handleDragAndDrop(
    S32 x, S32 y, MASK mask, bool drop, EDragAndDropType cargo_type,
    void* cargo_data, EAcceptance* accept, std::string& tooltip_msg)
{
    const LLUUID item_id = getCargoID(cargo_type, cargo_data);
    if (item_id.isNull() || !gInventory.getObject(item_id))
    {
        *accept = ACCEPT_NO;
        tooltip_msg = "Only inventory items can be held here";
        return true;
    }

    *accept = ACCEPT_YES_MULTI;
    if (drop)
    {
        mDroppedOnSelf = item_id == mDraggedItemID;
        addItem(item_id);
    }
    return true;
}

void ALPanelInventoryHoldingTray::changed(U32 mask)
{
    if (mask == LLInventoryObserver::NONE)
    {
        return;
    }

    const auto first_missing = std::remove_if(mItemIDs.begin(), mItemIDs.end(),
        [](const LLUUID& item_id) { return !gInventory.getObject(item_id); });
    if (first_missing != mItemIDs.end())
    {
        mItemIDs.erase(first_missing, mItemIDs.end());
    }
    rebuildItems();
}

void ALPanelInventoryHoldingTray::setDragStartCallback(
    std::function<bool(const LLUUID&)> callback)
{
    mDragStartCallback = std::move(callback);
}

void ALPanelInventoryHoldingTray::setContextMenuCallback(
    std::function<void(LLUICtrl*, S32, S32, const LLUUID&)> callback)
{
    mContextMenuCallback = std::move(callback);
}

void ALPanelInventoryHoldingTray::addItem(const LLUUID& item_id)
{
    if (std::find(mItemIDs.begin(), mItemIDs.end(), item_id) != mItemIDs.end())
    {
        return;
    }
    mItemIDs.push_back(item_id);
    rebuildItems();
}

void ALPanelInventoryHoldingTray::removeItem(const LLUUID& item_id)
{
    const auto found = std::find(mItemIDs.begin(), mItemIDs.end(), item_id);
    if (found == mItemIDs.end())
    {
        return;
    }
    mItemIDs.erase(found);
    rebuildItems();
}

void ALPanelInventoryHoldingTray::clearItems()
{
    mItemIDs.clear();
    rebuildItems();
}

void ALPanelInventoryHoldingTray::rebuildItems()
{
    if (!mItemsList)
    {
        return;
    }

    mItemsList->clear();
    for (const LLUUID& item_id : mItemIDs)
    {
        auto* row = new ALPanelInventoryHoldingItem(item_id,
            boost::bind(&ALPanelInventoryHoldingTray::removeItem, this, item_id),
            boost::bind(&ALPanelInventoryHoldingTray::startDrag, this, item_id),
            [this, item_id](LLUICtrl* ctrl, S32 x, S32 y)
            {
                if (mContextMenuCallback)
                {
                    mContextMenuCallback(ctrl, x, y, item_id);
                }
            });
        mItemsList->addItem(row, item_id, ADD_BOTTOM);
    }
}

void ALPanelInventoryHoldingTray::startDrag(const LLUUID& item_id)
{
    if (!mDragStartCallback || !mDragStartCallback(item_id))
    {
        return;
    }

    mDraggedItemID = item_id;
    mDroppedOnSelf = false;
    mEndDragConnection.disconnect();
    mEndDragConnection = LLToolDragAndDrop::instance().setEndDragCallback(boost::bind(
        &ALPanelInventoryHoldingTray::onEndDrag, this));
}

void ALPanelInventoryHoldingTray::onEndDrag()
{
    const bool accepted = LLToolDragAndDrop::instance().getLastAccept() >= ACCEPT_YES_COPY_SINGLE;
    const LLUUID item_id = mDraggedItemID;
    mEndDragConnection.disconnect();
    mDraggedItemID.setNull();
    if (accepted && !mDroppedOnSelf)
    {
        doOnIdleOneTime(boost::bind(&ALPanelInventoryHoldingTray::removeAfterDrag,
            getHandle(), item_id));
    }
    mDroppedOnSelf = false;
}

void ALPanelInventoryHoldingTray::removeAfterDrag(
    LLHandle<LLPanel> tray_handle, LLUUID item_id)
{
    if (tray_handle.isDead())
    {
        return;
    }
    static_cast<ALPanelInventoryHoldingTray*>(tray_handle.get())->removeItem(item_id);
}
