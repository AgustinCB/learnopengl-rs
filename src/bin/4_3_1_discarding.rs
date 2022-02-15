use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
use learnopengl::game::Game;
use learnopengl::light::DirectionalLight;
use learnopengl::plane::build_plane;

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Discarding fragments",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "12.1-modelloading.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
    )?;
    let directional_light = DirectionalLight::new(
        UnitVector3::new_normalize(Vector3::new(-0.2f32, -1f32, -0.3f32)),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
    );
    let light_cube = cube_mesh(vec![]);
    game.spawn_light(directional_light, &light_cube)?;
    let floor = build_plane(-0.5f32, 5f32, 2f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/metal.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    game.spawn_mesh(&floor, Transform::identity())?;
    game.spawn_mesh(&cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/marble.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]), Transform {
        position: Vector3::new(-1f32, 0f32, -1f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/marble.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]), Transform {
        position: Vector3::new(2f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    let grass = build_plane(-0.5f32, 0.5f32, 1f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/grass.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    for position in vec![
        Vector3::new(-1f32, 0f32, 0.48f32),
        Vector3::new(1.5f32, 0f32, 1.5f32),
        Vector3::new(0f32, 0f32, 0.48f32),
        Vector3::new(-0.8f32, 0f32, 0.48f32),
        Vector3::new(0.8f32, 0f32, 0.48f32),
    ] {
        game.spawn_mesh(&grass, Transform {
            position,
            rotation: Rotation3::from_euler_angles(std::f32::consts::PI/2f32, 0f32, 0f32),
            scale: Vector3::new(1f32, 1f32, 1f32),
        })?;
    }
    game.play_with_fps_camera(vec![])?;
    Ok(())
}