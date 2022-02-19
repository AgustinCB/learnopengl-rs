#version 410 core

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
uniform samplerCube shadowMap;
uniform float far_plane;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

out vec4 FragColor;

void main()
{
    float alpha = texture(material.diffuse0, TexCoords).a;
    if (alpha < 0.1) {
        discard;
    }
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);

    vec3 result = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculatePointLightWithShadowInWorldSpace(point_lights[i], shadowMap, far_plane, material, norm, FragPos, viewPos, viewDir, TexCoords);
    }

    FragColor = vec4(result, alpha);
}
