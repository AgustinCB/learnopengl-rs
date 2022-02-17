use nalgebra::{Scale3, Translation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::TextureInfo;
use learnopengl::game::Game;
use learnopengl::light::{DirectionalLight, PointLight};

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Model loading",
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
    game.spawn_light(directional_light.clone(), &light_cube)?;
    let point_light = PointLight::new(
        Vector3::new(0f32,  4f32,  0f32),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
        1f32,
        0.09f32,
        0.032f32,
    );
    game.spawn_light(point_light, &light_cube)?;
    let mut offsets = Vec::with_capacity(100);
    let offset = 0.1f32;
    for i in (-10..10).step_by(2) {
        for j in (-10..10).step_by(2) {
            let translation = Translation3::new(j as f32 / 10f32 + offset, i as f32 / 10f32 + offset, 0f32).to_homogeneous();
            let scale = Scale3::new(0.1f32, 0.1f32, 0.1f32).to_homogeneous();
            offsets.push(translation * scale);
        }
    }
    let cube = cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/container.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    game.spawn_instanced_mesh(&cube, offsets)?;
    game.play_with_fps_camera(vec![])?;
    Ok(())
}