use std::cell::RefCell;
use std::rc::Rc;
use hecs::World;
use nalgebra::UnitVector3;
use crate::camera::Camera;
use crate::ecs::systems::system::System;
use crate::light::{FlashLight, SpotLight};

pub struct FlashLightSystem {
    pub camera: Rc<RefCell<Camera>>
}

impl System for FlashLightSystem {
    fn name(&self) -> &str {
        "FlashLight"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (flashlight, spotlight)) in world.query_mut::<(&FlashLight, &mut SpotLight)>() {
            spotlight.set_direction(UnitVector3::new_normalize((*self.camera).borrow().front() - flashlight.offset_from_camera));
            spotlight.set_position((*self.camera).borrow().position() - flashlight.offset_from_camera);
        }
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }
}