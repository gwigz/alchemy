/**
 * @file alfloaterinventoryexplorer.h
 * @brief Opt-in Inventory Explorer floater shell
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

#ifndef AL_FLOATERINVENTORYEXPLORER_H
#define AL_FLOATERINVENTORYEXPLORER_H

#include "llfloater.h"

class ALFloaterInventoryExplorer final : public LLFloater
{
public:
    ALFloaterInventoryExplorer(const LLSD& key);
    ~ALFloaterInventoryExplorer() override = default;

    static const char* getPreferredInventoryFloater();
    static void togglePreferredInventory();
    static bool isPreferredInventoryVisible();
};

#endif // AL_FLOATERINVENTORYEXPLORER_H
