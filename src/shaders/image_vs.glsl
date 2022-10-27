layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex;

uniform mat2 mat;

uniform vec2 transform;

out vec2 uv;

void main() {
    uv = tex;

    vec2 p = mat * (transform + pos);
    gl_Position = vec4(p, 0.0, 1.0);
}
