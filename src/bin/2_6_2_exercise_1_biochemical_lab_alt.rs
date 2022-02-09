use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{FpsCamera, Input, QuitControl, TextureInfo, Transform};
use learnopengl::light::{DirectionalLight, PointLight, SpotLight};
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::keyboard::Keycode;
use learnopengl::ecs::systems::flashlight::FlashLightSystem;
use learnopengl::ecs::systems::input::InputType;
use learnopengl::game::Game;

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "biochemical lab",
        800,
        600,
        60,
        "09.1-lightingmapsvertex.glsl",
        "12.1-modelloading.glsl",
        "09.1-lightingmapsvertex.glsl",
        "09.1-lightfragment.glsl",
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
        game.spawn_mesh(
            &cube,
            Transform {
                position,
                rotation: Rotation3::identity(),
                scale: Vector3::new(1f32, 1f32, 1f32),
            }
        )?;
    }
    let directional_light = DirectionalLight::new(
        UnitVector3::new_normalize(Vector3::new(-0.2f32, -1f32, -0.3f32)),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
    );
    game.spawn_light(directional_light, &light_cube)?;
    let spot_light = SpotLight::new(
        UnitVector3::new_normalize(game.camera().borrow().front()),
        game.camera().borrow().position(),
        12.5f32.to_radians().cos(),
        17.5f32.to_radians().cos(),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
        1f32,
        0.09f32,
        0.032f32,
    );
    game.spawn_flash_light(spot_light, &light_cube, Vector3::repeat(0.1f32));
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
        game.spawn_light(point_light, &light_cube)?;
    }
    game.spawn((Input::new(vec![InputType::Quit, InputType::Keyboard]), QuitControl {
        quit_keycode: Keycode::Escape,
    }));
    game.spawn((Input::new(vec![InputType::Keyboard, InputType::Mouse]), FpsCamera {
        camera_speed: 0.1f32,
    }));
    game.play_with_fps_camera(vec![
        Box::new(FlashLightSystem { camera: game.camera().clone() })
    ])
}
