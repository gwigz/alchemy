/**
 * @file alrasterprecision_test.cpp
 * @brief Unit tests for low-precision raster grid and RGB555 arithmetic.
 *
 * Copyright (c) 2026, Alchemy Viewer Project.
 *
 * This source is licensed under the GNU Lesser General Public License,
 * version 2.1.
 */

#include "linden_common.h"

#include "../test/lltut.h"

#include "../alrasterprecision.h"

namespace tut
{
struct raster_precision_data
{
};

typedef test_group<raster_precision_data> raster_precision_group;
typedef raster_precision_group::object    raster_precision_object;
raster_precision_group                    raster_precision_tests("ALRasterPrecision");

template<>
template<>
void raster_precision_object::test<1>()
{
    const S32 expected[16] = {
        -4, 0, -3, 1, 2, -2, 3, -1, -3, 1, -4, 0, 3, -1, 2, -2,
    };

    for (S32 y = 0; y < 4; ++y)
    {
        for (S32 x = 0; x < 4; ++x)
        {
            ensure_equals("exact ordered-dither entry", ALRasterPrecision::orderedDitherOffset(x, y), expected[y * 4 + x]);
            ensure_equals("matrix repeats", ALRasterPrecision::orderedDitherOffset(x + 8, y + 12), expected[y * 4 + x]);
        }
    }
}

template<>
template<>
void raster_precision_object::test<2>()
{
    ensure_equals("black clamps", ALRasterPrecision::quantizeRGB555(0, 0, 0, true), U8(0));
    ensure_equals("white clamps", ALRasterPrecision::quantizeRGB555(255, 3, 0, true), U8(255));
    ensure_equals("without dither uses five-bit truncation", ALRasterPrecision::quantizeRGB555(127, 0, 0, false), U8(123));
    ensure_equals("positive dither reaches next five-bit level", ALRasterPrecision::quantizeRGB555(127, 2, 1, true), U8(132));
    ensure_equals("negative dither stays in lower five-bit level", ALRasterPrecision::quantizeRGB555(127, 0, 0, true), U8(123));
}

template<>
template<>
void raster_precision_object::test<3>()
{
    const ALRasterGrid grid =
        ALRasterPrecision::resolve(1280, 960, ALRasterPrecision::PROFILE_REFERENCE, ALRasterPrecision::GRID_ASPECT_480,
                                   ALRasterPrecision::VERTEX_GRID_OFF, false, false);

    ensure_equals("reference width", grid.mWidth, 320);
    ensure_equals("reference height", grid.mHeight, 240);
    ensure_approximately_equals("reference fills 4:3 viewport", grid.mDisplayRect.mWidth, 1.f, 5);
    ensure("reference enables RGB555", grid.mRGB555);
    ensure("reference enables ordered dither", grid.mOrderedDither);
    ensure("reference enables vertex quantization", grid.mVertexQuantization);
    ensure_equals("reference vertex width", grid.mVertexGridWidth, 320);
    ensure_equals("reference vertex height", grid.mVertexGridHeight, 240);
}

template<>
template<>
void raster_precision_object::test<4>()
{
    const ALRasterGrid grid =
        ALRasterPrecision::resolve(1920, 1080, ALRasterPrecision::PROFILE_REFERENCE, ALRasterPrecision::GRID_ASPECT_240,
                                   ALRasterPrecision::VERTEX_GRID_OFF, false, false);

    ensure_approximately_equals("wide reference display width", grid.mDisplayRect.mWidth, 0.75f, 5);
    ensure_approximately_equals("wide reference display left", grid.mDisplayRect.mX, 0.125f, 5);
    ensure_approximately_equals("wide reference source width", grid.mSourceRect.mWidth, 0.75f, 5);
    ensure_approximately_equals("wide reference source left", grid.mSourceRect.mX, 0.125f, 5);
    ensure_equals("wide reference vertex width compensates for crop", grid.mVertexGridWidth, 427);
    ensure_equals("wide reference vertex height", grid.mVertexGridHeight, 240);
}

template<>
template<>
void raster_precision_object::test<5>()
{
    const ALRasterGrid contemporary =
        ALRasterPrecision::resolve(1920, 1080, ALRasterPrecision::PROFILE_CONTEMPORARY, ALRasterPrecision::GRID_FIXED_320_240,
                                   ALRasterPrecision::VERTEX_GRID_OFF, true, true);
    ensure_equals("contemporary height", contemporary.mHeight, 360);
    ensure_equals("contemporary aspect-fit width", contemporary.mWidth, 640);
    ensure("contemporary keeps full color", !contemporary.mRGB555);
    ensure("contemporary has no ordered dither", !contemporary.mOrderedDither);
    ensure_equals("contemporary fine vertex width", contemporary.mVertexGridWidth, 1280);
    ensure_equals("contemporary fine vertex height", contemporary.mVertexGridHeight, 720);

    const ALRasterGrid custom =
        ALRasterPrecision::resolve(2560, 1080, ALRasterPrecision::PROFILE_CUSTOM, ALRasterPrecision::GRID_ASPECT_480,
                                   ALRasterPrecision::VERTEX_GRID_COARSE, true, true);
    ensure_equals("custom height", custom.mHeight, 480);
    ensure_equals("custom aspect-fit width", custom.mWidth, 1138);
    ensure("custom enables RGB555", custom.mRGB555);
    ensure("custom enables ordered dither", custom.mOrderedDither);
    ensure_equals("custom coarse vertex width", custom.mVertexGridWidth, 569);
    ensure_equals("custom coarse vertex height", custom.mVertexGridHeight, 240);
}


template<>
template<>
void raster_precision_object::test<6>()
{
    const ALRasterGrid custom =
        ALRasterPrecision::resolve(1280, 720, ALRasterPrecision::PROFILE_CUSTOM, ALRasterPrecision::GRID_ASPECT_360,
                                   ALRasterPrecision::VERTEX_GRID_OFF, false, true);
    ensure("custom can disable vertex quantization", !custom.mVertexQuantization);
    ensure_equals("disabled vertex width is safe", custom.mVertexGridWidth, 1);
    ensure_equals("disabled vertex height is safe", custom.mVertexGridHeight, 1);
    ensure("dither cannot run without RGB555", !custom.mOrderedDither);
}
} // namespace tut
