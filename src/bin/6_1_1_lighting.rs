#![feature(variant_count)]
#![feature(iter_advance_by)]

use itertools::Itertools;
use nalgebra::{Rotation3, Vector3};
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{ExtraUniform, Transform, UniformValue};
use learnopengl::game::Game;
use learnopengl::light::PointLight;
use learnopengl::sphere::sphere_mesh;

const N_ROWS: usize = 7;
const N_COLUMNS: usize = 7;
const SPACING: f32 = 2.5f32;

pub fn main() -> Result<(), String> {
    let mut game = Game::new_with_anti_alias("PBR lighting", 800, 600, 60, Vector3::new(0f32, 0f32, 0f32), "28.1-pbr_vertex.glsl", "28.1-pbr.glsl", "17.1-uniform_buffer_objects_vertex.glsl", "25.1-bloom_light_fragment.glsl", 16,)?;
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
    let sphere = sphere_mesh(vec![]);

    for row in 0..N_ROWS {
        let metallic = row as f32 / N_ROWS as f32;
        for col in 0..N_COLUMNS {
            let roughness = (col as f32 / N_COLUMNS as f32).clamp(0.05f32, 1f32);
            let e = game.spawn_mesh(&sphere, Transform {
                position: Vector3::new(
                    (col as f32 - (N_COLUMNS as f32 / 2f32)) * SPACING,
                    (row as f32 - (N_ROWS as f32 / 2f32)) * SPACING,
                    0f32,
                ),
                rotation: Rotation3::identity(),
                scale: Vector3::new(1f32, 1f32, 1f32),
            })?;
            game.add_to(e, vec![
                ExtraUniform {
                    name: "metallic",
                    value: UniformValue::Float(metallic),
                },
                ExtraUniform {
                    name: "roughness",
                    value: UniformValue::Float(roughness),
                },
                ExtraUniform {
                    name: "ao",
                    value: UniformValue::Float(1f32),
                },
                ExtraUniform {
                    name: "albedo",
                    value: UniformValue::Vector3(Vector3::new(0.5f32, 0f32, 0f32)),
                },
            ])?;
        }
    }
    game.play_with_fps_camera(vec![])?;
    Ok(())
}
