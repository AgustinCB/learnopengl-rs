use std::cell::RefCell;
use std::rc::Rc;
use hecs::World;
use sdl2::event::Event;
use crate::ecs::components::{Input, QuitControl};
use crate::ecs::systems::system::System;

pub struct QuitSystem {
    pub game_ended: Rc<RefCell<bool>>
}

impl System for QuitSystem {
    fn name(&self) -> &str {
        "Quit"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (input, quit_control)) in world.query_mut::<(&Input, &QuitControl)>() {
            for event in input.events.iter() {
                match event {
                    Event::Quit { .. } => {
                        *(*self.game_ended).borrow_mut() = true;
                    },
                    Event::KeyDown {
                        keycode: Some(k),
                        ..
                    } if *k == quit_control.quit_keycode => {
                        *(*self.game_ended).borrow_mut() = true;
                    },
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

