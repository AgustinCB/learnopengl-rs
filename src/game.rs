use std::cell::RefCell;
use std::rc::Rc;
use hecs::{Component, DynamicBundle, Entity};
use nalgebra::{Matrix4, Vector3};
use sdl2::keyboard::Keycode;
use crate::camera::Camera;
use crate::ecs::components::{FpsCamera, Input, InstancedMesh, InstancedModel, Mesh, Model, QuitControl, Skybox, Transform};
use crate::ecs::systems::fps_camera::FpsCameraSystem;
use crate::ecs::systems::input::{InputSystem, InputType};
use crate::ecs::systems::quit_system::QuitSystem;
use crate::ecs::systems::rendering::RenderingSystem;
use crate::ecs::systems::system::System;
use crate::ecs::world::World;
use crate::light::{FlashLight, Light, SpotLight};
use crate::loader::{load_instanced_model, load_model};
use crate::window::Window;

pub struct Game {
    camera: Rc<RefCell<Camera>>,
    _fps: usize,
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
        clear_color: Vector3<f32>,
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
            clear_color,
            light_vertex_shader,
            light_fragment_shader,
            model_vertex_shader,
            model_fragment_shader,
        )?;
        Ok(Game {
            camera,
            _fps: fps,
            window,
            world,
            game_ended: Rc::new(RefCell::new(false)),
            rendering_system: Some(rendering),
        })
    }

    pub fn camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    pub fn spawn_model(&mut self, model: Vec<Mesh>, transform: Transform) -> Result<Entity, String> {
        let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
        Ok(self.world.get_mut().spawn((
            Model::from_meshes(model, rendering)?,
            transform
        )))
    }

    pub fn spawn_model_from_file(&mut self, model: &str, transform: Transform) -> Result<Entity, String> {
        let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
        let model = load_model(model, rendering)?;
        Ok(self.world.get_mut().spawn((model, transform)))
    }

    pub fn spawn_instanced_model_from_file(&mut self, model: &str, models: Vec<Matrix4<f32>>) -> Result<Entity, String> {
        let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
        let model = load_instanced_model(model, rendering, models)?;
        let entity = self.world.get_mut().spawn(());
        self.world.get_mut().insert_one(entity, model)
            .map_err(|e| e.to_string())?;
        Ok(entity)
    }

    pub fn spawn_skybox(&mut self, skybox: &Skybox) -> Result<Entity, String> {
        let shader = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?
            .shader_for_skybox(skybox)?;
        Ok(self.world.get_mut().spawn((shader, skybox.clone())))
    }

    pub fn spawn_mesh(&mut self, mesh: &Mesh, transform: Transform) -> Result<Entity, String> {
        let shader = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?
            .shader_for_mesh(&mesh)?;
        Ok(self.world.get_mut().spawn((mesh.clone(), shader, transform)))
    }

    pub fn spawn_instanced_model(&mut self, model: Vec<Mesh>, models: Vec<Matrix4<f32>>) -> Result<Entity, String> {
        let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
        let entity = self.world.get_mut().spawn(());
        self.world.get_mut().insert_one(entity, InstancedModel::new(model, rendering, models))
            .map_err(|e| e.to_string())?;
        Ok(entity)
    }

    pub fn spawn_instanced_mesh(&mut self, mesh: &Mesh, models: Vec<Matrix4<f32>>) -> Result<Entity, String> {
        let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
        let shader = rendering.shader_for_mesh(mesh)?;
        let shader = rendering.instanced_rendering.shader_for_mesh(&shader)?;
        Ok(self.world.get_mut().spawn((InstancedMesh {
            models,
            mesh: mesh.clone()
        }, shader)))
    }

    pub fn spawn_light<L: Light + Send + Sync + 'static>(&mut self, light: L, mesh: &Mesh) -> Result<(), String> {
        let rendering = self.rendering_system.as_mut().ok_or("No Rendering system".to_string())?;
        let shader = rendering.shader_for_mesh(&mesh)?;
        self.world.get_mut().spawn((mesh.clone(), shader, light));
        Ok(())
    }

    pub fn spawn(&mut self, components: impl DynamicBundle) {
        self.world.get_mut().spawn(components);
    }

    pub fn add_to(&mut self, entity: Entity, component: impl Component) -> Result<(), String> {
        self.world.get_mut().insert_one(entity, component).map_err(|e| e.to_string())
    }

    pub fn spawn_flash_light(&mut self, light: SpotLight, mesh: &Mesh, offset: Vector3<f32>) {
        self.world.get_mut().spawn((light, mesh.clone(), FlashLight {
            offset_from_camera: offset,
        }));
    }

    pub fn play_with_fps_camera(&mut self, systems: Vec<Box<dyn System>>) -> Result<(), String> {
        self.spawn((Input::new(vec![InputType::Quit, InputType::Keyboard]), QuitControl {
            quit_keycode: Keycode::Escape,
        }));
        self.spawn((Input::new(vec![InputType::Keyboard, InputType::Mouse]), FpsCamera {
            camera_speed: 0.005f32,
        }));
        let rendering = self.rendering_system.take()
            .ok_or("No rendering system".to_string())?;
        self.world.add_system(Box::new(rendering));
        self.world.add_system(Box::new(InputSystem { event_pumper: RefCell::new(self.window.get_pumper()) }));
        self.world.add_system(Box::new(QuitSystem { game_ended: self.game_ended.clone() }));
        self.world.add_system(Box::new(FpsCameraSystem { camera: self.camera.clone(), mouse: self.window.mouse() }));
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
        }
        Ok(())
    }
}