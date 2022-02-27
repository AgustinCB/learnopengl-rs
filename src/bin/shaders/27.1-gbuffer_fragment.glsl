#version 410
layout (location = 0) out vec3 gPosition;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec4 gAlbedo;

#include "material.glsl"

in vec2 TexCoords;
in vec3 FragPos;
in vec3 Normal;

uniform Material material;

void main() {
    gPosition = FragPos;
    gNormal = normalize(Normal);
    // gAlbedo.rgb = texture(material.diffuse0, TexCoords).rgb;
    gAlbedo.rgb = vec3(0.95);
}