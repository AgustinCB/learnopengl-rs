use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Skybox, TextureInfo, Transform};
use learnopengl::game::Game;
use learnopengl::light::DirectionalLight;

pub fn main() -> Result<(), String> {
    let mut game = Game::new_with_anti_alias(
        "Anti Aliasing",
        800,
        600,
        120,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "12.1-modelloading.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
        16,
    )?;
    let directional_light = DirectionalLight::new(
        UnitVector3::new_normalize(Vector3::new(-0.2f32, -1f32, -0.3f32)),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
    );
    let skybox = game.spawn_skybox(&Skybox {
        texture_info: TextureInfo {
            id: 0,
            texture_type: TextureType::None,
            path: format!("{}/resource/skybox", env!("CARGO_MANIFEST_DIR")),
        }
    })?;
    game.add_to(skybox, directional_light)?;
    game.spawn_mesh(&cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/container.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]), Transform {
        position: Vector3::new(0f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    game.play_with_fps_camera(vec![])?;
    Ok(())
}