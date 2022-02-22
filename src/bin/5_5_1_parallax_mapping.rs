use std::cell::RefCell;
use hecs::World;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
use learnopengl::ecs::systems::system::System;
use learnopengl::game::Game;
use learnopengl::light::PointLight;
use learnopengl::plane::build_plane;

struct RotationSystem {
    total_ticks: RefCell<f32>,
}

impl System for RotationSystem {
    fn name(&self) -> &str {
        "Rotation System"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn update(&self, world: &mut World, delta_time: f32) -> Result<(), String> {
        let total_ticks = *self.total_ticks.borrow() + delta_time;
        self.total_ticks.replace(total_ticks);
        for (_e, transform) in world.query_mut::<&mut Transform>().without::<PointLight>() {
            transform.rotation = Rotation3::from_axis_angle(
                &UnitVector3::new_normalize(Vector3::new(1f32, 0f32, 1f32)),
                (total_ticks * -0.001f32).to_radians(),
            );
        }
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Parallax mapping",
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
    let point_light = PointLight::new(
        Vector3::new(1f32, 0.5f32, 1f32),
        Vector3::new(0.1f32, 0.1f32, 0.1f32),
        Vector3::new(1f32, 1f32, 1f32),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        1f32,
        0f32,
        0f32,
    );
    game.spawn_light(point_light, &light_cube)?;

    let floor = build_plane(0.5f32, 2f32, 2f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/bricks2.jpg", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 1,
            texture_type: TextureType::Specular,
            path: format!("{}/resource/bricks2.jpg", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 2,
            texture_type: TextureType::Normals,
            path: format!("{}/resource/bricks2_normal.jpg", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 3,
            texture_type: TextureType::Height,
            path: format!("{}/resource/bricks2_disp.jpg", env!("CARGO_MANIFEST_DIR")),
        },
    ]);
    game.spawn_mesh(&floor, Transform {
        position: Vector3::new(1f32, 1f32, -2f32),
        rotation: Rotation3::from_euler_angles(std::f32::consts::PI/2f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    let toybox = build_plane(0.5f32, 2f32, 2f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/toy_box_diffuse.png", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 1,
            texture_type: TextureType::Specular,
            path: format!("{}/resource/toy_box_diffuse.png", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 2,
            texture_type: TextureType::Normals,
            path: format!("{}/resource/toy_box_normal.png", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 3,
            texture_type: TextureType::Height,
            path: format!("{}/resource/toy_box_disp.png", env!("CARGO_MANIFEST_DIR")),
        },
    ]);
    game.spawn_mesh(&toybox, Transform {
        position: Vector3::new(5f32, 1f32, -2f32),
        rotation: Rotation3::from_euler_angles(std::f32::consts::PI/2f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    game.play_with_fps_camera(vec![
        Box::new(RotationSystem {
            total_ticks: RefCell::new(0f32),
        })
    ])?;
    Ok(())
}
