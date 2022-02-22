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

out vec4 FragColor;

in GS_OUT {
    vec2 TexCoords;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
    mat3 TBN;
    vec3 Normal;
} fs_in;

void main() {
    float alpha = texture(material.diffuse0, fs_in.TexCoords).a;
    if (alpha < 0.1) {
        discard;
    }
    vec3 viewDir = normalize(fs_in.TangentViewPos - fs_in.TangentFragPos);
    vec3 norm = texture(material.normal0, fs_in.TexCoords).rgb;
    norm = normalize(norm * 2.0 - 1.0);

    vec3 result = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculateDirectionalLightWithLightDirection(
            directional_lights[i], normalize(fs_in.TBN * directional_lights[i].direction), material, norm, viewDir, fs_in.TexCoords
        );
    }
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculatePointLightWithPosition(
            point_lights[i], fs_in.TBN * point_lights[i].position, material, norm, fs_in.TangentFragPos, viewDir, fs_in.TexCoords
        );
    }
    for (int i = 0; i < MAX_LIGHTS; i++) {
        vec3 lightPos = fs_in.TBN * spot_lights[i].position;
        result += calculateSpotLightWithPositionAndDirection(
            spot_lights[i], lightPos, normalize(lightPos - fs_in.TangentFragPos), material, norm, fs_in.TangentFragPos, viewDir, fs_in.TexCoords
        );
    }

    FragColor = vec4(result, alpha);
}