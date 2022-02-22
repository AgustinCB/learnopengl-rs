use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use hecs::{Entity, World};
use image::EncodableLayout;
use image::io::Reader;
use include_dir::{Dir, include_dir};
use log::warn;
use nalgebra::{Matrix4, Scale3, Vector3};
use russimp::texture::{TextureType as AssimpTextureType};
use crate::buffer::Buffer;
use crate::camera::Camera;
use crate::ecs::components::{Border, ExtraUniform, Mesh, Model, Shader, SkipRendering, Skybox, SKYBOX_VERTICES, TextureInfo, Transform, Transparent, UniformValue};
use crate::ecs::systems::rendering::instanced_rendering::InstancedRendering;
use crate::ecs::systems::rendering::normal_mapping_rendering::NormalMappingRendering;
use crate::ecs::systems::system::System;
use crate::light::{DirectionalLight, Light, PointLight, SpotLight};
use crate::program::Program;
use crate::shader_loader::{ShaderLoader, ShaderType};
use crate::texture::{Texture, TextureType};
use crate::vertex_array::VertexArray;

static BORDER_VERTEX_SHADER: &'static str = "17.1-uniform_buffer_objects_vertex_border.glsl";
static BORDER_FRAGMENT_SHADER: &'static str = "14.1-border_color.glsl";
static SKYBOX_VERTEX_SHADER: &'static str = "17.1-uniform_buffer_object_vertex_skybox.glsl";
static SKYBOX_FRAGMENT_SHADER: &'static str = "16.1-skybox_fragment.glsl";
static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

fn set_lights<T: Light + Send + Sync + 'static>(program: &Program, world: &mut World, name: &str) {
    let mut index = 0;
    for (_e, light) in world.query_mut::<&T>() {
        let name = format!("{}[{}]", name, index);
        light.set_light_in_program(program, &name);
        program.set_uniform_i1(&format!("{}.set", name), 1);
        index += 1;
    }
}

pub mod instanced_rendering;
pub mod normal_mapping_rendering;

pub struct RenderingSystem {
    border_program: Program,
    clear_color: Vector3<f32>,
    pub(crate) instanced_rendering: InstancedRendering,
    pub(crate) normal_mapping_rendering: NormalMappingRendering,
    light_program: Program,
    main_camera: Rc<RefCell<Camera>>,
    meshes_program: Program,
    skybox_program: Program,
    textures_loaded: HashMap<String, Arc<Texture>>,
    uniform_buffer: Buffer,
}

