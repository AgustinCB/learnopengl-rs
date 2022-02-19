#version 410 core
out vec4 FragColor;

in vec3 TexCoords;

uniform samplerCube depth_map;
uniform float near_plane;
uniform float far_plane;

float linearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0;
    return (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane));
}

void main() {
    float depth_value = texture(depth_map, TexCoords).r;
    FragColor = vec4(vec3(depth_value), 1.0);
}