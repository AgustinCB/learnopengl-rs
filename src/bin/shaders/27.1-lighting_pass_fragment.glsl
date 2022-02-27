#version 410 core
out vec4 FragColor;

in vec2 TexCoords;

#include "material.glsl"
#include "point_light.glsl"

const int MAX_LIGHTS = 32;
uniform PointLight point_lights[MAX_LIGHTS];
uniform sampler2D gPosition;
uniform sampler2D gNormal;
uniform sampler2D gAlbedo;
uniform sampler2D ssao;

void main() {
    vec3 FragPos = texture(gPosition, TexCoords).rgb;
    vec3 normal = texture(gNormal, TexCoords).rgb;
    vec3 diffuse = texture(gAlbedo, TexCoords).rgb;
    float occlusion = texture(ssao, TexCoords).r;
    vec3 specular = vec3(1.0);
    vec3 norm = normalize(normal);
    vec3 viewDir = normalize(-FragPos);

    vec3 result = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculatePointLightWithPositionWithoutMaterialWithOcculsion(point_lights[i], occlusion, point_lights[i].position, 32.0, diffuse, specular, norm, FragPos, viewDir, TexCoords);
    }

    FragColor = vec4(result, 1.0);
}
