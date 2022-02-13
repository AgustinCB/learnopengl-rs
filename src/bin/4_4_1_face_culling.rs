use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::DirectionalLight;

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Face culling",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "09.1-lightingmapsvertex.glsl",
        "12.1-modelloading.glsl",
        "09.1-lightingmapsvertex.glsl",
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
    gl_function!(Enable(gl::CULL_FACE));
    gl_function!(CullFace(gl::FRONT));
    gl_function!(FrontFace(gl::CCW));
    game.play_with_fps_camera(vec![])?;
    Ok(())
}