#version 330 core

#include "material.glsl"
#include "directional_light.glsl"
#include "point_light.glsl"
#include "spot_light.glsl"

#define MAX_LIGHTS 4
uniform DirectionalLight directional_lights[MAX_LIGHTS];
uniform PointLight point_lights[MAX_LIGHTS];
uniform SpotLight spot_lights[MAX_LIGHTS];
uniform Material material;
uniform vec3 viewPos;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

out vec4 FragColor;

void main()
{
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);

    vec3 result = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculateDirectionalLight(directional_lights[i], material, norm, viewDir, TexCoords);
    }
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculatePointLight(point_lights[i], material, norm, FragPos, viewDir, TexCoords);
    }
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculateSpotLight(spot_lights[i], material, norm, FragPos, viewDir, TexCoords);
    }

    FragColor = vec4(result, 1.0);
}
