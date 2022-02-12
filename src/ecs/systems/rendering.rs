use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use hecs::World;
use image::io::Reader;
use include_dir::{Dir, include_dir};
use nalgebra::Vector3;
use russimp::texture::{TextureType as MaterialTextureType};
use crate::buffer::Buffer;
use crate::camera::Camera;
use crate::ecs::components::{Mesh, Model, Shader, TextureInfo, Transform};
use crate::ecs::systems::system::System;
use crate::light::{DirectionalLight, Light, PointLight, SpotLight};
use crate::program::Program;
use crate::shader_loader::{ShaderLoader, ShaderType};
use crate::texture::{Texture, TextureType};
use crate::vertex_array::VertexArray;

static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

pub struct RenderingSystem {
    clear_color: Vector3<f32>,
    light_program: Program,
    main_camera: Rc<RefCell<Camera>>,
    meshes_program: Program,
    textures_loaded: HashMap<String, Arc<Texture>>,
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
        Ok(RenderingSystem {
            clear_color,
            main_camera: camera,
            light_program: Program::new(vec![
                shader_loader.load(ShaderType::Vertex, light_vertex_shader)?,
                shader_loader.load(ShaderType::Fragment, light_fragment_shader)?,
            ])?,
            meshes_program: Program::new(vec![
                shader_loader.load(ShaderType::Vertex, meshes_vertex_shader)?,
                shader_loader.load(ShaderType::Fragment, meshes_fragment_shader)?,
            ])?,
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
        }
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
            texture.set_image_2d_with_type(
                image.width() as u32,
                image.height() as u32,
                image.as_bytes(),
                image.color()
            )?;
            texture.generate_mipmap();
            self.textures_loaded.insert(texture_info.path.clone(), texture.clone());
            Ok(texture)
        }
    }

    fn set_lights<T: Light + Send + Sync + 'static>(&self, world: &mut World, name: &str) {
        let mut index = 0;
        for (_e, light) in world.query_mut::<&T>() {
            let name = format!("{}[{}]", name, index);
            light.set_light_in_program(&self.meshes_program, &name);
            self.meshes_program.set_uniform_i1(&format!("{}.set", name), 1);
            index += 1;
        }
    }

    fn draw_lights<T: Light + Send + Sync + 'static>(&self, world: &mut World) -> Result<(), String> {
        let look_at_matrix = (*self.main_camera).borrow().look_at_matrix();
        let projection = (*self.main_camera).borrow().projection();
        for (_e, (light, mesh, shader)) in world.query_mut::<(&T, &Mesh, &Shader)>() {
            light.set_light_drawing_program(
                &self.light_program, "light.specular", "model", ("view", &look_at_matrix), ("projection", &projection)
            );
            let n_vertices = mesh.vertices.len();
            shader.vertex_array.bind();
            gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32,));
            VertexArray::unbind();
        }
        Ok(())
    }

    fn render_mesh(&self, shader: &Shader, mesh: &Mesh) {
        let mut diffuse_index = 0;
        let mut specular_index = 0;
        if let Some(infos) = &mesh.textures {
            for (texture, info) in shader.textures.iter().zip(infos.iter()) {
                texture.bind(gl::TEXTURE0 + info.id as u32);
                let (texture_type, texture_index) = if info.texture_type == MaterialTextureType::Diffuse {
                    let index = diffuse_index;
                    diffuse_index += 1;
                    ("diffuse", index)
                } else {
                    let index = specular_index;
                    specular_index += 1;
                    ("specular", index)
                };
                self.meshes_program.set_uniform_i1(&format!("material.{}{}", texture_type, texture_index), info.id as i32);
            }
        }
        self.meshes_program.set_uniform_i1("material.n_diffuse", diffuse_index);
        self.meshes_program.set_uniform_i1("material.n_specular", specular_index);
        let shininess = mesh.shininess.clone().unwrap_or(32f32);
        self.meshes_program.set_uniform_f1("material.shininess", shininess);
        let n_vertices = mesh.vertices.len();
        shader.vertex_array.bind();
        gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32));
        VertexArray::unbind();
    }
}

impl System for RenderingSystem {
    fn name(&self) -> &str {
        "Rendering System"
    }

    fn start(&self, world: &mut World) -> Result<(), String> {
        for (_e, (shader, mesh)) in world.query_mut::<(&Shader, &Mesh)>() {
            self.setup_gl_objects(shader, mesh)?;
        }
        for (_e, model) in world.query::<&Model>().iter() {
            for (mesh, shader) in model.0.iter() {
                self.setup_gl_objects(shader, mesh)?;
            }
        }
        gl_function!(Enable(gl::DEPTH_TEST));
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
        gl_function!(Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        self.meshes_program.use_program();
        self.set_lights::<DirectionalLight>(world, "directional_lights");
        self.set_lights::<SpotLight>(world, "spot_lights");
        self.set_lights::<PointLight>(world, "point_lights");
        self.meshes_program.set_uniform_v3("viewPos" , (*self.main_camera).borrow().position());
        self.meshes_program.set_uniform_matrix4("projection", &(*self.main_camera).borrow().projection());
        self.meshes_program.set_uniform_matrix4("view", &(*self.main_camera).borrow().look_at_matrix());
        for (_e, (mesh, shader, transform)) in world.query::<(&Mesh, &Shader, &Transform)>().iter() {
            self.meshes_program.set_uniform_matrix4("model", &transform.get_model_matrix());
            self.render_mesh(shader, mesh);
        }
        for (_e, (model, transform)) in world.query::<(&Model, &Transform)>().iter() {
            self.meshes_program.set_uniform_matrix4("model", &transform.get_model_matrix());
            for (mesh, shader) in model.0.iter() {
                self.render_mesh(shader, mesh);
            }
        }
        self.light_program.use_program();
        self.draw_lights::<DirectionalLight>(world)?;
        self.draw_lights::<SpotLight>(world)?;
        self.draw_lights::<PointLight>(world)?;
        Ok(())
    }
}