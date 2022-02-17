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


vec3 calculatePointLight(
    PointLight light, Material material, vec3 normal, vec3 fragPos, vec3 viewDir, vec2 texCoords
) {
    if (!light.set) return vec3(0.0);
    vec3 lightDir = normalize(light.position - fragPos);

    float diff = max(dot(normal, lightDir), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    float distance    = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance +
                        light.quadratic * (distance * distance));

    vec3 ambient = vec3(0.0);
    vec3 diffuse = vec3(0.0);
    if (material.n_diffuse > 0) {
        ambient += light.ambient * vec3(texture(material.diffuse0, texCoords));
        diffuse += light.diffuse * diff * vec3(texture(material.diffuse0, texCoords));

        ambient *= attenuation;
        diffuse *= attenuation;
    }

    vec3 specular = vec3(0.0);
    if (material.n_specular > 0) {
        specular += light.specular * spec * vec3(texture(material.specular0, texCoords));
        specular *= attenuation;
    }

    return (ambient + diffuse + specular);
}