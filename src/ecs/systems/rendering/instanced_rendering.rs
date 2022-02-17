use std::sync::Arc;
use hecs::World;
use nalgebra::{Matrix4, Vector4};
use crate::buffer::Buffer;
use crate::ecs::components::{Border, get_flattened_matrices, InstancedMesh, InstancedModel, InstancedShader, Mesh, Shader, SkipRendering, Transparent};
use crate::program::Program;
use crate::shader_loader::{ShaderLoader, ShaderType};
use crate::vertex_array::VertexArray;

const INSTANCED_ARRAYS_VERTEX: &'static str = "19.1-instanced_arrays.glsl";
const INSTANCED_ARRAYS_FRAGMENT: &'static str = "12.1-modelloading.glsl";

pub struct InstancedRendering {
    pub(crate) program: Program,
}

impl InstancedRendering {
    pub fn new(shader_loader: &ShaderLoader) -> Result<InstancedRendering, String> {
        let program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, INSTANCED_ARRAYS_VERTEX)?,
            shader_loader.load(ShaderType::Fragment, INSTANCED_ARRAYS_FRAGMENT)?,
        ])?;
        program.bind_uniform_block("Matrices", 0);
        Ok(InstancedRendering {
            program
        })
    }

    pub fn shader_for_mesh(&mut self, shader: &Shader) -> Result<InstancedShader, String> {
        let offset_buffer = Arc::new(Buffer::new(gl::ARRAY_BUFFER));
        Ok(InstancedShader {
            offset_buffer,
            vertex_array: shader.vertex_array.clone(),
            vertex_buffer: shader.vertex_buffer.clone(),
            elements_buffer: shader.elements_buffer.clone(),
            textures: shader.textures.clone(),
        })
    }

    pub fn setup_world(&self, world: &mut World) -> Result<(), String> {
        for (_e, (shader, mesh)) in world.query_mut::<(&InstancedShader, &InstancedMesh)>() {
            self.setup(shader, &mesh.mesh, &mesh.models)?;
        }
        for (_e, model) in world.query::<&InstancedModel>().iter() {
            for (mesh, shader) in model.model.iter() {
                self.setup(shader, mesh, &model.models)?;
            }
        }
        Ok(())
    }

    fn setup(&self, shader: &InstancedShader, mesh: &Mesh, models: &[Matrix4<f32>]) -> Result<(), String> {
        shader.vertex_array.bind();
        shader.vertex_buffer.bind();
        shader.vertex_buffer.set_data(&mesh.flattened_data(), gl::STATIC_DRAW);
        shader.vertex_buffer.unbind();
        shader.offset_buffer.bind();
        shader.offset_buffer.set_data(&get_flattened_matrices(&models), gl::STATIC_DRAW);
        shader.offset_buffer.unbind();

        if let Some(elements_buffer) = &shader.elements_buffer {
            match &mesh.indices {
                Some(indices) => {
                    elements_buffer.bind();
                    elements_buffer.set_data(indices, gl::STATIC_DRAW);
                    elements_buffer.unbind();
                }
                None => Err("Elements buffer without indices")?,
            }
        }

        shader.vertex_buffer.bind();
        let total_size = mesh.vertex_info_size() as u32;
        let mut offset = 0;
        let mut attribute = 0;
        VertexArray::set_vertex_attrib_with_padding::<f32>(
            gl::FLOAT, attribute, total_size, 3, offset, false
        );
        offset += 3;
        attribute += 1;

        if mesh.normals.is_some() {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, attribute, total_size, 3, offset, false
            );
            offset += 3;
            attribute += 1;
        }

        if mesh.texture_coordinates.is_some() {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, attribute, total_size, 2, offset, false
            );
            attribute += 1;
        }
        shader.vertex_buffer.unbind();
        shader.offset_buffer.bind();
        let mat4size = Matrix4::<f32>::identity().len() as u32;
        let vec4size = Vector4::<f32>::identity().len() as u32;
        for i in attribute..attribute+4 {
            VertexArray::set_vertex_attrib_with_padding::<f32>(
                gl::FLOAT, i, mat4size, vec4size, (i - attribute) * vec4size, false
            );
            gl_function!(VertexAttribDivisor(i, 1));
        }
        shader.offset_buffer.unbind();
        VertexArray::unbind();
        Ok(())
    }

    pub fn render_world(&self, world: &mut World) {
        self.program.use_program();
        for (_e, (mesh, shader)) in world.query::<(&InstancedMesh, &InstancedShader)>().without::<Border>().without::<Transparent>().without::<SkipRendering>().iter() {
            gl_function!(StencilMask(0x00));
            self.render(&mesh.mesh, shader, mesh.models.len());
        }
        for (_e, model) in world.query::<&InstancedModel>().without::<Border>().without::<Transparent>().without::<SkipRendering>().iter() {
            gl_function!(StencilMask(0x00));
            for (mesh, shader) in model.model.iter() {
                self.render(mesh, shader, model.models.len());
            }
        }
    }

    fn render(&self, mesh: &Mesh, shader: &InstancedShader, models: usize) {
        mesh.set_program(&self.program, &shader.textures);
        let n_vertices = mesh.vertices.len();
        shader.vertex_array.bind();
        gl_function!(DrawArraysInstanced(gl::TRIANGLES, 0, n_vertices as _, models as _));
        VertexArray::unbind();
    }
}