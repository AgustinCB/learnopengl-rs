struct SpotLight {
    vec3 direction;
    vec3 position;
    float cutOff;
    float outerCutOff;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;
    bool set;
};

vec3 calculateSpotLightWithPositionAndDirection(
    SpotLight light, vec3 position, vec3 direction, Material material, vec3 normal, vec3 fragPos, vec3 viewDir, vec2 texCoords
) {
    if (!light.set) return vec3(0.0);
    vec3 lightDir = normalize(position - fragPos);
    float theta = dot(lightDir, normalize(-direction));

    float diff = max(dot(lightDir, normal), 0.0);

    float epsilon = light.cutOff - light.outerCutOff;
    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    float distance    = length(position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance +
    light.quadratic * (distance * distance));

    vec3 ambient = vec3(0.0);
    vec3 diffuse = vec3(0.0);
    if (material.n_diffuse > 0) {
        ambient += light.ambient * vec3(texture(material.diffuse0, texCoords));
        diffuse += light.diffuse * diff * vec3(texture(material.diffuse0, texCoords));

        ambient *= attenuation;
        diffuse *= attenuation * intensity;
    }

    vec3 specular = vec3(0.0);
    if (material.n_specular > 0) {
        specular += light.specular * spec * vec3(texture(material.specular0, texCoords));
        specular *= attenuation * intensity;
    }

    return ambient + diffuse + specular;
}

vec3 calculateSpotLight(
    SpotLight light, Material material, vec3 normal, vec3 fragPos, vec3 viewDir, vec2 texCoords
) {
    return calculateSpotLightWithPositionAndDirection(light, light.position, light.direction, material, normal, fragPos, viewDir, texCoords);
}
