/**
 * @file alpanelinventoryinspector.h
 * @brief Read-only details for the Inventory Explorer selection
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

#ifndef AL_PANELINVENTORYINSPECTOR_H
#define AL_PANELINVENTORYINSPECTOR_H

#include "llpanel.h"
#include "lluuid.h"

#include <boost/signals2/connection.hpp>

#include <array>
#include <functional>
#include <string>

class LLAvatarName;
class LLButton;
class LLIconCtrl;
class LLScrollContainer;
class LLTextBox;
class LLThumbnailCtrl;
class LLViewerInventoryItem;

class ALPanelInventoryInspector final : public LLPanel
{
public:
    struct ActionState
    {
        std::string label;
        std::string command;
        bool visible{ false };
        bool enabled{ false };
    };

    ALPanelInventoryInspector();
    ~ALPanelInventoryInspector() override;

    bool postBuild() override;
    void reshape(S32 width, S32 height, bool called_from_parent = true) override;
    void setObjectID(const LLUUID& object_id);
    void refreshObject();
    void setActions(const std::array<ActionState, 4>& actions);
    void setActionCallback(std::function<void(const std::string&)> callback);

private:
    void resizeDetailsPanel();
    void showEmpty();
    void showItem(const LLViewerInventoryItem& item);
    void onCreatorName(const LLUUID& creator_id, const LLAvatarName& avatar_name);

    LLPanel* mEmptyPanel{ nullptr };
    LLScrollContainer* mDetailsScroll{ nullptr };
    LLPanel* mDetailsPanel{ nullptr };
    LLThumbnailCtrl* mThumbnail{ nullptr };
    LLIconCtrl* mTypeIcon{ nullptr };
    LLTextBox* mNameText{ nullptr };
    LLTextBox* mTypeText{ nullptr };
    LLTextBox* mCreatorText{ nullptr };
    LLTextBox* mCreatedText{ nullptr };
    LLTextBox* mPermissionsText{ nullptr };
    LLTextBox* mStateText{ nullptr };
    std::array<LLButton*, 4> mActionButtons{};
    std::array<std::string, 4> mActionCommands;

    LLUUID mObjectID;
    std::function<void(const std::string&)> mActionCallback;
    boost::signals2::connection mCreatorNameConnection;
};

#endif // AL_PANELINVENTORYINSPECTOR_H
