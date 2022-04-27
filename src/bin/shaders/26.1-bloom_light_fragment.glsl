#version 410 core
layout (location = 0) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

struct Light {
    vec3 diffuse;
};

uniform Light light;

void main() {
    FragColor = vec4(light.diffuse, 1.0);
    float brightness = dot(FragColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    if (brightness > 1.0) {
        BrightColor = vec4(FragColor.rgb, 1.0);
    } else {
        BrightColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}