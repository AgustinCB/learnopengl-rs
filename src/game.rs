use std::cell::RefCell;
use std::rc::Rc;
use hecs::DynamicBundle;
use nalgebra::Vector3;
use crate::camera::Camera;
use crate::ecs::components::{Mesh, Transform};
use crate::ecs::systems::fps_camera::FpsCameraSystem;
use crate::ecs::systems::input::InputSystem;
use crate::ecs::systems::quit_system::QuitSystem;
use crate::ecs::systems::rendering::RenderingSystem;
use crate::ecs::systems::system::System;
use crate::ecs::world::World;
use crate::light::{FlashLight, Light, SpotLight};
use crate::window::Window;

pub struct Game {
    camera: Rc<RefCell<Camera>>,
    fps: usize,
    game_ended: Rc<RefCell<bool>>,
    rendering_system: Option<RenderingSystem>,
    window: Window,
    world: World,
}

impl Game {
    pub fn new(
        name: &str,
        width: usize,
        height: usize,
        fps: usize,
        model_vertex_shader: &'static str,
        model_fragment_shader: &'static str,
        light_vertex_shader: &'static str,
        light_fragment_shader: &'static str,
    ) -> Result<Game, String> {
        env_logger::init();
        let window = Window::new(name, width, height).unwrap();
        let world = World::new();
        let camera = Rc::new(RefCell::new(Camera::new(
            Vector3::new(0.0f32, 0f32, 3f32),
            Vector3::new(0f32, 0f32, -1f32),
            Vector3::y_axis(),
        )));
        let rendering = RenderingSystem::new(
            camera.clone(),
            light_vertex_shader,
            light_fragment_shader,
            model_vertex_shader,
            model_fragment_shader,
        )?;
        Ok(Game {
            camera,
            fps,
            window,
            world,
            game_ended: Rc::new(RefCell::new(false)),
            rendering_system: Some(rendering),
        })
    }

    pub fn camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    pub fn spawn_mesh(&mut self, mesh: &Mesh, transform: Transform) -> Result<(), String> {
        let shader = {
            let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
            rendering.shader_for_mesh(&mesh)?
        };
        self.world.get_mut().spawn((mesh.clone(), shader, transform));
        Ok(())
    }

    pub fn spawn_light<L: Light + Send + Sync + 'static>(&mut self, light: L, mesh: &Mesh) -> Result<(), String> {
        let shader = {
            let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
            rendering.shader_for_mesh(&mesh)?
        };
        self.world.get_mut().spawn((mesh.clone(), light, shader));
        Ok(())
    }

    pub fn spawn(&mut self, components: impl DynamicBundle) {
        self.world.get_mut().spawn(components);
    }

    pub fn spawn_flash_light(&mut self, light: SpotLight, mesh: &Mesh, offset: Vector3<f32>) {
        self.world.get_mut().spawn((light, mesh.clone(), FlashLight {
            offset_from_camera: offset,
        }));
    }

    pub fn play_with_fps_camera(&mut self, systems: Vec<Box<dyn System>>) -> Result<(), String> {
        let rendering = self.rendering_system.take()
            .ok_or("No rendering system".to_string())?;
        self.world.add_system(Box::new(rendering));
        self.world.add_system(Box::new(InputSystem { event_pumper: RefCell::new(self.window.get_pumper()) }));
        self.world.add_system(Box::new(QuitSystem { game_ended: self.game_ended.clone() }));
        self.world.add_system(Box::new(FpsCameraSystem { camera: self.camera.clone() }));
        for system in systems {
            self.world.add_system(system);
        }

        self.window.start_timer();
        self.world.start();
        while !(*self.game_ended.borrow()) {
            let delta_time = self.window.delta_time();

            self.world.early_update(delta_time);
            self.world.update(delta_time);
            self.world.late_update(delta_time);

            self.window.swap_buffers();

            self.window.delay(1000/self.fps);
        }
        Ok(())
    }
}