#version 410 core
out vec4 FragColor;

in vec2 TexCoords;

#include "material.glsl"
#include "point_light.glsl"

const int MAX_LIGHTS = 32;
uniform PointLight point_lights[MAX_LIGHTS];
uniform vec3 viewPos;
uniform sampler2D gPosition;
uniform sampler2D gNormal;
uniform sampler2D gAlbedoSpec;

void main() {
    vec3 FragPos = texture(gPosition, TexCoords).rgb;
    vec3 normal = texture(gNormal, TexCoords).rgb;
    vec3 diffuse = texture(gAlbedoSpec, TexCoords).rgb;
    float spec = texture(gAlbedoSpec, TexCoords).a;
    vec3 specular = vec3(spec, spec, spec);
    vec3 norm = normalize(normal);
    vec3 viewDir = normalize(viewPos - FragPos);

    vec3 result = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++) {
        result += calculatePointLightWithPositionWithoutMaterial(point_lights[i], point_lights[i].position, 32.0, diffuse, specular, norm, FragPos, viewDir, TexCoords);
    }

    FragColor = vec4(result, 1.0);
}