impl RenderingSystem {
    pub fn new(
        camera: Rc<RefCell<Camera>>,
        clear_color: Vector3<f32>,
        light_vertex_shader: &'static str,
        light_fragment_shader: &'static str,
        meshes_vertex_shader: &'static str,
        meshes_fragment_shader: &'static str,
    ) -> Result<RenderingSystem, String> {
        let shader_loader = ShaderLoader::new(&SHADERS_DIR);
        let border_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, BORDER_VERTEX_SHADER)?,
            shader_loader.load(ShaderType::Fragment, BORDER_FRAGMENT_SHADER)?,
        ])?;
        let light_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, light_vertex_shader)?,
            shader_loader.load(ShaderType::Fragment, light_fragment_shader)?,
        ])?;
        let meshes_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, meshes_vertex_shader)?,
            shader_loader.load(ShaderType::Fragment, meshes_fragment_shader)?,
        ])?;
        let skybox_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, SKYBOX_VERTEX_SHADER)?,
            shader_loader.load(ShaderType::Fragment, SKYBOX_FRAGMENT_SHADER)?,
        ])?;
        border_program.bind_uniform_block("Matrices", 0);
        light_program.bind_uniform_block("Matrices", 0);
        meshes_program.bind_uniform_block("Matrices", 0);
        skybox_program.bind_uniform_block("Matrices", 0);
        let uniform_buffer = Buffer::new(gl::UNIFORM_BUFFER);
        let buffer_size = Matrix4::<f32>::identity().len() * 2;
        uniform_buffer.bind();
        uniform_buffer.allocate_data::<f32>(buffer_size);
        uniform_buffer.unbind();
        uniform_buffer.link_to_binding_point(0, 0, buffer_size);
        Ok(RenderingSystem {
            border_program,
            clear_color,
            light_program,
            meshes_program,
            skybox_program,
            uniform_buffer,
            instanced_rendering: InstancedRendering::new(&shader_loader)?,
            normal_mapping_rendering: NormalMappingRendering::new(&shader_loader)?,
            main_camera: camera,
            textures_loaded: HashMap::new(),
        })
    }

    pub fn shader_for_mesh(&mut self, mesh: &Mesh) -> Result<Shader, String> {
        let vertex_array = Arc::new(VertexArray::new());
        let vertex_buffer = Arc::new(Buffer::new(gl::ARRAY_BUFFER));
        let elements_buffer = if mesh.indices.is_some() {
            Some(Arc::new(Buffer::new(gl::ELEMENT_ARRAY_BUFFER)))
        } else {
            None
        };
        let textures = self.setup_textures(&mesh.textures)?;
        Ok(Shader {
            vertex_array,
            vertex_buffer,
            elements_buffer,
            textures,
        })
    }

    pub fn shader_for_skybox(&mut self, skybox: &Skybox) -> Result<Shader, String> {
        let vertex_array = Arc::new(VertexArray::new());
        let vertex_buffer = Arc::new(Buffer::new(gl::ARRAY_BUFFER));
        Ok(Shader {
            vertex_array,
            vertex_buffer,
            elements_buffer: None,
            textures: vec![self.setup_cubemap_texture(&skybox.texture_info)?],
        })
    }

    fn setup_skybox(&self, shader: &Shader) -> Result<(), String> {
        shader.vertex_array.bind();
        shader.vertex_buffer.bind();
        shader.vertex_buffer.set_data(&SKYBOX_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib::<f32>(gl::FLOAT, 0, 3, false);
        VertexArray::unbind();
        self.skybox_program.use_program();
        self.skybox_program.set_uniform_i1("skybox", 0);
        Ok(())
    }

    fn setup_gl_objects(&self, shader: &Shader, mesh: &Mesh) -> Result<(), String> {
        shader.vertex_array.bind();
        shader.vertex_buffer.bind();
        shader.vertex_buffer.set_data(&mesh.flattened_data(), gl::STATIC_DRAW);

        if let Some(elements_buffer) = &shader.elements_buffer {
            match &mesh.indices {
                Some(indices) => {
                    elements_buffer.set_data(indices, gl::STATIC_DRAW);
                }
                None => Err("Elements buffer without indices")?,
            }
        }

        let mut offset = 0;
        let mut attribute = 0;
        VertexArray::set_vertex_attrib_with_padding::<f32>(
            gl::FLOAT, attribute, mesh.vertex_info_size() as _, 3, offset, false
        );
        offset += 3;
        attribute += 1;

        if mesh.normals.is_some() {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, attribute, mesh.vertex_info_size() as _, 3, offset, false
            );
            offset += 3;
            attribute += 1;
        }

        if mesh.texture_coordinates.is_some() {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, attribute, mesh.vertex_info_size() as _, 2, offset, false
            );
            offset += 2;
            attribute += 1;
        }

        if mesh.tangents.is_some() {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, attribute, mesh.vertex_info_size() as _, 3, offset, false
            );
            offset += 3;
            attribute += 1;
        }

        if mesh.bitangents.is_some() {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, attribute, mesh.vertex_info_size() as _, 3, offset, false
            );
        }
        VertexArray::unbind();
        Ok(())
    }

    fn setup_textures(&mut self, textures: &Option<Vec<TextureInfo>>) -> Result<Vec<Arc<Texture>>, String> {
        match textures {
            None => Ok(vec![]),
            Some(textures) => {
                textures.iter().map(|t| {
                    self.texture_info_to_texture(t)
                }).collect::<Result<Vec<Arc<Texture>>, String>>()
            }
        }
    }

    fn setup_cubemap_texture(&mut self, texture_info: &TextureInfo) -> Result<Arc<Texture>, String> {
        if let Some(texture) = self.textures_loaded.get(&texture_info.path) {
            Ok(texture.clone())
        } else {
            let texture = Arc::new(Texture::new(TextureType::CubeMap));
            texture.just_bind();
            let root = Path::new(&texture_info.path);
            for (i, path) in vec!["right.jpg", "left.jpg", "top.jpg", "bottom.jpg", "front.jpg", "back.jpg"].into_iter().enumerate() {
                let path = root.join(path);
                let image = Reader::open(path).map_err(|e| e.to_string())?
                    .decode().map_err(|e| e.to_string())?
                    .to_rgba8();
                texture.set_cube_map_face(i as u32, image.width() as _, image.height() as _, image.as_bytes());
            }
            texture.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
            texture.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            texture.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
            texture.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
            texture.set_parameter(gl::TEXTURE_WRAP_R, gl::REPEAT);
            Ok(texture)
        }
    }

    fn texture_info_to_texture(&mut self, texture_info: &TextureInfo) -> Result<Arc<Texture>, String> {
        if let Some(texture) = self.textures_loaded.get(&texture_info.path) {
            Ok(texture.clone())
        } else {
            let texture = Arc::new(Texture::new(TextureType::Texture2D));
            texture.bind(gl::TEXTURE0 + texture_info.id as u32);
            texture.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
            texture.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
            texture.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
            texture.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            let image = Reader::open(&texture_info.path).map_err(|e| e.to_string())?
                .decode().map_err(|e| e.to_string())?
                .flipv();
            match texture.set_image_2d_with_type(
                image.width() as u32,
                image.height() as u32,
                image.as_bytes(),
                image.color()
            ) {
                Ok(()) => {},
                Err(_) => {
                    let image = image.to_rgba8();
                    texture.set_image_2d(
                        image.width() as u32,
                        image.height() as u32,
                        image.as_bytes(),
                    );
                }
            }
            texture.generate_mipmap();
            self.textures_loaded.insert(texture_info.path.clone(), texture.clone());
            Ok(texture)
        }
    }

    fn draw_lights<T: Light + Send + Sync + 'static>(&self, world: &mut World) -> Result<(), String> {
        for (_e, (light, mesh, shader)) in world.query_mut::<(&T, &Mesh, &Shader)>().without::<Skybox>() {
            light.set_light_drawing_program_no_globals(
                &self.light_program, "light.specular", "model",
            );
            let n_vertices = mesh.vertices.len();
            shader.vertex_array.bind();
            gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32,));
            VertexArray::unbind();
        }
        Ok(())
    }

    fn render_mesh(&self, program: &Program, shader: &Shader, mesh: &Mesh) {
        mesh.set_program(&program, &shader.textures);
        let n_vertices = mesh.vertices.len();
        shader.vertex_array.bind();
        gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32));
        VertexArray::unbind();
    }

    fn render_mesh_border(&self, meshes: &[(&Mesh, &Shader, &Border, &Transform)]) {
        for (mesh, shader, border, transform) in meshes.iter() {
            let model = transform.get_model_matrix() * Scale3::new(border.scale, border.scale, border.scale).to_homogeneous();
            self.border_program.set_uniform_matrix4("model", &model);
            self.border_program.set_uniform_v3("borderColor", border.color);
            let n_vertices = mesh.vertices.len();
            shader.vertex_array.bind();
            gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32));
            VertexArray::unbind();
        }
    }

    fn render_bordered_objects(&self, world: &mut World) -> Result<(), String> {
        gl_function!(StencilFunc(gl::ALWAYS, 1, 0xff));
        gl_function!(StencilMask(0xff));
        self.render_objects(
            world.query::<(&Mesh, &Shader, &Transform)>().without::<SkipRendering>().with::<Border>().iter(),
            world.query::<(&Model, &Transform)>().without::<SkipRendering>().with::<Border>().iter(),
            world,
        )?;
        gl_function!(StencilFunc(gl::NOTEQUAL, 1, 0xff));
        gl_function!(StencilMask(0x00));
        gl_function!(Disable(gl::DEPTH_TEST));
        self.border_program.use_program();
        for (_e, (mesh, shader, transform, border)) in world.query::<(&Mesh, &Shader, &Transform, &Border)>().without::<SkipRendering>().iter() {
            self.render_mesh_border(&vec![(mesh, shader, border, transform)]);
        }
        for (_e, (model, transform, border)) in world.query::<(&Model, &Transform, &Border)>().iter() {
            self.render_mesh_border(
                &model.0.iter().map(|(m, s)| (m, s, border, transform))
                    .collect::<Vec<_>>(),
            );
        }
        gl_function!(StencilMask(0xff));
        gl_function!(StencilFunc(gl::ALWAYS, 0, 0xff));
        gl_function!(Enable(gl::DEPTH_TEST));
        Ok(())
    }

    fn setup_program_globals(&self, world: &mut World) {
        let projection = (*self.main_camera).borrow().projection();
        let view = (*self.main_camera).borrow().look_at_matrix();
        self.uniform_buffer.bind();
        self.uniform_buffer.set_sub_data(0, view.len(), view.as_slice());
        self.uniform_buffer.set_sub_data(view.len(), projection.len(), projection.as_slice());
        self.uniform_buffer.unbind();
        self.set_rendering_program(&self.meshes_program, world);
        self.set_rendering_program(&self.normal_mapping_rendering.dynamic_calculation_program, world);
        self.set_rendering_program(&self.normal_mapping_rendering.precomputed_program, world);
        self.set_rendering_program(&self.instanced_rendering.program, world);
    }

    fn set_rendering_program(&self, program: &Program, world: &mut World) {
        program.use_program();
        set_lights::<DirectionalLight>(&program, world, "directional_lights");
        set_lights::<SpotLight>(&program, world, "spot_lights");
        set_lights::<PointLight>(&program, world, "point_lights");
        program.set_uniform_v3("viewPos", (*self.main_camera).borrow().position());
    }

    fn render_skybox(&self, world: &mut World) -> Result<(), String> {
        let skybox = world.query_mut::<&Shader>().with::<Skybox>().into_iter().next();
        if let Some((_e, shader)) = skybox {
            gl_function!(DepthFunc(gl::EQUAL));
            self.skybox_program.use_program();
            shader.vertex_array.bind();
            let texture = shader.textures.get(0).ok_or("Skybox with no texture".to_string())?;
            texture.bind(gl::TEXTURE0);
            gl_function!(DrawArrays(gl::TRIANGLES, 0, 36));
            VertexArray::unbind();
            gl_function!(DepthFunc(gl::LESS));
        }
        Ok(())
    }

    fn render_non_bordered_objects(&self, world: &mut World) -> Result<(), String> {
        gl_function!(StencilMask(0x00));
        self.render_objects(
            world.query::<(&Mesh, &Shader, &Transform)>().without::<Border>().without::<Transparent>().without::<SkipRendering>().iter(),
            world.query::<(&Model, &Transform)>().without::<Border>().without::<Transparent>().without::<SkipRendering>().iter(),
            world,
        )?;
        Ok(())
    }

    fn get_rendering_program(&self, mesh: &Mesh) -> &Program {
        match &mesh.textures {
            Some(textures) => {
                let normal_texture = textures.iter().find(|t| t.texture_type == AssimpTextureType::Normals);
                if normal_texture.is_some() {
                    &self.normal_mapping_rendering.get_program_for_mesh(mesh)
                } else {
                    &self.meshes_program
                }
            }
            None => &self.meshes_program
        }
    }

    fn render_objects<
        'a,
        I: Iterator<Item=(Entity, (&'a Mesh, &'a Shader, &'a Transform))>,
        J: Iterator<Item=(Entity, (&'a Model, &'a Transform))>
    >(
        &self, mesh_query_results: I, model_query_results: J, world: &World,
    ) -> Result<(), String> {
        for (e, (mesh, shader, transform)) in mesh_query_results {
            let program = self.get_rendering_program(mesh);
            program.use_program();
            self.set_mesh_uniforms(program, world, e, &transform)?;
            self.render_mesh(&program, shader, mesh);
        }
        for (e, (model, transform)) in model_query_results {
            for (mesh, shader) in model.0.iter() {
                let program = self.get_rendering_program(mesh);
                program.use_program();
                self.set_mesh_uniforms(program, world, e, &transform)?;
                self.render_mesh(&program, shader, mesh);
            }
        }
        Ok(())
    }

    fn set_mesh_uniforms(&self, program: &Program, world: &World, e: Entity, transform: &&Transform) -> Result<(), String> {
        let extra_uniforms = world.query_one::<&Vec<ExtraUniform>>(e).map_err(|e| e.to_string())?.get().cloned();
        if let Some(extra_uniforms) = extra_uniforms {
            for eu in extra_uniforms {
                match &eu.value {
                    UniformValue::Float(f) => {
                        program.set_uniform_f1(eu.name, *f);
                    }
                    UniformValue::Texture(i) => {
                        program.set_uniform_i1(eu.name, *i as _);
                    },
                    UniformValue::Matrix(m) => {
                        program.set_uniform_matrix4(eu.name, m);
                    }
                }
            }
        }
        program.set_uniform_matrix4("model", &transform.get_model_matrix());
        Ok(())
    }

    fn render_transparent_objects(&self, world: &mut World) -> Result<(), String> {
        let mut entities = vec![];
        for (e, transform) in world.query::<&Transform>().without::<SkipRendering>()
            .with::<Transparent>()
            .with::<Shader>()
            .iter() {
            entities.push((transform.position.data.0[0][2], e));
        }
        entities.sort_by(|(z, _), (z1, _)|
            if (*z - *z1).abs() < f32::EPSILON {
                Ordering::Equal
            } else if *z < *z1 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        );
        gl_function!(StencilMask(0x00));
        self.meshes_program.use_program();
        for (_, e) in entities {
            self.render_entity(&self.meshes_program, e, world)?;
        }
        Ok(())
    }

    fn render_entity(&self, program: &Program, e: Entity, world: &mut World) -> Result<(), String> {
        let mut mesh = world.query_one::<(&Mesh, &Shader, &Transform)>(e).map_err(|e| e.to_string())?;
        match mesh.get() {
            Some((mesh, shader, transform)) => {
                self.set_mesh_uniforms(program, world, e, &transform)?;
                self.render_mesh(&program, shader, mesh);
            }
            None => {
                let mut model = world.query_one::<(&Model, &Transform)>(e).map_err(|e| e.to_string())?;
                if let Some((model, transform)) = model.get() {
                    self.set_mesh_uniforms(program, world, e, &transform)?;
                    for (mesh, shader) in model.0.iter() {
                        self.render_mesh(&program, shader, mesh);
                    }
                } else {
                    warn!("Renderable entity {:?} has no mesh nor model", e);
                }
            }
        }
        Ok(())
    }
}

impl System for RenderingSystem {
    fn name(&self) -> &str {
        "Rendering System"
    }

    fn start(&self, world: &mut World) -> Result<(), String> {
        gl_function!(Enable(gl::DEPTH_TEST));
        gl_function!(DepthFunc(gl::LESS));
        for (_e, shader) in world.query_mut::<&Shader>().with::<Skybox>() {
            self.setup_skybox(shader)?;
        }
        for (_e, (shader, mesh)) in world.query_mut::<(&Shader, &Mesh)>() {
            self.setup_gl_objects(shader, mesh)?;
        }
        for (_e, model) in world.query::<&Model>().iter() {
            for (mesh, shader) in model.0.iter() {
                self.setup_gl_objects(shader, mesh)?;
            }
        }
        self.instanced_rendering.setup_world(world)?;
        gl_function!(ClearStencil(0));
        gl_function!(ClearColor(self.clear_color.x, self.clear_color.y, self.clear_color.z, 1.0));
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        gl_function!(Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT));
        self.setup_program_globals(world);
        self.instanced_rendering.render_world(world);
        self.render_non_bordered_objects(world)?;
        if world.query_mut::<&Border>().into_iter().next().is_some() {
            gl_function!(Enable(gl::STENCIL_TEST));
            gl_function!(StencilFunc(gl::ALWAYS, 0, 0xff));
            gl_function!(StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE));
            self.render_bordered_objects(world)?;
        } else {
            gl_function!(Disable(gl::STENCIL_TEST));
        }
        if world.query_mut::<&Transparent>().into_iter().next().is_some() {
            gl_function!(Enable(gl::BLEND));
            gl_function!(BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
            self.render_transparent_objects(world)?;
        } else {
            gl_function!(Disable(gl::BLEND));
        }
        self.light_program.use_program();
        self.draw_lights::<DirectionalLight>(world)?;
        self.draw_lights::<SpotLight>(world)?;
        self.draw_lights::<PointLight>(world)?;
        self.render_skybox(world)?;
        Ok(())
    }
}
