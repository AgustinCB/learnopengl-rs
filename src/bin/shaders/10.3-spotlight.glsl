#version 330 core

#include "material.glsl"

#include "spot_light.glsl"

uniform SpotLight light;
uniform Material material;
uniform vec3 viewPos;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

out vec4 FragColor;

void main() {
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);

    vec3 result = calculateSpotLight(light, material, norm, FragPos, viewDir, TexCoords);
    FragColor = vec4(result, 1.0);
}