#version 410 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
    mat3 NormalMatrix;
} gs_in[];

out GS_OUT {
    vec2 TexCoords;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
    mat3 TBN;
    vec3 Normal;
} gs_out;

uniform vec3 viewPos;

void main() {
    vec3 edge1 = gs_in[1].FragPos - gs_in[0].FragPos;
    vec3 edge2 = gs_in[2].FragPos - gs_in[0].FragPos;
    vec2 delta_uv1 = gs_in[1].TexCoords - gs_in[0].TexCoords;
    vec2 delta_uv2 = gs_in[2].TexCoords - gs_in[0].TexCoords;
    float f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
    vec3 tangent = f * vec3(
        delta_uv2.y * edge1.x - delta_uv1.y * edge2.x,
        delta_uv2.y * edge1.y - delta_uv1.y * edge2.y,
        delta_uv2.y * edge1.z - delta_uv1.y * edge2.z
    );
    for (int i = 0; i < 3; i += 1) {
        vec3 T = normalize(gs_in[i].NormalMatrix * tangent);
        vec3 N = normalize(gs_in[i].Normal);
        T = normalize(T - dot(T, N) * N);
        vec3 B = cross(N, T);
        mat3 TBN = transpose(mat3(T, B, N));
        gs_out.TexCoords = gs_in[i].TexCoords;
        gs_out.TBN = TBN;
        gs_out.TangentViewPos = TBN * viewPos;
        gs_out.TangentFragPos = TBN * gs_in[i].FragPos;
        gs_out.Normal = gs_in[i].Normal;
        gl_Position = gl_in[i].gl_Position;
        EmitVertex();
    }
    EndPrimitive();
}