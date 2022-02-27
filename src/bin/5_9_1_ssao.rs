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
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::buffer::Buffer;
use learnopengl::camera::Camera;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Input, Mesh, Shader as MeshShader, SkipRendering, TextureInfo, Transform};
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

fn random_kernel() -> Vec<Vector3<f32>> {
    let mut kernel = vec![];
    let mut rng = thread_rng();
    for i in 0..64 {
        let mut sample = Vector3::new(
            rng.gen_range(-1f32..1f32), rng.gen_range(-1f32..1f32), rng.gen_range(0f32..1f32)
        ).normalize();
        sample *= rng.gen_range(0f32..1f32);
        let scale = i as f32/64f32;
        let scale = 0.1f32 + scale * scale * (1f32 - 0.1f32);
        sample *= scale;
        kernel.push(sample);
    }
    kernel
}

fn noise_texture() -> Vec<Vector3<f32>> {
    let mut noise = vec![];
    let mut rng = thread_rng();
    for _ in 0..16 {
        noise.push(
            Vector3::new(rng.gen_range(-1f32..1f32), rng.gen_range(-1f32..1f32), 0f32)
        );
    }
    noise
}

#[derive(Clone, Copy, Debug)]
enum RenderingTarget {
    Ssao = 0,
    Position = 1,
    Normal = 2,
    Albedo = 3,
    Occlusion = 4,
    Blur = 5,
}

impl RenderingTarget {
    fn iterator() -> Cycle<Iter<'static, RenderingTarget>> {
        static VALUES: [RenderingTarget; 6] = [Ssao, Position, Normal, Albedo, Occlusion, Blur];
        VALUES.iter().cycle()
    }
}

