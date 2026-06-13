#pragma once

#include <map>
#include <string>
#include <string_view>
#include <vector>

#include "llpointer.h"
#include "llrect.h"
#include "llsingleton.h"
#include "lluiimage.h"
#include "v2math.h"
#include "v3math.h"
#include "v4color.h"
#include "v4math.h"

class LLGLSLShader;
class LLUUID;
class LLVertexBuffer;

extern LLGLSLShader gSDFShapeProgram;

struct ALShapeStyle
{
    F32 cornerRadius = 0.f;
    F32 radiusTop = -1.f;     // < 0 falls back to cornerRadius
    F32 radiusBottom = -1.f;
    F32 radiusLeft = -1.f;
    F32 radiusRight = -1.f;
    F32 borderWidth = 0.f;
    F32 inset = 0.f;
    LLColor4 fill = LLColor4::transparent;
    LLColor4 border = LLColor4::transparent;
    LLColor4 shadowColor = LLColor4(0.f, 0.f, 0.f, 0.f);
    F32 shadowBlur = 0.f;
    LLVector2 shadowOffset;

    // Intrinsic size baked into the recipe, used to size the procedural image
    // without reading the legacy raster header. 0 falls back to a file read.
    S32 intrinsicWidth = 0;
    S32 intrinsicHeight = 0;

    bool castsShadow() const { return shadowBlur > 0.f && shadowColor.mV[VALPHA] > 0.f; }
};

class ALShapeBatch
{
public:
    static ALShapeBatch& instance();

    void emit(const LLRect& rect, const ALShapeStyle& style, const LLColor4& modulate);

    void flush();
    bool hasPending() const { return !mQuads.empty(); }

    // Registered with LLRender so deferred shapes drain at immediate-geometry
    // barriers without drawing ahead of older gGL vertices.
    static void flushHook();
    static bool hasPendingHook();

private:
    ALShapeBatch() = default;

    struct Quad
    {
        F32 x0, x1, y0, y1, cx, cy;
        LLVector2 half;
        LLVector2 shape;
        LLVector4 radii;
        LLVector3 shadow;
        LLColor4 fill;
        LLColor4 borderColor;
        LLColor4 shadowColor;
    };

    std::vector<Quad> mQuads;
    LLPointer<LLVertexBuffer> mBuffer;
    U32 mCapacityVerts = 0;
    bool mFlushing = false;
};

class ALShapeRegistry : public LLSingleton<ALShapeRegistry>
{
    LLSINGLETON_EMPTY_CTOR(ALShapeRegistry);

public:
    bool loadFromSettings();

    const ALShapeStyle* getRecipe(const std::string& name) const;

private:
    bool loadFromFilename(const std::string& filename);

    std::map<std::string, ALShapeStyle> mRecipes;
};

class ALProceduralUIImage final : public LLUIImage
{
public:
    enum class ScrollArrow
    {
        NONE,
        UP,
        DOWN,
        LEFT,
        RIGHT
    };

    ALProceduralUIImage(const std::string& name, const LLPointer<LLUIImage>& source, const ALShapeStyle& style);
    ALProceduralUIImage(const std::string& name, S32 width, S32 height, const LLRect& scale_rect,
                        const LLRect& clip_rect, LLUIImage::EScaleStyle scale_style, const ALShapeStyle& style);

    void draw(S32 x, S32 y, S32 width, S32 height, const LLColor4& color) const override;
    void drawSolid(S32 x, S32 y, S32 width, S32 height, const LLColor4& color) const override;
    void drawBorder(S32 x, S32 y, S32 width, S32 height, const LLColor4& color, S32 border_width) const override;
    void drawShaped(S32 x, S32 y, S32 width, S32 height, const LLColor4& color, const LLUIImageShape& shape) const override;
    S32 getWidth() const override { return mWidth; }
    S32 getHeight() const override { return mHeight; }

private:
    void drawWithStyle(S32 x, S32 y, S32 width, S32 height, const LLColor4& color, const ALShapeStyle& style) const;

    ALShapeStyle mStyle;
    S32 mWidth = 0;
    S32 mHeight = 0;
    ScrollArrow mScrollArrow = ScrollArrow::NONE;
};

class ALProceduralImageProvider final : public LLImageProviderInterface
{
public:
    static LLImageProviderInterface* wrap(LLImageProviderInterface* real);

    LLPointer<LLUIImage> getUIImage(std::string_view name, S32 priority) override;
    LLPointer<LLUIImage> getUIImageByID(const LLUUID& id, S32 priority) override;
    void cleanUp() override;

private:
    explicit ALProceduralImageProvider(LLImageProviderInterface* wrapped);

    LLImageProviderInterface* mWrapped;
    std::map<std::string, LLPointer<LLUIImage>> mCache;
};

struct ALUIImageSource
{
    std::string name;
    std::string fileName;
    LLRect scale;
    LLRect clip;
    LLUIImage::EScaleStyle scaleStyle = LLUIImage::SCALE_INNER;
};

struct ALUIImageResolution
{
    enum Kind
    {
        SHAPE,
        BITMAP
    };

    Kind kind = BITMAP;
    LLPointer<LLUIImage> image;
};

class ALUIImageResolver
{
public:
    static ALUIImageResolution resolve(const ALUIImageSource& source);
};
