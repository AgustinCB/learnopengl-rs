struct DirectionalLight {
    vec3 direction;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    bool set;
};

vec3 calculateDirectionalLightWithLightDirection(
    DirectionalLight light, vec3 direction, Material material, vec3 normal, vec3 viewDir, vec2 texCoords
) {
    if (!light.set) return vec3(0.0);
    vec3 lightDir = normalize(-direction);

    float diff = max(dot(lightDir, normal), 0.0);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    vec3 ambient = vec3(0.0);
    vec3 diffuse = vec3(0.0);
    if (material.n_diffuse > 0) {
        ambient += light.ambient * vec3(texture(material.diffuse0, texCoords));
        diffuse += light.diffuse * diff * vec3(texture(material.diffuse0, texCoords));
    }

    vec3 specular = vec3(0.0);
    if (material.n_specular > 0) {
        specular += light.specular * spec * vec3(texture(material.specular0, texCoords));
    }

    return (ambient + diffuse + specular);
}

vec3 calculateDirectionalLight(
    DirectionalLight light, Material material, vec3 normal, vec3 viewDir, vec2 texCoords
) {
    return calculateDirectionalLightWithLightDirection(light, light.direction, material, normal, viewDir, texCoords);
}
