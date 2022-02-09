use std::cell::RefCell;
use std::rc::Rc;
use learnopengl::camera::Camera;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{FpsCamera, Input, QuitControl, TextureInfo, Transform};
use learnopengl::ecs::systems::rendering::RenderingSystem;
use learnopengl::ecs::world::World;
use learnopengl::light::{DirectionalLight, FlashLight, PointLight, SpotLight};
use learnopengl::window::Window;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::keyboard::Keycode;
use learnopengl::ecs::systems::flashlight::FlashLightSystem;
use learnopengl::ecs::systems::fps_camera::FpsCameraSystem;
use learnopengl::ecs::systems::input::{InputSystem, InputType};
use learnopengl::ecs::systems::quit_system::QuitSystem;

pub fn main() -> Result<(), String> {
    env_logger::init();
    let mut window = Window::new("biochemical lab", 800, 600).unwrap();
    let mut world = World::new();
    let camera = Rc::new(RefCell::new(Camera::new(
        Vector3::new(0.0f32, 0f32, 3f32),
        Vector3::new(0f32, 0f32, -1f32),
        Vector3::y_axis(),
    )));
    let mut rendering = RenderingSystem::new(
        camera.clone(),
        "09.1-lightingmapsvertex.glsl",
        "09.1-lightfragment.glsl",
        "09.1-lightingmapsvertex.glsl",
       "12.1-modelloading.glsl",
    )?;
    let light_cube = cube_mesh(vec![]);
    let cube = cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            width: 512,
            height: 512,
            path: format!("{}/resource/container2.raw", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 1,
            texture_type: TextureType::Specular,
            width: 512,
            height: 512,
            path: format!("{}/resource/container2_specular.png.raw", env!("CARGO_MANIFEST_DIR")),
        },
    ]);

    let cube_positions: [Vector3<f32>; 10] = [
        Vector3::new(0.0f32, 0.0f32, 0.0f32),
        Vector3::new(2.0f32, 5.0f32, -15.0f32),
        Vector3::new(-1.5f32, -2.2f32, -2.5f32),
        Vector3::new(-3.8f32, -2.0f32, -12.3f32),
        Vector3::new(2.4f32, -0.4f32, -3.5f32),
        Vector3::new(-1.7f32, 3.0f32, -7.5f32),
        Vector3::new(1.3f32, -2.0f32, -2.5f32),
        Vector3::new(1.5f32, 2.0f32, -2.5f32),
        Vector3::new(1.5f32, 0.2f32, -1.5f32),
        Vector3::new(-1.3f32, 1.0f32, -1.5f32),
    ];
    for position in cube_positions {
        let shader = rendering.shader_for_mesh(&cube)?;
        world.get_mut().spawn((
            cube.clone(),
            shader,
            Transform {
                position,
                rotation: Rotation3::identity(),
                scale: Vector3::new(1f32, 1f32, 1f32),
            }
        ));
    }
    let directional_light = DirectionalLight::new(
        UnitVector3::new_normalize(Vector3::new(-0.2f32, -1f32, -0.3f32)),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
    );
    world.get_mut().spawn((directional_light, light_cube.clone(), rendering.shader_for_mesh(&light_cube)?));
    let spot_light = SpotLight::new(
        UnitVector3::new_normalize(camera.borrow().front()),
        camera.borrow().position(),
        12.5f32.to_radians().cos(),
        17.5f32.to_radians().cos(),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
        1f32,
        0.09f32,
        0.032f32,
    );
    world.get_mut().spawn((spot_light, light_cube.clone(), FlashLight {
        offset_from_camera: Vector3::repeat(0.1f32),
    }));
    let point_light_positions = [
        Vector3::new(2.7f32,  0.2f32,  2.0f32),
        Vector3::new( 2.3f32, -3.3f32, -4.0f32),
        Vector3::new(-4.0f32,  2.0f32, -12.0f32),
        Vector3::new( 4.0f32,  0.0f32, -3.0f32)
    ];
    let point_lights = point_light_positions.into_iter().map(|p| {
        PointLight::new(
            p,
            Vector3::new(0.36 * 0.2f32, 0.79f32 * 0.2f32, 0.19 * 0.2f32),
            Vector3::new(0.36 * 0.2f32, 0.79f32 * 0.5f32, 0.19 * 0.5f32),
            Vector3::new(0.36f32, 0.79f32, 0.19f32),
            1f32,
            0.09f32,
            0.032f32,
        )
    }).collect::<Vec<PointLight>>();
    for point_light in point_lights {
        world.get_mut().spawn((point_light, light_cube.clone(), rendering.shader_for_mesh(&light_cube)?));
    }
    world.get_mut().spawn((Input::new(vec![InputType::Quit, InputType::Keyboard]), QuitControl {
        quit_keycode: Keycode::Escape,
    }));
    world.get_mut().spawn((Input::new(vec![InputType::Keyboard, InputType::Mouse]), FpsCamera {
        camera_speed: 0.1f32,
    }));
    let game_ended = Rc::new(RefCell::new(false));
    world.add_system(Box::new(rendering));
    world.add_system(Box::new(InputSystem { event_pumper: RefCell::new(window.get_pumper()) }));
    world.add_system(Box::new(QuitSystem { game_ended: game_ended.clone() }));
    world.add_system(Box::new(FlashLightSystem { camera: camera.clone() }));
    world.add_system(Box::new(FpsCameraSystem { camera: camera.clone() }));

    window.start_timer();
    world.start();
    while !*game_ended.borrow() {
        let delta_time = window.delta_time();

        world.early_update(delta_time);
        world.update(delta_time);
        world.late_update(delta_time);

        window.swap_buffers();

        window.delay(1000/60);
    }

    Ok(())
}
