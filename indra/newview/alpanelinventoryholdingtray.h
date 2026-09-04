/**
 * @file alpanelinventoryholdingtray.h
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

#ifndef AL_PANELINVENTORYHOLDINGTRAY_H
#define AL_PANELINVENTORYHOLDINGTRAY_H

#include "llinventoryobserver.h"
#include "llpanel.h"
#include "lluuid.h"

#include <boost/signals2/connection.hpp>

#include <functional>
#include <vector>

class LLFlatListView;
class LLUICtrl;

class ALPanelInventoryHoldingTray final : public LLPanel, public LLInventoryObserver
{
public:
    ALPanelInventoryHoldingTray();
    ~ALPanelInventoryHoldingTray() override;

    bool postBuild() override;
    bool handleDragAndDrop(S32 x, S32 y, MASK mask, bool drop,
                           EDragAndDropType cargo_type, void* cargo_data,
                           EAcceptance* accept, std::string& tooltip_msg) override;
    void changed(U32 mask) override;

    bool hasItems() const { return !mItemIDs.empty(); }
    void setDragStartCallback(std::function<bool(const LLUUID&)> callback);
    void setContextMenuCallback(
        std::function<void(LLUICtrl*, S32, S32, const LLUUID&)> callback);

private:
    void addItem(const LLUUID& item_id);
    void removeItem(const LLUUID& item_id);
    void clearItems();
    void rebuildItems();
    void startDrag(const LLUUID& item_id);
    void onEndDrag();
    static void removeAfterDrag(LLHandle<LLPanel> tray_handle, LLUUID item_id);

    LLFlatListView* mItemsList{ nullptr };
    std::vector<LLUUID> mItemIDs;
    LLUUID mDraggedItemID;
    bool mDroppedOnSelf{ false };
    std::function<bool(const LLUUID&)> mDragStartCallback;
    std::function<void(LLUICtrl*, S32, S32, const LLUUID&)> mContextMenuCallback;
    boost::signals2::connection mEndDragConnection;
};

#endif // AL_PANELINVENTORYHOLDINGTRAY_H
