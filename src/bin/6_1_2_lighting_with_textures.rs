#![feature(variant_count)]
#![feature(iter_advance_by)]

use itertools::Itertools;
use nalgebra::{Rotation3, Vector3};
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
use learnopengl::game::Game;
use learnopengl::light::PointLight;
use learnopengl::sphere::sphere_mesh;

const N_ROWS: usize = 7;
const N_COLUMNS: usize = 7;
const SPACING: f32 = 2.5f32;

pub fn main() -> Result<(), String> {
    let mut game = Game::new_with_anti_alias(
        "PBR lighting", 800, 600, 60, Vector3::new(0f32, 0f32, 0f32),
        "28.1-pbr_vertex.glsl", "28.2-pbr.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl", "25.1-bloom_light_fragment.glsl",
        16,
    )?;
    let mut light_cube = cube_mesh(vec![]);
    light_cube.vertices = light_cube.vertices.iter().map(|v| v * 0.25).collect_vec();
    for (position, color) in vec![
        (Vector3::new(-10f32, 10f32, 10f32), Vector3::new(300f32, 300f32, 300f32)),
        (Vector3::new(10f32, 10f32, 10f32), Vector3::new(300f32, 300f32, 300f32)),
        (Vector3::new(-10f32, -10f32, 10f32), Vector3::new(300f32, 300f32, 300f32)),
        (Vector3::new(10f32, -10f32, 10f32), Vector3::new(300f32, 300f32, 300f32)),
    ] {
        let point_light = PointLight::new(
            position,
            Vector3::zeros(),
            Vector3::zeros(),
            color,
            1f32,
            0.7f32,
            1.8f32,
        );
        game.spawn_light(point_light, &light_cube)?;
    }
    let sphere = sphere_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/rusted_iron/albedo.png", env!("CARGO_MANIFEST_DIR"))
        },
        TextureInfo {
            id: 1,
            texture_type: TextureType::Metalness,
            path: format!("{}/resource/rusted_iron/metallic.png", env!("CARGO_MANIFEST_DIR"))
        },
        TextureInfo {
            id: 2,
            texture_type: TextureType::Roughness,
            path: format!("{}/resource/rusted_iron/roughness.png", env!("CARGO_MANIFEST_DIR"))
        },
        TextureInfo {
            id: 3,
            texture_type: TextureType::AmbientOcclusion,
            path: format!("{}/resource/rusted_iron/ao.png", env!("CARGO_MANIFEST_DIR"))
        },
        TextureInfo {
            id: 4,
            texture_type: TextureType::Normals,
            path: format!("{}/resource/rusted_iron/normal.png", env!("CARGO_MANIFEST_DIR"))
        },
    ]);

    for row in 0..N_ROWS {
        for col in 0..N_COLUMNS {
            game.spawn_mesh(&sphere, Transform {
                position: Vector3::new(
                    (col as f32 - (N_COLUMNS as f32 / 2f32)) * SPACING,
                    (row as f32 - (N_ROWS as f32 / 2f32)) * SPACING,
                    0f32,
                ),
                rotation: Rotation3::identity(),
                scale: Vector3::new(1f32, 1f32, 1f32),
            })?;
        }
    }
    game.play_with_fps_camera(vec![])?;
    Ok(())
}
