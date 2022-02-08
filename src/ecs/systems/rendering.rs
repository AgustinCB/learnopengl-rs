use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Read};
use std::rc::Rc;
use hecs::World;
use include_dir::{Dir, include_dir};
use russimp::texture::{TextureType as MaterialTextureType};
use crate::buffer::Buffer;
use crate::camera::Camera;
use crate::ecs::components::{Mesh, Shader, TextureInfo, Transform};
use crate::ecs::systems::system::System;
use crate::light::{DirectionalLight, Light, PointLight, SpotLight};
use crate::program::Program;
use crate::shader_loader::{ShaderLoader, ShaderType};
use crate::texture::{Texture, TextureType};
use crate::vertex_array::VertexArray;

static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

pub struct RenderingSystem {
    light_program: Program,
    main_camera: Rc<RefCell<Camera>>,
    meshes_program: Program,
}

impl RenderingSystem {
    pub fn new(
        camera: Rc<RefCell<Camera>>,
        light_vertex_shader: &'static str,
        light_fragment_shader: &'static str,
        meshes_vertex_shader: &'static str,
        meshes_fragment_shader: &'static str,
    ) -> Result<RenderingSystem, String> {
        let shader_loader = ShaderLoader::new(&SHADERS_DIR);
        Ok(RenderingSystem {
            main_camera: camera,
            light_program: Program::new(vec![
                shader_loader.load(ShaderType::Vertex, light_vertex_shader)?,
                shader_loader.load(ShaderType::Fragment, light_fragment_shader)?,
            ])?,
            meshes_program: Program::new(vec![
                shader_loader.load(ShaderType::Vertex, meshes_vertex_shader)?,
                shader_loader.load(ShaderType::Fragment, meshes_fragment_shader)?,
            ])?,
        })
    }

    pub fn shader_for_mesh(&mut self, mesh: &Mesh) -> Result<Shader, String> {
        let vertex_array = VertexArray::new();
        let vertex_buffer = Buffer::new(gl::ARRAY_BUFFER);
        let elements_buffer = if mesh.indices.is_some() {
            Some(Buffer::new(gl::ELEMENT_ARRAY_BUFFER))
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

    fn setup_textures(&self, textures: &Option<Vec<TextureInfo>>) -> Result<Vec<Texture>, String> {
        match textures {
            None => Ok(vec![]),
            Some(textures) => {
                textures.iter().map(|t| {
                    let texture = Texture::new(TextureType::Texture2D);
                    texture.bind(gl::TEXTURE0 + t.id as u32);
                    texture.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
                    texture.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
                    texture.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
                    texture.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
                    let f = File::open(&t.path)
                        .map_err(|e| e.to_string())?;
                    let mut reader = BufReader::new(f);
                    let mut buffer = Vec::new();
                    reader.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                    texture.set_image_2d(t.width as u32, t.height as u32, &buffer);
                    texture.generate_mipmap();
                    Ok(texture)
                }).collect::<Result<Vec<Texture>, String>>()
            }
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
            shader.vertex_array.bind();
            light.set_light_drawing_program(
                &self.light_program, "light.specular", "model", ("view", &look_at_matrix), ("projection", &projection)
            );
            let n_vertices = mesh.vertices.len();
            gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32,));
            VertexArray::unbind();
        }
        Ok(())
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
        Ok(())
    }

    fn update(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, world: &mut World) -> Result<(), String> {
        self.meshes_program.use_program();
        self.set_lights::<DirectionalLight>(world, "directional_lights");
        self.set_lights::<SpotLight>(world, "spot_lights");
        self.set_lights::<PointLight>(world, "point_lights");
        self.meshes_program.set_uniform_v3("viewPos" , (*self.main_camera).borrow().position());
        self.meshes_program.set_uniform_matrix4("projection", &(*self.main_camera).borrow().projection());
        self.meshes_program.set_uniform_matrix4("view", &(*self.main_camera).borrow().look_at_matrix());
        for (_e, (shader, mesh, transform)) in world.query::<(&Shader, &Mesh, &Transform)>().iter() {
            self.meshes_program.set_uniform_matrix4("model", &transform.get_model_matrix());
            shader.vertex_array.bind();
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
            self.meshes_program.set_uniform_f1("material.shininess", 32f32);
            let n_vertices = mesh.vertices.len();
            gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32));
            VertexArray::unbind();
        }
        self.light_program.use_program();
        self.draw_lights::<DirectionalLight>(world)?;
        self.draw_lights::<SpotLight>(world)?;
        self.draw_lights::<PointLight>(world)?;
        Ok(())
    }
}