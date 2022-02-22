#version 410 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTextCoords;

layout (std140) uniform Matrices {
    mat4 view;
    mat4 projection;
};
uniform mat4 model;

out VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
    mat3 NormalMatrix;
} vs_out;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vs_out.FragPos = vec3(model * vec4(aPos, 1.0));
    vs_out.Normal = normalMatrix * aNormal;
    vs_out.NormalMatrix = normalMatrix;
    vs_out.TexCoords = aTextCoords;
}