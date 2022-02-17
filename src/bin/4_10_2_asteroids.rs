use nalgebra::{Rotation3, UnitVector3, Vector3};
use rand::Rng;
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
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
    let model_path = format!("{}/../LOGL/resources/objects/planet/planet.obj", env!("CARGO_MANIFEST_DIR"));
    game.spawn_model_from_file(&model_path, Transform {
        position: Vector3::new(1f32, 0f32, 0f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(0.25f32, 0.25f32, 0.25f32),
    })?;
    let offset = 0.625f32;
    let radius = 2.5f32;
    let amount = 100000;
    let mut rng = rand::thread_rng();
    let offsets = Vec::from_iter(
        (0..amount).into_iter()
            .map(|i| {
                let angle = i as f32 / amount as f32 * 360f32;
                let displacement = (rng.gen_range(0.0f32..1.0) % (2f32 * offset * 100f32)) / 100f32 - offset;
                let x = angle.sin() * radius + displacement;
                let displacement = (rng.gen_range(0.0f32..1.0) % (2f32 * offset * 100f32)) / 100f32 - offset;
                let y = displacement * 0.4f32;
                let displacement = (rng.gen_range(0.0f32..1.0) % (2f32 * offset * 100f32)) / 100f32 - offset;
                let z = angle.cos() * radius + displacement;
                Vector3::new(x, y, z)
            })
    );
    game.spawn_instanced_model_from_file(&model_path, Transform {
        position: Vector3::new(0f32, 0f32, 0f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(0.1f32, 0.1f32, 0.1f32),
    }, offsets)?;
    game.play_with_fps_camera(vec![])?;
    Ok(())
}