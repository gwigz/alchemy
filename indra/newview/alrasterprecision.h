/**
 * @file alrasterprecision.h
 * @brief Low-precision raster grid and RGB555 reference arithmetic.
 *
 * Copyright (c) 2026, Alchemy Viewer Project.
 *
 * This source is licensed under the GNU Lesser General Public License,
 * version 2.1.
 */

#ifndef AL_RASTER_PRECISION_H
#define AL_RASTER_PRECISION_H

#include "stdtypes.h"

struct ALRasterRect
{
    F32 mX      = 0.f;
    F32 mY      = 0.f;
    F32 mWidth  = 1.f;
    F32 mHeight = 1.f;
};

struct ALRasterGrid
{
    S32          mWidth  = 1;
    S32          mHeight = 1;
    ALRasterRect mDisplayRect;
    ALRasterRect mSourceRect;
    bool         mRGB555        = false;
    bool         mOrderedDither = false;
    bool         mVertexQuantization = false;
    S32          mVertexGridWidth    = 1;
    S32          mVertexGridHeight   = 1;
};

class ALRasterPrecision
{
public:
    enum EProfile : S32
    {
        PROFILE_CONTEMPORARY = 0,
        PROFILE_REFERENCE    = 1,
        PROFILE_CUSTOM       = 2,
    };

    enum EGridMode : S32
    {
        GRID_FIXED_320_240 = 0,
        GRID_ASPECT_240    = 1,
        GRID_ASPECT_360    = 2,
        GRID_ASPECT_480    = 3,
    };

    enum EVertexGridMode : S32
    {
        VERTEX_GRID_OFF    = 0,
        VERTEX_GRID_FINE   = 1,
        VERTEX_GRID_MATCH  = 2,
        VERTEX_GRID_COARSE = 3,
    };

    static ALRasterGrid resolve(S32  viewport_width,
                                S32  viewport_height,
                                S32  profile,
                                S32  custom_grid_mode,
                                S32  custom_vertex_grid_mode,
                                bool custom_rgb555,
                                bool custom_ordered_dither);

    static S32 orderedDitherOffset(S32 x, S32 y);
    static U8  quantizeRGB555(U8 channel, S32 x, S32 y, bool ordered_dither);
};

#endif
