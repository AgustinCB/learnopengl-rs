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
uniform float height_scale;

out vec4 FragColor;

in GS_OUT {
    vec2 TexCoords;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
    mat3 TBN;
    vec3 Normal;
} fs_in;

vec2 ParallaxMapping(vec2 texCoords, vec3 viewDir, Material material) {
    /*
    Without steep calculation:

    float height = texture(material.height0, texCoords).r;
    vec2 p = viewDir.xy / viewDir.z * (height * height_scale);
    return texCoords - p;
    */
    const float minLayers = 8.0;
    const float maxLayers = 32.0;
    float numLayers = mix(maxLayers, minLayers, max(dot(vec3(0.0, 0.0, 1.0), viewDir), 0.0));
    float layerDepth = 1.0 / numLayers;
    float currentLayerDepth = 0.0;
    vec2 P = viewDir.xy * height_scale;
    vec2 deltaTexCoords = P / numLayers;

    vec2 currentTexCoords = texCoords;
    float currentDepthMapValue = texture(material.height0, currentTexCoords).r;
    while (currentLayerDepth < currentDepthMapValue) {
        currentTexCoords -= deltaTexCoords;
        currentDepthMapValue = texture(material.height0, currentTexCoords).r;
        currentLayerDepth += layerDepth;
    }
    vec2 prevTexCoords = currentTexCoords + deltaTexCoords;

    float afterDepth = currentDepthMapValue - currentLayerDepth;
    float beforeDepth = texture(material.height0, prevTexCoords).r - currentLayerDepth + layerDepth;

    float weight = afterDepth / (afterDepth - beforeDepth);

    return prevTexCoords * weight + currentTexCoords * (1.0 - weight);
}

void main() {
    float alpha = texture(material.diffuse0, fs_in.TexCoords).a;
    if (alpha < 0.1) {
        discard;
    }
    vec3 viewDir = normalize(fs_in.TangentViewPos - fs_in.TangentFragPos);
    vec2 texCoords = fs_in.TexCoords;
    if (material.n_height > 0) {
        texCoords = ParallaxMapping(texCoords, viewDir, material);
        if (texCoords.x > 1.0 || texCoords.y > 1.0 || texCoords.x < 0.0 || texCoords.y < 0.0)
            discard;
    }
    vec3 norm = texture(material.normal0, texCoords).rgb;
    norm = normalize(norm * 2.0 - 1.0);

    vec3 result = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculateDirectionalLightWithLightDirection(
            directional_lights[i], normalize(fs_in.TBN * directional_lights[i].direction), material, norm, viewDir, texCoords
        );
    }
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculatePointLightWithPosition(
            point_lights[i], fs_in.TBN * point_lights[i].position, material, norm, fs_in.TangentFragPos, viewDir, texCoords
        );
    }
    for (int i = 0; i < MAX_LIGHTS; i++) {
        vec3 lightPos = fs_in.TBN * spot_lights[i].position;
        result += calculateSpotLightWithPositionAndDirection(
            spot_lights[i], lightPos, normalize(lightPos - fs_in.TangentFragPos), material, norm, fs_in.TangentFragPos, viewDir, texCoords
        );
    }

    FragColor = vec4(result, alpha);
}