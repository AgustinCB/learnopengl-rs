#version 410 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoords;

out vec2 TexCoords;

uniform mat4 rotation;

void main() {
    TexCoords = aTexCoords;
    gl_Position = rotation * vec4(aPos.x, aPos.y, 0.0, 1.0);
}