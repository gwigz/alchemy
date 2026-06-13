out vec4 frag_color;

in vec2 vary_local;
in vec2 vary_halfsize;
in vec2 vary_shape;
in vec3 vary_shadow;
in vec4 vary_radii;
in vec4 vary_fill;
in vec4 vary_border;
in vec4 vary_shadowcol;

// Per-corner radii r = (top-right, bottom-right, top-left, bottom-left).
float sdRoundedBox(vec2 p, vec2 b, vec4 r)
{
    r = min(r, vec4(min(b.x, b.y)));
    r.xy = (p.x > 0.0) ? r.xy : r.zw;
    r.x  = (p.y > 0.0) ? r.x : r.y;
    vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

float erf7(float x)
{
    x *= 1.1283791670955126;
    float xx = x * x;
    x += (0.24295 + (0.03395 + 0.0104 * xx) * xx) * (x * xx);
    return x * inversesqrt(1.0 + x * x);
}

float blurredRoundedBox(vec2 p, vec2 b, vec4 radii, float blur)
{
    float blur_radius = max(blur, 0.001);
    vec4 r = sqrt(radii * radii + 1.25 * blur_radius * blur_radius);
    float d = sdRoundedBox(p, b, r);
    return clamp(0.5 - 0.5 * erf7(d / (blur_radius * 1.4142135623730951)), 0.0, 1.0);
}

void main()
{
    vec2 b = vary_halfsize;
    float border = max(vary_shape.y, 0.0);

    float d = sdRoundedBox(vary_local, b, vary_radii);
    float aa = max(fwidth(d), 1e-4);

    float outer = 1.0 - smoothstep(-aa, aa, d);
    float inner = 1.0 - smoothstep(-aa, aa, d + border);
    float ring  = max(outer - inner, 0.0);

    float surface_opacity = clamp(vary_fill.a, 0.0, 1.0);
    vec4 fillP = vec4(vary_fill.rgb * surface_opacity, surface_opacity) * inner;

    float border_alpha = vary_border.a;
    vec4 bordP = vec4(vary_border.rgb * border_alpha, border_alpha) * ring;

    vec4 outP = fillP;
    outP = bordP + outP * (1.0 - bordP.a);

    float blur = vary_shadow.x;
    if (blur > 0.0)
    {
        // Box shadow matching the rounded box silhouette.
        float scov = blurredRoundedBox(vary_local - vary_shadow.yz, b, vary_radii, blur);
        vec4 shadP = vec4(vary_shadowcol.rgb * vary_shadowcol.a, vary_shadowcol.a) * scov;
        outP += shadP * (1.0 - outP.a);
    }

    frag_color = (outP.a > 0.0) ? vec4(outP.rgb / outP.a, outP.a) : vec4(0.0);
}
