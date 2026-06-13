uniform mat4 modelview_projection_matrix;

in vec3 position;
in vec2 texcoord0;
in vec2 texcoord1;
in vec2 texcoord2;
in vec3 normal;
in vec4 tangent;
in vec4 diffuse_color;
in vec4 weight4;
in vec4 clothing;

out vec2 vary_local;
out vec2 vary_halfsize;
out vec2 vary_shape;
out vec3 vary_shadow;
out vec4 vary_radii;
out vec4 vary_fill;
out vec4 vary_border;
out vec4 vary_shadowcol;

void main()
{
    gl_Position = modelview_projection_matrix * vec4(position, 1.0);
    vary_local = texcoord0;
    vary_halfsize = texcoord1;
    vary_shape = texcoord2;
    vary_shadow = normal;
    vary_radii = clothing;
    vary_fill = diffuse_color;
    vary_border = tangent;
    vary_shadowcol = weight4;
}
