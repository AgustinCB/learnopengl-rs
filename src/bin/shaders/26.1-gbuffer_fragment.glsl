#version 410
layout (location = 0) out vec3 gPosition;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec4 gAlbedoSpec;

#include "material.glsl"

in vec2 TexCoords;
in vec3 FragPos;
in vec3 Normal;

uniform Material material;

void main() {
    gPosition = FragPos;
    gNormal = normalize(Normal);
    gAlbedoSpec.rgb = texture(material.diffuse0, TexCoords).rgb;
    if (material.n_specular > 0) {
        gAlbedoSpec.a = texture(material.specular0, TexCoords).r;
    } else {
        gAlbedoSpec.a = 0.0;
    }
}