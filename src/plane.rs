use nalgebra::{Vector2, Vector3};
use crate::ecs::components::{Mesh, TextureInfo};

const VERTICES: [f32; 18] = [
    1.0f32, 1.0f32, 1.0f32,
    -1.0f32, 1.0f32, 1.0f32,
    -1.0f32, 1.0f32, -1.0f32,
    1.0f32, 1.0f32, 1.0f32,
    -1.0f32, 1.0f32, -1.0f32,
    1.0f32, 1.0f32, -1.0f32,
];

const NORMALS: [Vector3<f32>; 6] = [
    Vector3::new(0f32, 1f32, 0f32); 6
];

const TEXTURE_COORDINATES: [Vector2<f32>; 6] = [
    Vector2::new(1f32, 0f32),
    Vector2::new(0f32, 0f32),
    Vector2::new(0f32, 1f32),
    Vector2::new(1f32, 0f32),
    Vector2::new(0f32, 1f32),
    Vector2::new(1f32, 1f32),
];

pub fn build_plane(y_position: f32, scale: f32, texture_scale: f32, textures: Vec<TextureInfo>) -> Mesh {
    let mut vertices = vec![];
    for i in 0..VERTICES.len() / 3 {
        vertices.push(
            Vector3::new(
                VERTICES[i * 3] * scale,
                y_position,
                VERTICES[i * 3 + 2] * scale,
            )
        );
    }
    let textures = if textures.len() == 0 {
        None
    } else {
        Some(textures)
    };
    let texture_coordinates = if textures.is_none() {
        None
    } else {
        Some(TEXTURE_COORDINATES.to_vec().into_iter().map(|v| v * texture_scale).collect())
    };
    Mesh {
        textures,
        texture_coordinates,
        vertices,
        normals: Some(NORMALS.to_vec()),
        tangents: None,
        bitangents: None,
        indices: None,
        shininess: None,
    }
}