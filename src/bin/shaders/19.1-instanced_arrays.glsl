#version 410 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTextCoords;
layout (location = 3) in vec3 aOffset;

layout (std140) uniform Matrices {
    mat4 view;
    mat4 projection;
};
uniform mat4 model;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

void main() {
    vec4 pos = model * vec4(aPos, 1.0) + vec4(aOffset, 0.0);
    gl_Position = projection * view * pos;
    FragPos = vec3(pos);
    Normal = aNormal;
    TexCoords = aTextCoords;
}