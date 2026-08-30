/**
 * @file alrasterprecision.cpp
 * @brief Low-precision raster grid and RGB555 reference arithmetic.
 *
 * Copyright (c) 2026, Alchemy Viewer Project.
 *
 * This source is licensed under the GNU Lesser General Public License,
 * version 2.1.
 */

#include "llviewerprecompiledheaders.h"

#include "alrasterprecision.h"

#include <algorithm>
#include <cmath>

namespace
{
constexpr F32 REFERENCE_ASPECT = 4.f / 3.f;

S32 aspectWidth(S32 height, F32 viewport_aspect)
{
    return llmax(1, static_cast<S32>(std::lround(static_cast<F32>(height) * viewport_aspect)));
}

void fitReferenceFrame(ALRasterGrid& grid, F32 viewport_aspect)
{
    if (viewport_aspect > REFERENCE_ASPECT)
    {
        grid.mDisplayRect.mWidth = REFERENCE_ASPECT / viewport_aspect;
        grid.mDisplayRect.mX     = (1.f - grid.mDisplayRect.mWidth) * 0.5f;
        grid.mSourceRect.mWidth  = REFERENCE_ASPECT / viewport_aspect;
        grid.mSourceRect.mX      = (1.f - grid.mSourceRect.mWidth) * 0.5f;
    }
    else if (viewport_aspect < REFERENCE_ASPECT)
    {
        grid.mDisplayRect.mHeight = viewport_aspect / REFERENCE_ASPECT;
        grid.mDisplayRect.mY      = (1.f - grid.mDisplayRect.mHeight) * 0.5f;
        grid.mSourceRect.mHeight  = viewport_aspect / REFERENCE_ASPECT;
        grid.mSourceRect.mY       = (1.f - grid.mSourceRect.mHeight) * 0.5f;
    }
}
} // namespace

ALRasterGrid ALRasterPrecision::resolve(S32  viewport_width,
                                        S32  viewport_height,
                                        S32  profile,
                                        S32  custom_grid_mode,
                                        S32  custom_vertex_grid_mode,
                                        bool custom_rgb555,
                                        bool custom_ordered_dither)
{
    viewport_width            = llmax(1, viewport_width);
    viewport_height           = llmax(1, viewport_height);
    const F32 viewport_aspect = static_cast<F32>(viewport_width) / static_cast<F32>(viewport_height);

    S32          grid_mode        = custom_grid_mode;
    S32          vertex_grid_mode = custom_vertex_grid_mode;
    ALRasterGrid grid;

    switch (profile)
    {
        case PROFILE_REFERENCE:
            grid_mode           = GRID_FIXED_320_240;
            vertex_grid_mode    = VERTEX_GRID_MATCH;
            grid.mRGB555        = true;
            grid.mOrderedDither = true;
            break;
        case PROFILE_CUSTOM:
            grid.mRGB555        = custom_rgb555;
            grid.mOrderedDither = custom_rgb555 && custom_ordered_dither;
            break;
        case PROFILE_CONTEMPORARY:
        default:
            grid_mode        = GRID_ASPECT_360;
            vertex_grid_mode = VERTEX_GRID_FINE;
            break;
    }

    switch (grid_mode)
    {
        case GRID_FIXED_320_240:
            grid.mWidth  = 320;
            grid.mHeight = 240;
            fitReferenceFrame(grid, viewport_aspect);
            break;
        case GRID_ASPECT_240:
            grid.mHeight = 240;
            grid.mWidth  = aspectWidth(grid.mHeight, viewport_aspect);
            break;
        case GRID_ASPECT_480:
            grid.mHeight = 480;
            grid.mWidth  = aspectWidth(grid.mHeight, viewport_aspect);
            break;
        case GRID_ASPECT_360:
        default:
            grid.mHeight = 360;
            grid.mWidth  = aspectWidth(grid.mHeight, viewport_aspect);
            break;
    }

    F32 vertex_scale = 0.f;
    switch (vertex_grid_mode)
    {
        case VERTEX_GRID_FINE:
            vertex_scale = 2.f;
            break;
        case VERTEX_GRID_MATCH:
            vertex_scale = 1.f;
            break;
        case VERTEX_GRID_COARSE:
            vertex_scale = 0.5f;
            break;
        case VERTEX_GRID_OFF:
        default:
            break;
    }

    if (vertex_scale > 0.f)
    {
        // A fixed 4:3 raster samples only the centered source rectangle on a
        // wider or taller display. Convert that cropped grid to its equivalent
        // full-viewport density so the vertex lattice and sampled cells agree.
        grid.mVertexQuantization = true;
        grid.mVertexGridWidth = llmax(1, static_cast<S32>(std::lround(
            static_cast<F32>(grid.mWidth) * vertex_scale / grid.mSourceRect.mWidth)));
        grid.mVertexGridHeight = llmax(1, static_cast<S32>(std::lround(
            static_cast<F32>(grid.mHeight) * vertex_scale / grid.mSourceRect.mHeight)));
    }

    return grid;
}

S32 ALRasterPrecision::orderedDitherOffset(S32 x, S32 y)
{
    static constexpr S32 MATRIX[16] = {
        -4, 0, -3, 1, 2, -2, 3, -1, -3, 1, -4, 0, 3, -1, 2, -2,
    };

    const S32 column = ((x % 4) + 4) % 4;
    const S32 row    = ((y % 4) + 4) % 4;
    return MATRIX[row * 4 + column];
}

U8 ALRasterPrecision::quantizeRGB555(U8 channel, S32 x, S32 y, bool ordered_dither)
{
    const S32 offset   = ordered_dither ? orderedDitherOffset(x, y) : 0;
    const S32 adjusted = llclamp(static_cast<S32>(channel) + offset, 0, 255);
    const S32 five_bit = adjusted / 8;
    return static_cast<U8>((five_bit << 3) | (five_bit >> 2));
}
