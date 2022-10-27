layout (location = 0) in vec2 pos;

uniform mat2 mat;

uniform vec2 transform;

void main() {
    vec2 p = mat * (transform + pos);
    gl_Position = vec4(p, 0.0, 1.0);
}
