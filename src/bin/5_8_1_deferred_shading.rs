#![feature(variant_count)]
#![feature(iter_advance_by)]

use std::cell::RefCell;
use std::iter::{Cycle, Peekable};
use std::rc::Rc;
use std::slice::Iter;
use hecs::World;
use include_dir::{Dir, include_dir};
use itertools::Itertools;
use nalgebra::{Rotation3, Vector3};
use rand::{Rng, thread_rng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::buffer::Buffer;
use learnopengl::camera::Camera;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Input, Mesh, Shader as MeshShader, SkipRendering, Transform};
use learnopengl::ecs::systems::input::InputType;
use learnopengl::ecs::systems::system::System;
use learnopengl::frame_buffer::FrameBuffer;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::{Light, PointLight};
use learnopengl::multiple_render_target::MultipleRenderTarget;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::shader_loader::{ShaderLoader, ShaderType};
use learnopengl::texture::TextureFormat;
use learnopengl::vertex_array::VertexArray;
use crate::RenderingTarget::*;

const QUAD_VERTICES: [f32; 24] = [
    -1f32, 1f32, 0f32, 1f32,
    -1f32, -1f32, 0f32, 0f32,
    1f32, -1f32, 1f32, 0f32,
    -1f32, 1f32, 0f32, 1f32,
    1f32, -1f32, 1f32, 0f32,
    1f32, 1f32, 1f32, 1f32,
];
static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

#[derive(Clone, Copy, Debug)]
enum RenderingTarget {
    DeferredShading = 0,
    Position = 1,
    Normal = 2,
    Albedo = 3,
    Specular = 4,
}

impl RenderingTarget {
    fn iterator() -> Cycle<Iter<'static, RenderingTarget>> {
        static VALUES: [RenderingTarget; 5] = [DeferredShading, Position, Normal, Albedo, Specular];
        VALUES.iter().cycle()
    }
}

struct FrameBufferSystem {
    camera: Rc<RefCell<Camera>>,
    program: Program,
    frame_buffer: MultipleRenderTarget,
    light_program: Program,
    quad_program: Program,
    greyscale_quad_program: Program,
    targets: RefCell<Peekable<Cycle<Iter<'static, RenderingTarget>>>>,
    vao: VertexArray,
    _vbo: Buffer,
}

struct RenderingControl {
    forward: Keycode,
    backward: Keycode,
}

impl FrameBufferSystem {
    pub fn new(camera: Rc<RefCell<Camera>>) -> Result<FrameBufferSystem, String> {
        let shader_loader = ShaderLoader::new(&SHADERS_DIR);
        let frame_buffer = MultipleRenderTarget::new_with_formats(800, 600, &vec![
            TextureFormat::FloatingPoint, TextureFormat::FloatingPoint, TextureFormat::UnsignedByteWithAlpha,
        ]);
        let quad_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/15.1-postprocessing_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/26.1-quad_fragment.glsl"))?
        ])?;
        let greyscale_quad_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/15.1-postprocessing_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/26.1-greyscale_quad_fragment.glsl"))?
        ])?;
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/25.1-blur_vertex.glsl"))?,
            shader_loader.load(ShaderType::Fragment, "26.1-lighting_pass_fragment.glsl")?,
        ])?;
        let light_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/26.1-bloom_light_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/26.1-bloom_light_fragment.glsl"))?,
        ])?;
        quad_program.use_program();
        quad_program.set_uniform_i1("texture1", 0);
        greyscale_quad_program.use_program();
        greyscale_quad_program.set_uniform_i1("texture1", 0);
        program.use_program();
        program.set_uniform_i1("gPosition", 0);
        program.set_uniform_i1("gNormal", 1);
        program.set_uniform_i1("gAlbedoSpec", 2);
        light_program.use_program();
        light_program.bind_uniform_block("Matrices", 0);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(FrameBufferSystem {
            camera,
            light_program,
            program,
            frame_buffer,
            greyscale_quad_program,
            quad_program,
            vao,
            targets: RefCell::new(RenderingTarget::iterator().peekable()),
            _vbo: vbo,
        })
    }
}

impl System for FrameBufferSystem {
    fn name(&self) -> &str {
        "Frame Buffer"
    }

