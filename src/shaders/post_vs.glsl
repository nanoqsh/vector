const vec2 verts[4] = vec2[4](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

out vec2 uv;

void main() {
    uv = verts[gl_VertexID];
    gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}
