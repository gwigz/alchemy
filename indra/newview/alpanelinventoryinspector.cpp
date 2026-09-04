/**
 * @file alpanelinventoryinspector.cpp
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

#include "llviewerprecompiledheaders.h"

#include "alpanelinventoryinspector.h"

#include "llagent.h"
#include "llavatarname.h"
#include "llavatarnamecache.h"
#include "llbutton.h"
#include "lldate.h"
#include "lliconctrl.h"
#include "llinventoryfunctions.h"
#include "llinventorymodel.h"
#include "llinventorytype.h"
#include "llscrollcontainer.h"
#include "lltextbox.h"
#include "llthumbnailctrl.h"
#include "lluicolortable.h"
#include "llviewerinventory.h"
#include "llwearabletype.h"
#include "rlvactions.h"

#include <array>
#include <string>
#include <utility>
#include <vector>

namespace
{
constexpr const char* EMPTY_VALUE = "\xE2\x80\x94";

std::string joinLabels(const std::vector<std::string>& labels)
{
    std::string result;
    for (const std::string& label : labels)
    {
        if (!result.empty())
        {
            result += ", ";
        }
        result += label;
    }
    return result;
}

const char* inventoryExplorerIcon(const LLViewerInventoryItem& item)
{
    switch (item.getInventoryType())
    {
        case LLInventoryType::IT_TEXTURE: return "InvExplorer_Item_Texture";
        case LLInventoryType::IT_SOUND: return "InvExplorer_Item_Sound";
        case LLInventoryType::IT_CALLINGCARD: return "InvExplorer_Item_CallingCard";
        case LLInventoryType::IT_LANDMARK: return "InvExplorer_Item_Landmark";
        case LLInventoryType::IT_OBJECT:
        case LLInventoryType::IT_ATTACHMENT: return "InvExplorer_Item_Object";
        case LLInventoryType::IT_NOTECARD: return "InvExplorer_Item_Notecard";
        case LLInventoryType::IT_CATEGORY: return "InvExplorer_Item_Folder";
        case LLInventoryType::IT_LSL: return "InvExplorer_Item_Script";
        case LLInventoryType::IT_SNAPSHOT: return "InvExplorer_Item_Snapshot";
        case LLInventoryType::IT_WEARABLE:
        {
            const LLWearableType::EType wearable_type = item.getWearableType();
            return wearable_type >= LLWearableType::WT_SHAPE &&
                   wearable_type <= LLWearableType::WT_EYES
                ? "InvExplorer_Item_BodyPart"
                : "InvExplorer_Item_Clothing";
        }
        case LLInventoryType::IT_ANIMATION: return "InvExplorer_Item_Animation";
        case LLInventoryType::IT_GESTURE: return "InvExplorer_Item_Gesture";
        case LLInventoryType::IT_MESH: return "InvExplorer_Item_Mesh";
        case LLInventoryType::IT_SETTINGS: return "InvExplorer_Item_Settings";
        case LLInventoryType::IT_MATERIAL: return "InvExplorer_Item_Material";
        default: return "InvExplorer_Inspector";
    }
}

std::string inventoryTypeLabel(const LLViewerInventoryItem& item)
{
    if (item.getInventoryType() == LLInventoryType::IT_WEARABLE)
    {
        return LLWearableType::getInstance()->getTypeLabel(item.getWearableType());
    }

    const std::string label = LLInventoryType::lookupHumanReadable(item.getInventoryType());
    return label.empty() ? "Inventory item" : label;
}

std::string permissionLabel(const LLViewerInventoryItem& item)
{
    const LLPermissions& permissions = item.getPermissions();
    const std::array<std::pair<PermissionBit, const char*>, 3> permission_names{{
        { PERM_COPY, "Copy" },
        { PERM_MODIFY, "Modify" },
        { PERM_TRANSFER, "Transfer" },
    }};

    std::vector<std::string> allowed;
    for (const auto& [permission, name] : permission_names)
    {
        if (permissions.allowOperationBy(permission, gAgent.getID(), gAgent.getGroupID()))
        {
            allowed.emplace_back(name);
        }
    }
    return allowed.empty() ? "No permissions" : joinLabels(allowed);
}

std::string stateLabel(const LLViewerInventoryItem& item)
{
    std::vector<std::string> states;
    if (item.getIsFavorite())
    {
        states.emplace_back("Favorite");
    }
    if (get_is_item_worn(&item))
    {
        states.emplace_back("Worn");
    }
    if (item.getIsLinkType())
    {
        states.emplace_back("Link");
    }
    return states.empty() ? std::string() : joinLabels(states);
}
}

static LLPanelInjector<ALPanelInventoryInspector> t_panel_al_inventory_inspector(
    "panel_al_inventory_inspector");

ALPanelInventoryInspector::ALPanelInventoryInspector()
:   LLPanel()
{
}

ALPanelInventoryInspector::~ALPanelInventoryInspector()
{
    mCreatorNameConnection.disconnect();
}

bool ALPanelInventoryInspector::postBuild()
{
    if (!LLPanel::postBuild())
    {
        return false;
    }

    mEmptyPanel = getChild<LLPanel>("empty_panel");
    mDetailsScroll = getChild<LLScrollContainer>("details_scroll");
    mDetailsPanel = getChild<LLPanel>("details_panel");
    mThumbnail = getChild<LLThumbnailCtrl>("item_thumbnail");
    mTypeIcon = getChild<LLIconCtrl>("item_type_icon");
    mNameText = getChild<LLTextBox>("item_name");
    mTypeText = getChild<LLTextBox>("item_type");
    mCreatorText = getChild<LLTextBox>("item_creator");
    mCreatedText = getChild<LLTextBox>("item_created");
    mPermissionsText = getChild<LLTextBox>("item_permissions");
    mStateText = getChild<LLTextBox>("item_state");
    mActionButtons = {
        getChild<LLButton>("primary_action"),
        getChild<LLButton>("edit_action"),
        getChild<LLButton>("share_action"),
        getChild<LLButton>("properties_action"),
    };
    for (std::size_t index = 0; index < mActionButtons.size(); ++index)
    {
        mActionButtons[index]->setCommitCallback(
            [this, index](LLUICtrl*, const LLSD&)
            {
                if (mActionCallback && !mActionCommands[index].empty())
                {
                    mActionCallback(mActionCommands[index]);
                }
            });
    }
    resizeDetailsPanel();
    showEmpty();
    return true;
}

void ALPanelInventoryInspector::setActions(
    const std::array<ActionState, 4>& actions)
{
    static constexpr std::array<S32, 4> widths{ 44, 40, 48, 66 };
    S32 left = 14;
    for (std::size_t index = 0; index < actions.size(); ++index)
    {
        const ActionState& action = actions[index];
        mActionCommands[index] = action.command;
        mActionButtons[index]->setLabel(action.label);
        mActionButtons[index]->setVisible(action.visible);
        mActionButtons[index]->setEnabled(action.enabled);
        if (action.visible)
        {
            mActionButtons[index]->setOrigin(left, mActionButtons[index]->getRect().mBottom);
            mActionButtons[index]->reshape(widths[index], mActionButtons[index]->getRect().getHeight());
            left += widths[index] + 4;
        }
    }
}

void ALPanelInventoryInspector::setActionCallback(
    std::function<void(const std::string&)> callback)
{
    mActionCallback = std::move(callback);
}

void ALPanelInventoryInspector::reshape(S32 width, S32 height, bool called_from_parent)
{
    LLPanel::reshape(width, height, called_from_parent);
    resizeDetailsPanel();
}

void ALPanelInventoryInspector::resizeDetailsPanel()
{
    if (!mDetailsScroll || !mDetailsPanel)
    {
        return;
    }

    const S32 width = mDetailsScroll->getVisibleContentRect().getWidth();
    mDetailsPanel->reshape(width, mDetailsPanel->getRect().getHeight());
}

void ALPanelInventoryInspector::setObjectID(const LLUUID& object_id)
{
    mObjectID = object_id;
    refreshObject();
}

void ALPanelInventoryInspector::refreshObject()
{
    mCreatorNameConnection.disconnect();

    const LLInventoryObject* object = gInventory.getObject(mObjectID);
    if (!object)
    {
        showEmpty();
        return;
    }

    mEmptyPanel->setVisible(false);
    mDetailsScroll->setVisible(true);
    mNameText->setText(object->getName());
    mThumbnail->setValue(object->getThumbnailUUID());
    mThumbnail->setVisible(object->getThumbnailUUID().notNull());
    mTypeIcon->setVisible(object->getThumbnailUUID().isNull());

    if (const LLViewerInventoryItem* item = gInventory.getItem(mObjectID))
    {
        showItem(*item);
        return;
    }

    mTypeIcon->setValue("InvExplorer_Item_Folder");
    mTypeText->setText(std::string("Folder"));
    mCreatorText->setText(std::string(EMPTY_VALUE));
    mCreatedText->setText(std::string(EMPTY_VALUE));
    mPermissionsText->setText(std::string(EMPTY_VALUE));
    mStateText->setText(std::string(object->getIsFavorite() ? "Favorite" : ""));
    mStateText->setColor(LLUIColorTable::instance().getColor(
        object->getIsFavorite() ? "InventoryFavoriteColor" : "LabelTextColor"));
}

void ALPanelInventoryInspector::showEmpty()
{
    mObjectID.setNull();
    mEmptyPanel->setVisible(true);
    mDetailsScroll->setVisible(false);
}

void ALPanelInventoryInspector::showItem(const LLViewerInventoryItem& item)
{
    mTypeIcon->setValue(inventoryExplorerIcon(item));
    mTypeText->setText(inventoryTypeLabel(item));
    mCreatedText->setText(
        LLDate(static_cast<F64>(item.getCreationDate())).toLocalDateString("%Y-%m-%d %H:%M"));
    mPermissionsText->setText(permissionLabel(item));
    mStateText->setText(stateLabel(item));
    mStateText->setColor(LLUIColorTable::instance().getColor(
        item.getIsFavorite() ? "InventoryFavoriteColor" : "LabelTextColor"));

    const LLUUID creator_id = item.getCreatorUUID();
    if (creator_id.isNull())
    {
        mCreatorText->setText(std::string(EMPTY_VALUE));
        return;
    }
    if (RlvActions::isRlvEnabled() &&
        !RlvActions::canShowName(RlvActions::SNC_DEFAULT, creator_id))
    {
        mCreatorText->setText(std::string("Hidden"));
        return;
    }

    LLAvatarName avatar_name;
    if (LLAvatarNameCache::get(creator_id, &avatar_name))
    {
        mCreatorText->setText(avatar_name.getCompleteName());
    }
    else
    {
        mCreatorText->setText(std::string("Loading..."));
        mCreatorNameConnection = LLAvatarNameCache::get(creator_id, boost::bind(
            &ALPanelInventoryInspector::onCreatorName, this, _1, _2));
    }
}

void ALPanelInventoryInspector::onCreatorName(
    const LLUUID& creator_id,
    const LLAvatarName& avatar_name)
{
    const LLViewerInventoryItem* item = gInventory.getItem(mObjectID);
    if (item && item->getCreatorUUID() == creator_id)
    {
        mCreatorText->setText(avatar_name.getCompleteName());
    }
}
