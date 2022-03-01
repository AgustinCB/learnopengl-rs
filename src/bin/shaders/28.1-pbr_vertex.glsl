#version 410 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTextCoords;

layout (std140) uniform Matrices {
    mat4 view;
    mat4 projection;
};
uniform mat4 model;

out vec3 WorldPos;
out vec3 Normal;
out vec2 TexCoords;

void main() {
    TexCoords = aTextCoords;
    WorldPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(model) * aNormal;
    
    gl_Position = projection * view * model * vec4(aPos, 1.0);
}