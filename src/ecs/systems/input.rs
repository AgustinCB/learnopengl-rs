use std::cell::RefCell;
use std::collections::HashMap;
use hecs::World;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use crate::ecs::components::Input;
use crate::ecs::systems::system::System;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum InputType {
    Keyboard,
    Mouse,
    Other,
    Quit,
}

impl From<&Event> for InputType {
    fn from(e: &Event) -> Self {
        match &e {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape, ..), .. } =>
                InputType::Quit,
            e if e.is_keyboard() => InputType::Keyboard,
            e if e.is_mouse() => InputType::Mouse,
            _ => InputType::Other,
        }
    }
}

pub struct InputSystem {
    pub event_pumper: RefCell<EventPump>,
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "Input"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        let mut events_by_type = HashMap::new();
        for event in self.event_pumper.borrow_mut().poll_iter() {
            let event_type = InputType::from(&event);
            if !events_by_type.contains_key(&event_type) {
                events_by_type.insert(event_type.clone(), vec![]);
            }
            events_by_type.get_mut(&event_type).unwrap().push(event);
        }
        for (_e, input) in world.query_mut::<&mut Input>() {
            let mut new_events = vec![];
            for input_type in input.input_types.iter() {
                if let Some(events) = events_by_type.get(input_type) {
                    new_events.extend(events.clone());
                }
            }
            input.events = new_events;
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