    fn start(&self, world: &mut World) -> Result<(), String> {
        self.frame_buffer.bind();
        self.frame_buffer.set_draw_buffers();
        MultipleRenderTarget::unbind();
        self.program.use_program();
        for (i, (_e, light)) in world.query_mut::<&PointLight>().into_iter().enumerate() {
            let name = format!("point_lights[{}]", i);
            light.set_light_in_program(&self.program, &name);
            self.program.set_uniform_i1(&format!("{}.set", name), 1);
        }
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        self.frame_buffer.bind();
        gl_function!(ClearColor(0f32, 0f32, 0f32, 0f32));
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (input, rendering_control)) in _world.query_mut::<(&Input, &RenderingControl)>() {
            for e in input.events.iter() {
                match &e {
                    Event::KeyDown { keycode: Some(k), repeat, .. } if !*repeat && k == &rendering_control.backward => {
                        self.targets.borrow_mut().
                            advance_by(std::mem::variant_count::<RenderingTarget>() - 1)
                            .map_err(|e| e.to_string())?;
                    }
                    Event::KeyDown { keycode: Some(k), repeat, .. } if !*repeat && k == &rendering_control.forward => {
                        self.targets.borrow_mut().next();
                    }
                    _ => {},
                }
            }
        }
        Ok(())
    }

    fn late_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        MultipleRenderTarget::unbind();
        self.vao.bind();

        gl_function!(ClearColor(0f32, 0f32, 0f32, 1f32));
        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        match self.targets.borrow_mut().peek().unwrap().clone() {
            RenderingTarget::Position => {
                self.quad_program.use_program();
                self.frame_buffer.textures.get(0).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Normal => {
                self.quad_program.use_program();
                self.frame_buffer.textures.get(1).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Albedo => {
                self.quad_program.use_program();
                self.frame_buffer.textures.get(2).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Specular => {
                self.greyscale_quad_program.use_program();
                self.frame_buffer.textures.get(2).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::DeferredShading => {
                self.program.use_program();
                self.program.set_uniform_v3("viewPos", self.camera.borrow().position());
                for (i, texture) in self.frame_buffer.textures.iter().enumerate() {
                    texture.bind(gl::TEXTURE0 + i as u32);
                }
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));

                gl_function!(BindFramebuffer(gl::READ_FRAMEBUFFER, self.frame_buffer.resource));
                gl_function!(BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0));
                gl_function!(BlitFramebuffer(0, 0, 800, 600, 0, 0, 800, 600, gl::DEPTH_BUFFER_BIT, gl::NEAREST));
                FrameBuffer::unbind();
                self.light_program.use_program();
                for (_e, (light, mesh, shader)) in world.query_mut::<(&PointLight, &Mesh, &MeshShader)>() {
                    self.light_program.set_uniform_matrix4("model", &light.model);
                    self.light_program.set_uniform_v3("light.diffuse", light.diffuse);
                    let n_vertices = mesh.vertices.len();
                    shader.vertex_array.bind();
                    gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32,));
                    VertexArray::unbind();
                }
            }
        };
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new_with_anti_alias(
        "Deferred shading",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "26.1-gbuffer_fragment.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "25.1-bloom_light_fragment.glsl",
        16,
    )?;
    let mut rnd = thread_rng();
    let mut light_cube = cube_mesh(vec![]);
    light_cube.vertices = light_cube.vertices.iter().map(|v| v * 0.25).collect_vec();
    for _ in 0..32 {
        let position = Vector3::new(
            rnd.gen_range(-3f32..3f32),
            rnd.gen_range(-3f32..3f32),
            rnd.gen_range(-3f32..3f32),
        );
        let color = Vector3::new(
            rnd.gen_range(0.5f32..1f32),
            rnd.gen_range(0.5f32..1f32),
            rnd.gen_range(0.5f32..1f32),
        );
        let point_light = PointLight::new(
            position,
            Vector3::new(0.1f32, 0.1f32, 0.1f32),
            color,
            Vector3::zeros(),
            1f32,
            1f32,
            0f32,
        );
        let e = game.spawn_light(point_light, &light_cube)?;
        game.add_to(e, SkipRendering)?;
    }
    let model_path = format!("{}/../LOGL/resources/objects/backpack/backpack.obj", env!("CARGO_MANIFEST_DIR"));
    let model = game.load_model(&model_path)?;

    for position in vec![
        Vector3::new(-3f32, -0.5f32, -3f32),
        Vector3::new(0f32, -0.5f32, -3f32),
        Vector3::new(3f32, -0.5f32, -3f32),
        Vector3::new(-3f32, -0.5f32, 0f32),
        Vector3::new(0f32, -0.5f32, 0f32),
        Vector3::new(3f32, -0.5f32, 0f32),
        Vector3::new(-3f32, -0.5f32, 3f32),
        Vector3::new(0f32, -0.5f32, 3f32),
        Vector3::new(3f32, -0.5f32, 3f32),
    ] {
        game.spawn_loaded_model(&model, Transform {
            position,
            rotation: Rotation3::identity(),
            scale: Vector3::new(1f32, 1f32, 1f32),
        })?;
    }
    game.spawn((Input::new(vec![InputType::Keyboard]), RenderingControl {
        backward: Keycode::Q,
        forward: Keycode::E,
    }));
    game.play_with_fps_camera(vec![Box::new(FrameBufferSystem::new(game.camera())?)])?;
    Ok(())
}
