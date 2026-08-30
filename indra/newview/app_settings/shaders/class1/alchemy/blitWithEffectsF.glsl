/**
 * @file blitWithEffectsF.glsl
 *
 * Final display-space blit. All post-effects (vignette, film grain, CVD
 * compensation, dither, preview overlays) live in postEffectUtilsF.glsl
 * and are auto-linked here via mFeatures.hasPostEffects; this file is just
 * the orchestrator that sequences them in the canonical order.
 *
 * Order matters:
 *   vignette  → CVD compensation → film grain → dither → preview
 * Vignette runs before CVD so the accessibility pass considers the final
 * darkening. Film grain runs after CVD so the grain stays neutral in tint.
 * Dither runs last (before preview) so quantization is resolved against
 * the true final color. It is only active here when the post chain is
 * HDR; in the 8-bit case, colorCorrectF dithers earlier instead. Preview overlays are
 * debug-only.
 *
 * $LicenseInfo:firstyear=2026&license=viewerlgpl$
 * Alchemy Viewer Source Code
 * Copyright © 2026, Rye <rye@alchemyviewer.org>
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
 *
 * $/LicenseInfo$
 */

/*[EXTRA_CODE_HERE]*/

in vec2 vary_fragcoord;
out vec4 frag_color;

// =============================================================================
// Uniforms
// =============================================================================

uniform sampler2D diffuseRect;      // Linear Rec.709 / linear-sRGB.
uniform sampler2D depthMap;

// Reference still: a grabbed frame that the live image is wiped against.
// See LLPipeline::requestReferenceStill.
uniform sampler2D uReferenceStill;
uniform int       uRefWipeMode;     // 0 off, 1 wipe, 2 side by side.
uniform float     uRefWipePos;      // Seam position, 0..1 across the frame.

// Low-precision rasterization. Rectangles are normalized as x, y, width,
// height. The CPU resolves every profile and aspect-ratio rule so this shader
// only consumes one coordinate system.
uniform bool uRasterEnabled;
uniform vec2 uRasterGridSize;
uniform vec4 uRasterDisplayRect;
uniform vec4 uRasterSourceRect;
uniform bool uRasterRGB555;
uniform bool uRasterOrderedDither;

// =============================================================================
// Forward Declarations
// =============================================================================

vec3 clampHDRRange(vec3 color);

// From postEffectUtilsF.glsl (auto-linked via mFeatures.hasPostEffects).
vec3 applyVignette(vec3 color, vec2 uv);
vec3 applyCVDCompensation(vec3 color);
vec3 applyFilmGrain(vec3 color, vec2 fragCoord);
#ifdef DITHER
vec3 applyDither(vec3 color, vec2 fragCoord);
#endif
vec3 applyPreview(vec3 color);

// =============================================================================
// Low-precision rasterization
// =============================================================================

const int RASTER_DITHER_4X4[16] = int[16](
    -4,  0, -3,  1,
     2, -2,  3, -1,
    -3,  1, -4,  0,
     3, -1,  2, -2
);

bool rasterContains(vec2 uv)
{
    vec2 local = (uv - uRasterDisplayRect.xy) / uRasterDisplayRect.zw;
    return all(greaterThanEqual(local, vec2(0.0))) && all(lessThan(local, vec2(1.0)));
}

vec2 rasterCell(vec2 uv)
{
    vec2 local = clamp((uv - uRasterDisplayRect.xy) / uRasterDisplayRect.zw,
                       vec2(0.0), vec2(0.999999));
    return floor(local * uRasterGridSize);
}

vec2 rasterSampleUV(vec2 uv)
{
    vec2 cell = rasterCell(uv);
    vec2 local = (cell + 0.5) / uRasterGridSize;
    return uRasterSourceRect.xy + local * uRasterSourceRect.zw;
}

vec3 applyRasterRGB555(vec3 color, vec2 cell)
{
    ivec2 dither_cell = ivec2(cell) & ivec2(3);
    int index = dither_cell.y * 4 + dither_cell.x;
    float offset = uRasterOrderedDither ? float(RASTER_DITHER_4X4[index]) : 0.0;
    vec3 encoded = clamp(color * 255.0 + offset, vec3(0.0), vec3(255.0));
    return floor(encoded / 8.0) / 31.0;
}

// =============================================================================
// Reference still
// =============================================================================
//
// Substituted for the live sample *before* the print effects below, so
// vignette, grain and dither land on both sides of the seam. That is the
// point: the comparison is then about the grade, not about the print
// treatment, which is what a still is for.
//
// uRefWipeMode is zero whenever there is no still, so the sampler is never
// read unless one has been grabbed.
vec4 sampleWithReference(vec2 uv)
{
    if (uRefWipeMode == 1)
    {
        // Wipe: the still to the left of the seam, live to the right.
        return (uv.x < uRefWipePos) ? texture(uReferenceStill, uv)
                                    : texture(diffuseRect, uv);
    }

    if (uRefWipeMode == 2)
    {
        // Side by side: both squeezed two to one, so the same region of the
        // image appears twice rather than two different halves of it.
        return (uv.x < 0.5) ? texture(uReferenceStill, vec2(uv.x * 2.0, uv.y))
                            : texture(diffuseRect,     vec2((uv.x - 0.5) * 2.0, uv.y));
    }

    return texture(diffuseRect, uv);
}

// A hairline at the seam, so the eye knows which side it is looking at. Drawn
// in screen pixels rather than UV so it stays one line wide at any resolution.
vec3 applyWipeSeam(vec3 color, vec2 uv)
{
    if (uRefWipeMode == 0)
    {
        return color;
    }

    float seam  = (uRefWipeMode == 1) ? uRefWipePos : 0.5;
    float width = fwidth(uv.x);
    float line  = 1.0 - smoothstep(0.0, width, abs(uv.x - seam));
    return mix(color, vec3(1.0), line * 0.75);
}

// =============================================================================
// Main
// =============================================================================
void main()
{
    // === DISPLAY SPACE =======================================================

    vec2 screen_uv = vary_fragcoord.xy;
    vec2 sample_uv = screen_uv;
    vec2 cell = floor(gl_FragCoord.xy);

    if (uRasterEnabled)
    {
        if (!rasterContains(screen_uv))
        {
            frag_color = vec4(0.0, 0.0, 0.0, 1.0);
            gl_FragDepth = texture(depthMap, screen_uv).r;
            return;
        }

        cell = rasterCell(screen_uv);
        sample_uv = rasterSampleUV(screen_uv);
    }

    vec4 diff = sampleWithReference(sample_uv);

    diff.rgb = applyVignette(diff.rgb, screen_uv);
    diff.rgb = applyCVDCompensation(diff.rgb);
    diff.rgb = applyFilmGrain(diff.rgb, uRasterEnabled ? cell : gl_FragCoord.xy);
#ifdef DITHER
    if (!uRasterRGB555)
    {
        diff.rgb = applyDither(diff.rgb, gl_FragCoord.xy);
    }
#endif
    diff.rgb = applyPreview(diff.rgb);   // debug only — no-op when uPreviewMode == 0
    diff.rgb = applyWipeSeam(diff.rgb, screen_uv);

    diff.rgb = clampHDRRange(diff.rgb);
    if (uRasterRGB555)
    {
        diff.rgb = applyRasterRGB555(diff.rgb, cell);
    }
    frag_color = diff;

    // Reverse-Z neutral: copies the raw stored depth value verbatim, no convention math.
    gl_FragDepth = texture(depthMap, screen_uv).r;
}
