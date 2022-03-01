struct Material {
    sampler2D diffuse0;
    sampler2D specular0;
    sampler2D normal0;
    sampler2D height0;
    sampler2D metalness0;
    sampler2D roughness0;
    sampler2D ao0;
    float shininess;
    int n_diffuse;
    int n_specular;
    int n_height;
};