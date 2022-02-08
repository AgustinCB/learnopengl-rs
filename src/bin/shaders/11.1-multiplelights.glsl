#version 330 core

#include "simple_material.glsl"
#include "simple_directional_light.glsl"
#include "simple_point_light.glsl"
#include "simple_spot_light.glsl"

#define N_POINT_LIGHTS 4
uniform DirectionalLight dirLight;
uniform PointLight pointLights[N_POINT_LIGHTS];
uniform SpotLight spotLight;
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

    vec3 result = calculateDirectionalLight(dirLight, material, norm, viewDir, TexCoords);
    for (int i = 0; i < N_POINT_LIGHTS; i++) {
        result += calculatePointLight(pointLights[i], material, norm, FragPos, viewDir, TexCoords);
    }
    result += calculateSpotLight(spotLight, material, norm, FragPos, viewDir, TexCoords);

    FragColor = vec4(result, 1.0);
}
