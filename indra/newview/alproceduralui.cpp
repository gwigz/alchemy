#include "llviewerprecompiledheaders.h"

#include "alproceduralui.h"

#include <cstring>

#include "lldir.h"
#include "llfile.h"
#include "llglslshader.h"
#include "llinitparam.h"
#include "llrender.h"
#include "llrender2dutils.h"
#include "llshadermgr.h"
#include "lltexture.h"
#include "lluicolortable.h"
#include "lluictrlfactory.h"
#include "llviewertexturelist.h"
#include "llvertexbuffer.h"
#include "llxmlnode.h"
#include "v4coloru.h"
#include "v4math.h"

LLGLSLShader gSDFShapeProgram;

namespace
{
    constexpr U32 SHAPE_MASK =
        LLVertexBuffer::MAP_VERTEX |
        LLVertexBuffer::MAP_TEXCOORD0 |
        LLVertexBuffer::MAP_TEXCOORD1 |
        LLVertexBuffer::MAP_TEXCOORD2 |
        LLVertexBuffer::MAP_NORMAL |
        LLVertexBuffer::MAP_TANGENT |
        LLVertexBuffer::MAP_COLOR |
        LLVertexBuffer::MAP_WEIGHT4 |
        LLVertexBuffer::MAP_CLOTHWEIGHT;

    constexpr S32 QUAD_VERTS = 4;
    constexpr S32 QUAD_INDICES = 6;
    constexpr F32 AA_MARGIN = 2.f;
    constexpr size_t MAX_QUADS = 2048;

    ALProceduralUIImage::ScrollArrow scrollArrowFromName(const std::string& name)
    {
        if (name == "ScrollArrow_Up")    return ALProceduralUIImage::ScrollArrow::UP;
        if (name == "ScrollArrow_Down")  return ALProceduralUIImage::ScrollArrow::DOWN;
        if (name == "ScrollArrow_Left")  return ALProceduralUIImage::ScrollArrow::LEFT;
        if (name == "ScrollArrow_Right") return ALProceduralUIImage::ScrollArrow::RIGHT;
        return ALProceduralUIImage::ScrollArrow::NONE;
    }

    void drawScrollArrowCaret(S32 x, S32 y, S32 width, S32 height, ALProceduralUIImage::ScrollArrow arrow, F32 alpha)
    {
        if (arrow == ALProceduralUIImage::ScrollArrow::NONE || width <= 0 || height <= 0)
        {
            return;
        }

        const F32 cx = x + width * 0.5f;
        const F32 cy = y + height * 0.5f;
        const F32 half = llmax(3.f, llmin((F32)width, (F32)height) * 0.28f);
        const F32 inset = half * 0.55f;
        LLVector2 a, b, c;
        switch (arrow)
        {
            case ALProceduralUIImage::ScrollArrow::UP:
                a.set(cx, cy + inset);
                b.set(cx - half, cy - inset);
                c.set(cx + half, cy - inset);
                break;
            case ALProceduralUIImage::ScrollArrow::DOWN:
                a.set(cx, cy - inset);
                b.set(cx - half, cy + inset);
                c.set(cx + half, cy + inset);
                break;
            case ALProceduralUIImage::ScrollArrow::LEFT:
                a.set(cx - inset, cy);
                b.set(cx + inset, cy + half);
                c.set(cx + inset, cy - half);
                break;
            case ALProceduralUIImage::ScrollArrow::RIGHT:
                a.set(cx + inset, cy);
                b.set(cx - inset, cy + half);
                c.set(cx - inset, cy - half);
                break;
            default:
                return;
        }

        gl_triangle_2d((S32)llround(a.mV[VX]), (S32)llround(a.mV[VY]),
                       (S32)llround(b.mV[VX]), (S32)llround(b.mV[VY]),
                       (S32)llround(c.mV[VX]), (S32)llround(c.mV[VY]),
                       LLColor4(1.f, 1.f, 1.f, 0.28f * llclamp(alpha, 0.f, 1.f)),
                       true);
    }

    LLColor4 resolveToken(bool provided, const std::string& token, const LLColor4& fallback)
    {
        if (!provided)
        {
            return fallback;
        }
        return LLUIColorTable::instance().getColor(token).get();
    }

