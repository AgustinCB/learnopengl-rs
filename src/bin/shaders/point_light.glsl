struct PointLight {
    vec3 position;

    float constant;
    float linear;
    float quadratic;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    bool set;
};

float shadowCalculation(vec4 fragPosLightSpace, sampler2D shadowMap, vec3 normal, vec3 lightDir) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    float closesDepth = texture(shadowMap, projCoords.xy).r;
    float currentDepth = projCoords.z;
    if (currentDepth > 1.0) return 0.0;
    float bias = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    for (int x = -1; x <= 1; x += 1) {
        for (int y = -1; y <= 1; y += 1) {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow /= 9.0;
    return shadow;
}

vec3 calculatePointLight(
    PointLight light, Material material, vec3 normal, vec3 fragPos, vec3 viewDir, vec2 texCoords
) {
    if (!light.set) return vec3(0.0);
    vec3 lightDir = normalize(light.position - fragPos);

    float diff = max(dot(lightDir, normal), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    float distance    = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance +
                        light.quadratic * (distance * distance));

    vec3 ambient = vec3(0.0);
    vec3 diffuse = vec3(0.0);
    ambient += light.ambient * vec3(texture(material.diffuse0, texCoords));
    diffuse += light.diffuse * diff * vec3(texture(material.diffuse0, texCoords));

    ambient *= attenuation;
    diffuse *= attenuation;

    vec3 specular = vec3(0.0);
    specular += light.specular * spec * vec3(texture(material.specular0, texCoords));
    specular *= attenuation;

    return (ambient + diffuse + specular);
}

vec3 calculatePointLightWithShadow(
    PointLight light, sampler2D shadowMap, vec4 fragPosLightSpace, Material material, vec3 normal, vec3 fragPos, vec3 viewDir, vec2 texCoords
) {
    if (!light.set) return vec3(0.0);
    vec3 lightDir = normalize(light.position - fragPos);

    float diff = max(dot(lightDir, normal), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    float distance    = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance +
    light.quadratic * (distance * distance));

    vec3 ambient = vec3(0.0);
    vec3 diffuse = vec3(0.0);
    ambient += light.ambient * vec3(texture(material.diffuse0, texCoords));
    diffuse += light.diffuse * diff * vec3(texture(material.diffuse0, texCoords));

    ambient *= attenuation;
    diffuse *= attenuation;

    vec3 specular = vec3(0.0);
    specular += light.specular * spec * vec3(texture(material.specular0, texCoords));
    specular *= attenuation;

    float shadow = shadowCalculation(fragPosLightSpace, shadowMap, normal, lightDir);
    return (ambient + (1.0 - shadow) * (diffuse + specular));
}
