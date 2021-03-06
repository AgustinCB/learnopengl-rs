use hecs::World;
use nalgebra::Vector3;
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Input, TextureInfo, Transform};
use learnopengl::ecs::systems::input::InputType;
use learnopengl::ecs::systems::system::System;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::PointLight;
use learnopengl::plane::build_plane;

struct GammaCorrectionControl;

enum UpdateLightsAction {
    Nothing,
    IncreaseAttenuation(Vector3<f32>),
    DecreaseAttenuation(Vector3<f32>),
}

struct GammaCorrectionSystem {
    with: Keycode,
    without: Keycode,
}

impl System for GammaCorrectionSystem {
    fn name(&self) -> &str {
        "Gamma Correction"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        let mut action = UpdateLightsAction::Nothing;
        for (_e, (input, _)) in world.query_mut::<(&Input, &GammaCorrectionControl)>() {
            for event in input.events.iter() {
                if let Event::KeyDown {
                    keycode: Some(k), ..
                } = event {
                    if k == &self.with {
                        gl_function!(Enable(gl::FRAMEBUFFER_SRGB));
                        action = UpdateLightsAction::DecreaseAttenuation(Vector3::new(0f32, 0f32, 1f32));
                    } else if k == &self.without {
                        gl_function!(Disable(gl::FRAMEBUFFER_SRGB));
                        action = UpdateLightsAction::IncreaseAttenuation(Vector3::new(0f32, 1f32, 0f32));
                    }
                }
            }
        }

        match action {
            UpdateLightsAction::DecreaseAttenuation(v) | UpdateLightsAction::IncreaseAttenuation(v) => {
                for (_e, point_light) in world.query_mut::<&mut PointLight>() {
                    point_light.constant = v.data.0[0][0];
                    point_light.linear = v.data.0[0][1];
                    point_light.quadratic = v.data.0[0][2];
                }
            }
            _ => {}
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
        "Gamma Correction",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "12.1-modelloading.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
    )?;
    let light_cube = cube_mesh(vec![]);

    let mut color = 0.25f32;
    for position in vec![
        Vector3::new(-3f32, 2f32, 0f32),
        Vector3::new(-1f32,  2f32, 0f32),
        Vector3::new(1f32, 2f32, 0f32),
        Vector3::new(3f32, 2f32, 0f32),
    ] {
        let point_light = PointLight::new(
            position,
            Vector3::new(color * 0.2f32, color * 0.2f32, color * 0.2f32),
            Vector3::new(color * 0.5f32, color * 0.5f32, color * 0.5f32),
            Vector3::new(color, color, color),
            0f32,
            1f32,
            0f32,
        );
        game.spawn_light(point_light, &light_cube)?;
        color *= 2f32;
    }

    let floor = build_plane(-0.5f32, 10f32, 10f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/wood.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    game.spawn_mesh(&floor, Transform::identity())?;
    game.spawn((Input::new(vec![InputType::Keyboard]), GammaCorrectionControl {}));
    game.play_with_fps_camera(vec![
        Box::new(GammaCorrectionSystem {
            with: Keycode::X,
            without: Keycode::C,
        })
    ])?;
    Ok(())
}