struct FrameBufferSystem {
    blur_program: Program,
    blur_ssao: FrameBuffer,
    camera: Rc<RefCell<Camera>>,
    frame_buffer: MultipleRenderTarget,
    greyscale_quad_program: Program,
    light_program: Program,
    program: Program,
    quad_program: Program,
    ssao: FrameBuffer,
    ssao_program: Program,
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
            TextureFormat::FloatingPoint, TextureFormat::FloatingPoint, TextureFormat::UnsignedByte,
        ]);
        let ssao = FrameBuffer::intermediate_with_format(800, 600, TextureFormat::Grey);
        let blur_ssao = FrameBuffer::intermediate_with_format(800, 600, TextureFormat::Grey);
        let quad_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, "15.1-postprocessing_vertex.glsl")?,
            shader_loader.load(ShaderType::Fragment, "26.1-quad_fragment.glsl")?
        ])?;
        let greyscale_quad_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, "15.1-postprocessing_vertex.glsl")?,
            shader_loader.load(ShaderType::Fragment, "27.1-greyscale_quad_fragment.glsl")?
        ])?;
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/25.1-blur_vertex.glsl"))?,
            shader_loader.load(ShaderType::Fragment, "27.1-lighting_pass_fragment.glsl")?,
        ])?;
        let light_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex as _, "26.1-bloom_light_vertex.glsl")?,
            shader_loader.load(ShaderType::Fragment as _, "26.1-bloom_light_fragment.glsl")?,
        ])?;
        let ssao_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, "15.1-postprocessing_vertex.glsl")?,
            shader_loader.load(ShaderType::Fragment, "27.1-ssao_fragment.glsl")?,
        ])?;
        let blur_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, "15.1-postprocessing_vertex.glsl")?,
            shader_loader.load(ShaderType::Fragment, "27.1-blur_fragment.glsl")?,
        ])?;
        greyscale_quad_program.use_program();
        greyscale_quad_program.set_uniform_i1("texture1", 0);
        quad_program.use_program();
        quad_program.set_uniform_i1("texture1", 0);
        program.use_program();
        program.set_uniform_i1("gPosition", 0);
        program.set_uniform_i1("gNormal", 1);
        program.set_uniform_i1("gAlbedo", 2);
        program.set_uniform_i1("ssao", 3);
        light_program.use_program();
        light_program.bind_uniform_block("Matrices", 0);
        ssao_program.use_program();
        ssao_program.set_uniform_i1("gPosition", 0);
        ssao_program.set_uniform_i1("gNormal", 1);
        for (i, sample) in random_kernel().into_iter().enumerate() {
            ssao_program.set_uniform_v3(&format!("samples[{}]", i), sample);
        }
        for (i, noise) in noise_texture().into_iter().enumerate() {
            ssao_program.set_uniform_v3(&format!("noise[{}]", i), noise);
        }
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(FrameBufferSystem {
            blur_program,
            blur_ssao,
            camera,
            frame_buffer,
            greyscale_quad_program,
            light_program,
            program,
            quad_program,
            ssao,
            ssao_program,
            vao,
            targets: RefCell::new(RenderingTarget::iterator().peekable()),
            _vbo: vbo,
        })
    }

    fn render_ssao_texture(&self) {
        gl_function!(Enable(gl::DEPTH_TEST));
        self.ssao.bind();
        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        self.ssao_program.use_program();
        let camera = (*self.camera).borrow();
        self.ssao_program.set_uniform_matrix4("projection", &camera.projection());
        for (i, texture) in self.frame_buffer.textures[0..2].iter().enumerate() {
            texture.bind(gl::TEXTURE0 + i as u32);
        }
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
        FrameBuffer::unbind();
    }

    fn render_blur_ssao_texture(&self) {
        self.render_ssao_texture();
        self.blur_ssao.bind();
        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        self.blur_program.use_program();
        self.ssao.texture.bind(gl::TEXTURE0);
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
        FrameBuffer::unbind();
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
                gl_function!(Disable(gl::DEPTH_TEST));
                self.quad_program.use_program();
                self.frame_buffer.textures.get(0).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Normal => {
                gl_function!(Disable(gl::DEPTH_TEST));
                self.quad_program.use_program();
                self.frame_buffer.textures.get(1).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Albedo => {
                gl_function!(Disable(gl::DEPTH_TEST));
                self.quad_program.use_program();
                self.frame_buffer.textures.get(2).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Occlusion => {
                gl_function!(Enable(gl::DEPTH_TEST));
                self.render_ssao_texture();
                gl_function!(Disable(gl::DEPTH_TEST));
                self.greyscale_quad_program.use_program();
                self.ssao.texture.bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            }
            RenderingTarget::Blur => {
                gl_function!(Enable(gl::DEPTH_TEST));
                self.render_blur_ssao_texture();
                gl_function!(Disable(gl::DEPTH_TEST));
                self.greyscale_quad_program.use_program();
                self.blur_ssao.texture.bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            }
            RenderingTarget::Ssao => {
                gl_function!(Enable(gl::DEPTH_TEST));
                self.render_blur_ssao_texture();
                self.program.use_program();
                for (i, texture) in self.frame_buffer.textures.iter().enumerate() {
                    texture.bind(gl::TEXTURE0 + i as u32);
                }
                self.ssao.texture.bind(gl::TEXTURE0 + self.frame_buffer.textures.len() as u32);
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
        "27.1-geometry_vertex.glsl",
        "27.1-gbuffer_fragment.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "25.1-bloom_light_fragment.glsl",
        16,
    )?;
    let light_cube = cube_mesh(vec![]);
    let point_light = PointLight::new(
        Vector3::new(0f32, 4f32, 0f32),
        Vector3::new(0.1f32, 0.1f32, 0.35f32),
        Vector3::new(0.2f32, 0.2f32, 0.7f32),
        Vector3::new(0.2f32, 0.2f32, 0.7f32),
        1f32,
        0.09f32,
        0.032f32,
    );
    let e = game.spawn_light(point_light, &light_cube)?;
    game.add_to(e, SkipRendering)?;
    let model_path = format!("{}/../LOGL/resources/objects/backpack/backpack.obj", env!("CARGO_MANIFEST_DIR"));
    let mut cube = cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/marble.jpg", env!("CARGO_MANIFEST_DIR")),
        },
    ]);
    cube.normals = Some(
        cube.normals.clone().unwrap().into_iter()
            .map(|v| v * -1f32)
            .collect_vec()
    );
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(0f32, 3.5f32, 0f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(20f32, 9f32, 20f32),
    })?;
    let model = game.load_model(&model_path)?;
    game.spawn_loaded_model(&model, Transform {
        position: Vector3::zeros(),
        rotation: Rotation3::from_axis_angle(&Vector3::x_axis(), 270f32.to_radians()),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    game.spawn((Input::new(vec![InputType::Keyboard]), RenderingControl {
        backward: Keycode::Q,
        forward: Keycode::E,
    }));
    game.play_with_fps_camera(vec![Box::new(FrameBufferSystem::new(game.camera())?)])?;
    Ok(())
}
