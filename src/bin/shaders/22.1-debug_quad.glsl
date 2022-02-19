#version 410 core
layout (location = 0) in vec3 aPos;

layout (std140) uniform Matrices {
    mat4 view;
    mat4 projection;
};

out vec3 TexCoords;

void main() {
    TexCoords = aPos;
    gl_Position = (view * projection * vec4(aPos, 1.0)).xyww;
}