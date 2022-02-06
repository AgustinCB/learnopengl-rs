#version 330 core

struct Light {
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Light light;

out vec4 FragColor;
out vec3 Normal;
out vec2 TexCoords;

void main() {
    FragColor = vec4(light.specular, 1.0);
}