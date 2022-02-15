use nalgebra::{Rotation3, UnitVector3, Vector3};
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::Transform;
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
    game.spawn_light(directional_light, &light_cube)?;
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
    let model_path = format!("{}/../LOGL/resources/objects/backpack/backpack.obj", env!("CARGO_MANIFEST_DIR"));
    game.spawn_model_from_file(&model_path, Transform {
        position: Vector3::new(0f32, 0f32, 0f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    game.play_with_fps_camera(vec![])?;
    Ok(())
}