use std::cell::RefCell;
use std::rc::Rc;
use hecs::World;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseUtil;
use crate::camera::Camera;
use crate::ecs::components::{FpsCamera, Input};
use crate::ecs::systems::system::System;

pub struct FpsCameraSystem {
    pub camera: Rc<RefCell<Camera>>,
    pub mouse: MouseUtil,
}

impl System for FpsCameraSystem {
    fn name(&self) -> &str {
        "FPS Camera"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        self.mouse.show_cursor(false);
        Ok(())
    }

    fn early_update(&self, world: &mut World, delta_time: f32) -> Result<(), String> {
        for (_e, (input, fps_camera)) in world.query_mut::<(&Input, &FpsCamera)>() {
            let camera_speed = delta_time * fps_camera.camera_speed;
            for event in input.events.iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::W),
                        ..
                    } => {
                        (*self.camera).borrow_mut().move_forward(camera_speed);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::S),
                        ..
                    } => {
                        (*self.camera).borrow_mut().move_forward(-camera_speed);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::D),
                        ..
                    } => {
                        (*self.camera).borrow_mut().move_right(camera_speed);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::A),
                        ..
                    } => {
                        (*self.camera).borrow_mut().move_right(-camera_speed);
                    }
                    Event::MouseMotion { xrel, yrel, .. } => {
                        let sensitivity = 0.1f32;
                        let xoffset = *xrel as f32 * sensitivity;
                        let yoffset = *yrel as f32 * sensitivity;
                        (*self.camera).borrow_mut().move_front(xoffset, yoffset);
                    }
                    Event::MouseWheel { y, .. } => {
                        (*self.camera).borrow_mut().move_fov(-(*y as f32));
                    }
                    _ => {}
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