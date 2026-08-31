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

class LLAvatarName;
class LLIconCtrl;
class LLTextBox;
class LLThumbnailCtrl;
class LLView;
class LLViewerInventoryItem;

class ALPanelInventoryInspector final : public LLPanel
{
public:
    ALPanelInventoryInspector();
    ~ALPanelInventoryInspector() override;

    bool postBuild() override;
    void setObjectID(const LLUUID& object_id);
    void refreshObject();

private:
    void showEmpty();
    void showItem(const LLViewerInventoryItem& item);
    void onCreatorName(const LLUUID& creator_id, const LLAvatarName& avatar_name);

    LLPanel* mEmptyPanel{ nullptr };
    LLView* mDetailsView{ nullptr };
    LLThumbnailCtrl* mThumbnail{ nullptr };
    LLIconCtrl* mTypeIcon{ nullptr };
    LLTextBox* mNameText{ nullptr };
    LLTextBox* mTypeText{ nullptr };
    LLTextBox* mCreatorText{ nullptr };
    LLTextBox* mCreatedText{ nullptr };
    LLTextBox* mPermissionsText{ nullptr };
    LLTextBox* mAttachmentText{ nullptr };
    LLTextBox* mStateText{ nullptr };

    LLUUID mObjectID;
    boost::signals2::connection mCreatorNameConnection;
};

#endif // AL_PANELINVENTORYINSPECTOR_H
