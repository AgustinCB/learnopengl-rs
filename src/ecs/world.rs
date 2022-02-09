use crate::ecs::systems::system::System;
use hecs::{World as HecsWorld};
use log::error;

pub(crate) fn handle_result<T: Default, E: ToString>(result: Result<T, E>) -> T {
    match result {
        Ok(v) => v,
        Err(s) => {
            error!("{}", &s.to_string());
            T::default()
        }
    }
}

pub struct World {
    systems: Vec<Box<dyn System>>,
    world: HecsWorld,
}

impl World {
    pub fn new() -> World {
        World {
            systems: vec![],
            world: HecsWorld::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn get_mut(&mut self) -> &mut HecsWorld {
        &mut self.world
    }

    pub fn start(&mut self) {
        for system in self.systems.iter() {
            handle_result(system.start(&mut self.world)
                .map_err(|s| {
                    format!("There was an error on {}: {}", system.name(), &s)
                })
            );
        }
    }

    pub fn early_update(&mut self, delta_time: f32) {
        for system in self.systems.iter() {
            handle_result(system.early_update(&mut self.world, delta_time)
                .map_err(|s| {
                    format!("There was an error on {}: {}", system.name(), &s)
                })
            );
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for system in self.systems.iter() {
            handle_result(system.update(&mut self.world, delta_time)
                .map_err(|s| {
                    format!("There was an error on {}: {}", system.name(), &s)
                })
            );
        }
    }

    pub fn late_update(&mut self, delta_time: f32) {
        for system in self.systems.iter() {
            handle_result(system.late_update(&mut self.world, delta_time)
                .map_err(|s| {
                    format!("There was an error on {}: {}", system.name(), &s)
                })
            );
        }
    }
}