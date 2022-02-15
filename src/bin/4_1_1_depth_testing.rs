use hecs::World;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Input, TextureInfo, Transform};
use learnopengl::ecs::systems::input::InputType;
use learnopengl::ecs::systems::system::System;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::DirectionalLight;
use learnopengl::plane::build_plane;

struct DepthControl;

struct ControlDepth {
    less_key: Keycode,
    always_key: Keycode,
}

impl System for ControlDepth {
    fn name(&self) -> &str {
        "control depth"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (input, _)) in world.query_mut::<(&Input, &DepthControl)>() {
            for event in input.events.iter() {
                if let Event::KeyDown {
                    keycode: Some(k), ..
                } = event {
                    if k == &self.less_key {
                        gl_function!(DepthFunc(gl::LESS));
                    } else if k == &self.always_key {
                        gl_function!(DepthFunc(gl::ALWAYS));
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Depth testing",
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
    let cube = cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/marble.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    let floor = build_plane(-0.5f32, 5f32, 2f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/metal.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(-1f32, 0f32, -1f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(2f32, 0f32, 0f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    game.spawn_mesh(&floor, Transform::identity())?;
    game.spawn((Input::new(vec![InputType::Keyboard]), DepthControl{}));
    game.play_with_fps_camera(vec![
        Box::new(ControlDepth {
            less_key: Keycode::X,
            always_key: Keycode::C,
        })
    ])?;
    Ok(())
}