    struct ShapeParams : public LLInitParam::Block<ShapeParams>
    {
        Mandatory<std::string> name;
        Optional<std::string>  kind;
        Optional<F32>          radius;
        Optional<F32>          radius_top;
        Optional<F32>          radius_bottom;
        Optional<F32>          radius_left;
        Optional<F32>          radius_right;
        Optional<F32>          inset;
        Optional<S32>          width;
        Optional<S32>          height;
        Optional<std::string>  fill;
        Optional<LLColor4>     fill_color;
        Optional<std::string>  border;
        Optional<LLColor4>     border_color;
        Optional<F32>          border_width;
        Optional<std::string>  shadow;
        Optional<LLColor4>     shadow_color;
        Optional<F32>          shadow_blur;
        Optional<F32>          shadow_offset_x;
        Optional<F32>          shadow_offset_y;

        ShapeParams()
        :   name("name"),
            kind("kind"),
            radius("radius"),
            radius_top("radius_top"),
            radius_bottom("radius_bottom"),
            radius_left("radius_left"),
            radius_right("radius_right"),
            inset("inset"),
            width("width"),
            height("height"),
            fill("fill"),
            fill_color("fill_color"),
            border("border"),
            border_color("border_color"),
            border_width("border_width"),
            shadow("shadow"),
            shadow_color("shadow_color"),
            shadow_blur("shadow_blur"),
            shadow_offset_x("shadow_offset_x"),
            shadow_offset_y("shadow_offset_y")
        {
        }
    };

    struct ShapesFile : public LLInitParam::Block<ShapesFile>
    {
        Multiple<ShapeParams> shapes;

        ShapesFile()
        :   shapes("shape")
        {
        }
    };

    U32 readBE32(const U8* bytes)
    {
        return ((U32)bytes[0] << 24) | ((U32)bytes[1] << 16) | ((U32)bytes[2] << 8) | (U32)bytes[3];
    }

    bool readTextureHeader(const std::string& filename, U8 (&header)[24])
    {
        const std::string full_path = gDirUtilp->findSkinnedFilename(LLDir::TEXTURES, filename);
        if (full_path.empty())
        {
            return false;
        }

        llifstream file(full_path.c_str(), std::ios::in | std::ios::binary);
        return file.read((char*)header, sizeof(header)).good();
    }

