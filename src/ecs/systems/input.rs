use std::cell::RefCell;
use std::collections::HashMap;
use hecs::World;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use crate::ecs::components::Input;
use crate::ecs::systems::system::System;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    pub pressed_down: RefCell<HashMap<Keycode, Event>>,
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "Input"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        self.pressed_down.borrow_mut().drain();
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
        if let Some(keyboard_events) = events_by_type.remove(&InputType::Keyboard) {
            for e in keyboard_events {
                match &e {
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        self.pressed_down.borrow_mut().insert(*keycode, e);
                    },
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        self.pressed_down.borrow_mut().remove(keycode);
                    },
                    _ => {},
                }
            }
        }
        events_by_type.insert(
            InputType::Keyboard,
            self.pressed_down.borrow().values().cloned().collect(),
        );
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