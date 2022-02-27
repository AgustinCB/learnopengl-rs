#version 410 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D texture1;

void main() {
    float grey = texture(texture1, TexCoords).r;
    FragColor = vec4(vec3(grey, grey, grey), 1.0);
}
