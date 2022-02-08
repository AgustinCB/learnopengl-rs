use std::cell::RefCell;
use std::rc::Rc;
use gl;
use learnopengl::camera::Camera;
use learnopengl::gl_function;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
use learnopengl::ecs::systems::rendering::RenderingSystem;
use learnopengl::ecs::world::World;
use learnopengl::light::{DirectionalLight, FlashLight, PointLight, SpotLight};
use learnopengl::window::Window;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::ecs::systems::flashlight::FlashLightSystem;

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
    let mut yaw = -90f32;
    let mut pitch = 0f32;
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
    world.add_system(Box::new(rendering));
    world.add_system(Box::new(FlashLightSystem { camera: camera.clone() }));

    window.start_timer();
    world.start();
    gl_function!(Enable(gl::DEPTH_TEST));
    gl_function!(ClearColor(1f32, 1f32, 1f32, 1.0));
    'gameloop: loop {
        let delta_time = window.delta_time();
        let camera_speed = 0.01f32 * delta_time;
        for event in window.events() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    (*camera).borrow_mut().move_forward(camera_speed);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    (*camera).borrow_mut().move_forward(-camera_speed);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    (*camera).borrow_mut().move_right(camera_speed);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    (*camera).borrow_mut().move_right(-camera_speed);
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    let sensitivity = 0.2f32;
                    let xoffset = xrel as f32 * sensitivity;
                    let yoffset = yrel as f32 * sensitivity;
                    yaw += xoffset;
                    pitch += yoffset;
                    pitch = pitch.clamp(-89f32, 89f32);
                    (*camera).borrow_mut().set_front(yaw, pitch);
                }
                Event::MouseWheel { y, .. } => {
                    (*camera).borrow_mut().move_fov(-(y as f32));
                }
                _ => {}
            }
        }

        gl_function!(Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        world.update();
        world.late_update();

        window.swap_buffers();

        window.delay(1000/60);
    }

    Ok(())
}
