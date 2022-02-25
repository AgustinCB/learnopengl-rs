use crate::shader::Shader;
use gl;
use include_dir::Dir;
use regex::Regex;

pub enum ShaderType {
    Fragment = gl::FRAGMENT_SHADER as isize,
    Geometry = gl::GEOMETRY_SHADER as isize,
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
        Shader::new(shader_type as _, &self.load_file(glsl)?)
    }

    fn load_file(&self, glsl: &str) -> Result<String, String> {
        let mut content = self.shaders
            .get_file(glsl)
            .ok_or(format!("Shader {} not found", glsl))
            .map(|f| f.contents_utf8())?
            .map(|s| s.to_string())
            .ok_or("Empty file")
            .map_err(|s| s.to_string())?;
        let include_str = "#include".to_string();
        let includes_regex = Regex::new(
            r#"include "(.+)""#
        ).map_err(|e| e.to_string())?;

        for cap in includes_regex.captures_iter(&(content.clone())) {
            content = content.replace(&(include_str.clone() + r#" ""# + &cap[1] + r#"""#), &self.load_file(&cap[1])?);
        }
        Ok(content)
    }
}