    bool readPngDimensions(const std::string& filename, S32& width, S32& height)
    {
        static const U8 PNG_SIGNATURE[8] = { 0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n' };

        U8 header[24] = {};
        if (!readTextureHeader(filename, header) ||
            memcmp(header, PNG_SIGNATURE, sizeof(PNG_SIGNATURE)) != 0 ||
            memcmp(header + 12, "IHDR", 4) != 0)
        {
            return false;
        }

        width = (S32)readBE32(header + 16);
        height = (S32)readBE32(header + 20);
        return width > 0 && height > 0;
    }

    bool readJ2CDimensions(const std::string& filename, S32& width, S32& height)
    {
        U8 header[24] = {};
        if (!readTextureHeader(filename, header) ||
            header[0] != 0xff || header[1] != 0x4f ||
            header[2] != 0xff || header[3] != 0x51)
        {
            return false;
        }

        const U32 xsiz = readBE32(header + 8);
        const U32 ysiz = readBE32(header + 12);
        const U32 xosiz = readBE32(header + 16);
        const U32 yosiz = readBE32(header + 20);
        if (xsiz <= xosiz || ysiz <= yosiz)
        {
            return false;
        }

        width = (S32)(xsiz - xosiz);
        height = (S32)(ysiz - yosiz);
        return true;
    }

    bool readImageDimensions(const std::string& filename, S32& width, S32& height)
    {
        return readPngDimensions(filename, width, height) || readJ2CDimensions(filename, width, height);
    }
}

// ============================================================================
// ALShapeBatch
// ============================================================================

ALShapeBatch& ALShapeBatch::instance()
{
    static ALShapeBatch sInstance;
    return sInstance;
}

void ALShapeBatch::flushHook()
{
    instance().flush();
}

bool ALShapeBatch::hasPendingHook()
{
    return instance().hasPending();
}

void ALShapeBatch::emit(const LLRect& rect, const ALShapeStyle& style, const LLColor4& modulate)
{
    if (rect.getWidth() <= 0 || rect.getHeight() <= 0)
    {
        return;
    }

    if (style.inset > 0.f &&
        style.borderWidth > 0.f &&
        style.fill.mV[VALPHA] > 0.f &&
        style.border.mV[VALPHA] > 0.f)
    {
        ALShapeStyle outer = style;
        outer.inset = 0.f;
        outer.fill = LLColor4::transparent;
        emit(rect, outer, modulate);

        ALShapeStyle inner = style;
        inner.border = LLColor4::transparent;
        inner.borderWidth = 0.f;
        inner.shadowColor = LLColor4::transparent;
        emit(rect, inner, modulate);
        return;
    }

    LLColor4 fill = style.fill;
    LLColor4 border = style.border;
    LLColor4 shadow = style.shadowColor;
    for (S32 i = 0; i < 4; ++i)
    {
        fill.mV[i] *= modulate.mV[i];
        border.mV[i] *= modulate.mV[i];
        shadow.mV[i] *= modulate.mV[i];
    }

    if (!gSDFShapeProgram.mProgramObject)
    {
        if (fill.mV[VALPHA] > 0.f)
        {
            gl_rect_2d(rect, fill, true);
        }
        return;
    }

    const LLVector3 ui_scale = gGL.getUIScale();
    const LLVector3 ui_trans = gGL.getUITranslation();
    const F32 sx = ui_scale.mV[VX];
    const F32 sy = ui_scale.mV[VY];

    const F32 cx = (ui_trans.mV[VX] + (rect.mLeft + rect.mRight) * 0.5f) * sx;
    const F32 cy = (ui_trans.mV[VY] + (rect.mBottom + rect.mTop) * 0.5f) * sy;

    const F32 half_w = llmax(0.f, rect.getWidth() * 0.5f - style.inset) * sx;
    const F32 half_h = llmax(0.f, rect.getHeight() * 0.5f - style.inset) * sy;
    const F32 blur = style.castsShadow() ? style.shadowBlur * sx : 0.f;
    const F32 off_x = style.shadowOffset.mV[VX] * sx;
    const F32 off_y = style.shadowOffset.mV[VY] * sy;

    // Levien shadow falloff reaches ~0 near 3x blur; grow the quad to contain it.
    const F32 grow_x = AA_MARGIN + blur * 3.f + llabs(off_x);
    const F32 grow_y = AA_MARGIN + blur * 3.f + llabs(off_y);

    Quad q;
    q.x0 = cx - half_w - grow_x;
    q.x1 = cx + half_w + grow_x;
    q.y0 = cy - half_h - grow_y;
    q.y1 = cy + half_h + grow_y;
    q.cx = cx;
    q.cy = cy;
    q.half.set(half_w, half_h);
    q.shape.set(0.f, style.borderWidth * sx);
    F32 cTL = style.cornerRadius, cTR = style.cornerRadius, cBR = style.cornerRadius, cBL = style.cornerRadius;
    if (style.radiusTop >= 0.f)    { cTL = cTR = style.radiusTop; }
    if (style.radiusBottom >= 0.f) { cBL = cBR = style.radiusBottom; }
    if (style.radiusLeft >= 0.f)   { cTL = cBL = style.radiusLeft; }
    if (style.radiusRight >= 0.f)  { cTR = cBR = style.radiusRight; }
    q.radii.set(cTR * sx, cBR * sx, cTL * sx, cBL * sx);
    q.shadow.set(blur, off_x, off_y);
    q.fill = fill;
    q.borderColor = border;
    q.shadowColor = shadow;
    mQuads.push_back(q);

    if (mQuads.size() >= MAX_QUADS)
    {
        flush();
    }
}

void ALShapeBatch::flush()
{
    if (mFlushing || mQuads.empty())
    {
        return;
    }
    if (!gSDFShapeProgram.mProgramObject)
    {
        mQuads.clear();
        return;
    }
    mFlushing = true;

    const U32 vert_count = (U32)mQuads.size() * QUAD_VERTS;
    const U32 index_count = (U32)mQuads.size() * QUAD_INDICES;
    if (mBuffer.isNull() || mCapacityVerts < vert_count)
    {
        mBuffer = new LLVertexBuffer(SHAPE_MASK);
        if (!mBuffer->allocateBuffer(vert_count, index_count))
        {
            mBuffer = nullptr;
            mCapacityVerts = 0;
            mQuads.clear();
            mFlushing = false;
            return;
        }
        mCapacityVerts = vert_count;
    }

    LLStrider<LLVector3> pos;
    LLStrider<LLVector2> local;
    LLStrider<LLVector2> halfsize;
    LLStrider<LLVector2> shape;
    LLStrider<LLVector4> radii;
    LLStrider<LLVector3> shadow_params;
    LLStrider<LLVector4a> border_color;
    LLStrider<LLColor4U> fill_color;
    LLStrider<LLVector4> shadow_color;
    LLStrider<U16> indices;
    if (!mBuffer->getVertexStrider(pos) ||
        !mBuffer->getTexCoord0Strider(local) ||
        !mBuffer->getTexCoord1Strider(halfsize) ||
        !mBuffer->getTexCoord2Strider(shape) ||
        !mBuffer->getClothWeightStrider(radii) ||
        !mBuffer->getNormalStrider(shadow_params) ||
        !mBuffer->getTangentStrider(border_color) ||
        !mBuffer->getColorStrider(fill_color) ||
        !mBuffer->getWeight4Strider(shadow_color) ||
        !mBuffer->getIndexStrider(indices))
    {
        mQuads.clear();
        mFlushing = false;
        return;
    }

    S32 v = 0;
    for (const Quad& q : mQuads)
    {
        const LLVector2 corners[QUAD_VERTS] = {
            LLVector2(q.x0, q.y0), LLVector2(q.x1, q.y0),
            LLVector2(q.x1, q.y1), LLVector2(q.x0, q.y1)
        };

        const LLColor4U fill_u = q.fill;
        LLVector4a border_rgba;
        border_rgba.set(q.borderColor.mV[VRED], q.borderColor.mV[VGREEN], q.borderColor.mV[VBLUE], q.borderColor.mV[VALPHA]);
        const LLVector4 shadow_rgba(q.shadowColor.mV[VRED], q.shadowColor.mV[VGREEN], q.shadowColor.mV[VBLUE], q.shadowColor.mV[VALPHA]);

        for (S32 i = 0; i < QUAD_VERTS; ++i, ++v)
        {
            pos[v].set(corners[i].mV[VX], corners[i].mV[VY], 0.f);
            local[v].set(corners[i].mV[VX] - q.cx, corners[i].mV[VY] - q.cy);
            halfsize[v] = q.half;
            shape[v] = q.shape;
            radii[v] = q.radii;
            shadow_params[v] = q.shadow;
            border_color[v] = border_rgba;
            fill_color[v] = fill_u;
            shadow_color[v] = shadow_rgba;
        }

        const U16 base = (U16)(v - QUAD_VERTS);
        *indices++ = base + 0;
        *indices++ = base + 1;
        *indices++ = base + 2;
        *indices++ = base + 0;
        *indices++ = base + 2;
        *indices++ = base + 3;
    }

    mBuffer->unmapBuffer();

    LLGLSLShader* prev = LLGLSLShader::sCurBoundShaderPtr;
    gSDFShapeProgram.bind();
    mBuffer->setBuffer();
    mBuffer->drawRange(LLRender::TRIANGLES, 0, vert_count - 1, index_count, 0);

    if (prev)
    {
        prev->bind();
    }
    else
    {
        gSDFShapeProgram.unbind();
    }

    mQuads.clear();
    mFlushing = false;
}

// ============================================================================
// ALShapeRegistry
// ============================================================================

bool ALShapeRegistry::loadFromSettings()
{
    mRecipes.clear();

    bool result = false;
    for (const std::string& path :
         gDirUtilp->findSkinnedFilenames(LLDir::SKINBASE, "shapes.xml", LLDir::ALL_SKINS))
    {
        result |= loadFromFilename(path);
    }
    return result;
}

bool ALShapeRegistry::loadFromFilename(const std::string& filename)
{
    LLXMLNodePtr root;
    if (!LLXMLNode::parseFile(filename, root, NULL))
    {
        LL_WARNS() << "Unable to parse shape file " << filename << LL_ENDL;
        return false;
    }

    if (!root->hasName("shapes"))
    {
        LL_WARNS() << filename << " is not a valid shape definition file" << LL_ENDL;
        return false;
    }

    ShapesFile params;
    LLXUIParser parser;
    parser.readXUI(root, params, filename);

    if (!params.validateBlock())
    {
        LL_WARNS() << filename << " failed to load" << LL_ENDL;
        return false;
    }

    for (LLInitParam::ParamIterator<ShapeParams>::const_iterator it = params.shapes.begin();
         it != params.shapes.end(); ++it)
    {
        const ShapeParams& sp = *it;

        if (sp.kind.isProvided() && sp.kind() != "rounded_rect")
        {
            LL_WARNS() << filename << ": shape '" << sp.name()
                       << "' uses unsupported kind '" << sp.kind()
                       << "', treating as rounded_rect" << LL_ENDL;
        }

        ALShapeStyle style;
        style.cornerRadius = sp.radius();
        style.radiusTop = sp.radius_top.isProvided() ? sp.radius_top() : -1.f;
        style.radiusBottom = sp.radius_bottom.isProvided() ? sp.radius_bottom() : -1.f;
        style.radiusLeft = sp.radius_left.isProvided() ? sp.radius_left() : -1.f;
        style.radiusRight = sp.radius_right.isProvided() ? sp.radius_right() : -1.f;
        style.borderWidth = sp.border_width();
        style.inset = sp.inset();
        style.intrinsicWidth = sp.width();
        style.intrinsicHeight = sp.height();
        style.fill = sp.fill_color.isProvided()
            ? sp.fill_color()
            : resolveToken(sp.fill.isProvided(), sp.fill(), LLColor4::transparent);
        style.border = sp.border_color.isProvided()
            ? sp.border_color()
            : resolveToken(sp.border.isProvided(), sp.border(), LLColor4::transparent);
        style.shadowColor = sp.shadow_color.isProvided()
            ? sp.shadow_color()
            : resolveToken(sp.shadow.isProvided(), sp.shadow(), LLColor4(0.f, 0.f, 0.f, 0.f));
        style.shadowBlur = sp.shadow_blur();
        style.shadowOffset.set(sp.shadow_offset_x(), sp.shadow_offset_y());

        mRecipes[sp.name()] = style;
    }

    return true;
}

const ALShapeStyle* ALShapeRegistry::getRecipe(const std::string& name) const
{
    auto it = mRecipes.find(name);
    return it != mRecipes.end() ? &it->second : nullptr;
}

// ============================================================================
// ALProceduralUIImage
// ============================================================================

ALProceduralUIImage::ALProceduralUIImage(const std::string& name, const LLPointer<LLUIImage>& source, const ALShapeStyle& style)
:   LLUIImage(name, NULL),
    mStyle(style),
    mWidth(source->getWidth()),
    mHeight(source->getHeight()),
    mScrollArrow(scrollArrowFromName(name))
{
    copyLayoutFrom(*source);
    mCachedW = mWidth;
    mCachedH = mHeight;
}

ALProceduralUIImage::ALProceduralUIImage(const std::string& name, S32 width, S32 height, const LLRect& scale_rect,
                                         const LLRect& clip_rect, LLUIImage::EScaleStyle scale_style, const ALShapeStyle& style)
:   LLUIImage(name, NULL),
    mStyle(style),
    mWidth(clip_rect != LLRect::null ? clip_rect.getWidth() : width),
    mHeight(clip_rect != LLRect::null ? clip_rect.getHeight() : height),
    mScrollArrow(scrollArrowFromName(name))
{
    mCachedW = mWidth;
    mCachedH = mHeight;
    setScaleStyle(scale_style);

    const F32 full_width = (F32)llmax(width, 1);
    const F32 full_height = (F32)llmax(height, 1);
    if (clip_rect != LLRect::null)
    {
        setClipRegion(LLRectf(llclamp((F32)clip_rect.mLeft / full_width, 0.f, 1.f),
                              llclamp((F32)clip_rect.mTop / full_height, 0.f, 1.f),
                              llclamp((F32)clip_rect.mRight / full_width, 0.f, 1.f),
                              llclamp((F32)clip_rect.mBottom / full_height, 0.f, 1.f)));
    }
    if (scale_rect != LLRect::null)
    {
        const F32 visible_width = (F32)llmax(mWidth, 1);
        const F32 visible_height = (F32)llmax(mHeight, 1);
        setScaleRegion(LLRectf(llclamp((F32)scale_rect.mLeft / visible_width, 0.f, 1.f),
                               llclamp((F32)scale_rect.mTop / visible_height, 0.f, 1.f),
                               llclamp((F32)scale_rect.mRight / visible_width, 0.f, 1.f),
                               llclamp((F32)scale_rect.mBottom / visible_height, 0.f, 1.f)));
    }
}

void ALProceduralUIImage::draw(S32 x, S32 y, S32 width, S32 height, const LLColor4& color) const
{
    drawWithStyle(x, y, width, height, color, mStyle);
}

void ALProceduralUIImage::drawWithStyle(S32 x, S32 y, S32 width, S32 height, const LLColor4& color, const ALShapeStyle& style) const
{
    LLRect rect;
    rect.setOriginAndSize(x, y, width, height);
    ALShapeBatch::instance().emit(rect, style, color);
    if (mScrollArrow != ScrollArrow::NONE)
    {
        ALShapeBatch::instance().flush();
        drawScrollArrowCaret(x, y, width, height, mScrollArrow, color.mV[VALPHA]);
    }
}

void ALProceduralUIImage::drawSolid(S32 x, S32 y, S32 width, S32 height, const LLColor4& color) const
{
    draw(x, y, width, height, color);
}

void ALProceduralUIImage::drawBorder(S32 x, S32 y, S32 width, S32 height, const LLColor4& color, S32 border_width) const
{
    ALShapeStyle style = mStyle;
    style.fill = LLColor4::transparent;
    style.shadowColor = LLColor4(0.f, 0.f, 0.f, 0.f);
    style.shadowBlur = 0.f;
    style.border = color;
    style.borderWidth = (F32)llmax(border_width, 1);
    LLRect rect;
    rect.setOriginAndSize(x, y, width, height);
    ALShapeBatch::instance().emit(rect, style, LLColor4::white);
}

void ALProceduralUIImage::drawShaped(S32 x, S32 y, S32 width, S32 height, const LLColor4& color, const LLUIImageShape& shape) const
{
    ALShapeStyle style = mStyle;
    if (shape.radius_top >= 0.f)    style.radiusTop = shape.radius_top;
    if (shape.radius_bottom >= 0.f) style.radiusBottom = shape.radius_bottom;
    if (shape.radius_left >= 0.f)   style.radiusLeft = shape.radius_left;
    if (shape.radius_right >= 0.f)  style.radiusRight = shape.radius_right;
    if (shape.inset >= 0.f)         style.inset = shape.inset;
    drawWithStyle(x, y, width, height, color, style);
}

// ============================================================================
// ALProceduralImageProvider
// ============================================================================

ALProceduralImageProvider::ALProceduralImageProvider(LLImageProviderInterface* wrapped)
:   mWrapped(wrapped)
{
}

LLImageProviderInterface* ALProceduralImageProvider::wrap(LLImageProviderInterface* real)
{
    return new ALProceduralImageProvider(real);
}

LLPointer<LLUIImage> ALProceduralImageProvider::getUIImage(std::string_view name, S32 priority)
{
    const std::string name_str(name);
    const ALShapeStyle* style = ALShapeRegistry::instance().getRecipe(name_str);
    if (!style)
    {
        return mWrapped->getUIImage(name, priority);
    }

    auto cached = mCache.find(name_str);
    if (cached != mCache.end())
    {
        return cached->second;
    }

    LLPointer<LLUIImage> real = mWrapped->getUIImage(name, priority);
    if (real.notNull() && dynamic_cast<ALProceduralUIImage*>(real.get()))
    {
        mCache[name_str] = real;
        return real;
    }
    if (real.isNull() || real->getImage().isNull())
    {
        return real;
    }

    LLPointer<LLUIImage> procedural = new ALProceduralUIImage(name_str, real, *style);
    if (LLUIImageList* image_list = dynamic_cast<LLUIImageList*>(mWrapped))
    {
        image_list->releaseUIImageTexture(name_str);
    }
    else
    {
        real->releaseImage();
    }
    mCache[name_str] = procedural;
    return procedural;
}

LLPointer<LLUIImage> ALProceduralImageProvider::getUIImageByID(const LLUUID& id, S32 priority)
{
    return mWrapped->getUIImageByID(id, priority);
}

void ALProceduralImageProvider::cleanUp()
{
    mCache.clear();
    mWrapped->cleanUp();
}

// ============================================================================
// ALUIImageResolver
// ============================================================================

ALUIImageResolution ALUIImageResolver::resolve(const ALUIImageSource& source)
{
    ALUIImageResolution resolution;

    if (const ALShapeStyle* style = ALShapeRegistry::instance().getRecipe(source.name))
    {
        S32 width = style->intrinsicWidth;
        S32 height = style->intrinsicHeight;

        // A recipe that bakes its own size needs no legacy raster on disk; only
        // fall back to a header read when the recipe omits intrinsic dimensions.
        if (width <= 0 || height <= 0)
        {
            readImageDimensions(source.fileName, width, height);
        }

        if (width > 0 && height > 0)
        {
            resolution.kind = ALUIImageResolution::SHAPE;
            resolution.image = new ALProceduralUIImage(
                source.name, width, height, source.scale, source.clip, source.scaleStyle, *style);
            return resolution;
        }

        LL_WARNS("ViewerImages") << "Failed to size procedural shape "
                                 << source.name << ", falling back to " << source.fileName << LL_ENDL;
    }

    resolution.kind = ALUIImageResolution::BITMAP;
    return resolution;
}
