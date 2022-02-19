#version 410 core
layout (location = 0) in vec3 aPos;

uniform mat4 model;
out vec4 FragPos;

void main() {
    gl_Position = model * vec4(aPos, 1.0);
    FragPos = model * vec4(aPos, 1.0);
}