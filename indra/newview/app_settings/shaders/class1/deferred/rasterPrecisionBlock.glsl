/**
 * Screen-space vertex quantization shared by the main world-material vertex
 * shaders. This is spliced only into those programs; UI, HUD, sky, water,
 * post-processing, reflection, impostor-bake, and shadow programs stay clean.
 */

uniform bool uRasterVertexEnabled;
uniform vec2 uRasterVertexGridSize;

vec4 quantizeRasterVertex(vec4 clip_position)
{
    if (!uRasterVertexEnabled || clip_position.w <= 0.0)
    {
        return clip_position;
    }

    vec2 normalized = clip_position.xy / clip_position.w;
    vec2 raster_position = (normalized * 0.5 + 0.5) * uRasterVertexGridSize;
    raster_position = floor(raster_position + 0.5);
    normalized = raster_position / uRasterVertexGridSize * 2.0 - 1.0;
    clip_position.xy = normalized * clip_position.w;
    return clip_position;
}
