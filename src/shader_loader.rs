use std::collections::HashMap;
use crate::shader::Shader;
use gl;
use include_dir::Dir;

pub enum ShaderType {
    Fragment = gl::FRAGMENT_SHADER as isize,
    Vertex = gl::VERTEX_SHADER as isize,
}

pub struct ShaderLoader {
    shaders: &'static Dir<'static>,
}

impl ShaderLoader {
    pub fn new(shaders: &'static Dir) -> ShaderLoader {
        ShaderLoader {
            shaders
        }
    }

    pub fn load(&self, shader_type: ShaderType, glsl: &'static str) -> Result<Shader, String> {
        let content = self.shaders
            .get_file(glsl)
            .ok_or("Shader not found")
            .map_err(|s| s.to_string())
            .map(|f| f.contents_utf8())?;
        let content = content.ok_or("Empty file").map_err(|s| s.to_string())?;
        Shader::new(shader_type as _, content)
    }
}