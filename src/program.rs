use super::gl_function;
use crate::shader::Shader;
use gl;
use nalgebra::{Matrix4, Vector3};
use std::ffi::CString;
use std::ptr;
use log::warn;

fn check_success(
    resource: gl::types::GLuint,
    success_type: gl::types::GLenum,
) -> Result<(), String> {
    let mut status = gl::FALSE as gl::types::GLint;
    gl_function!(GetProgramiv(resource, success_type, &mut status));

    if status != (gl::TRUE as gl::types::GLint) {
        let mut len = 0;
        gl_function!(GetProgramiv(resource, gl::INFO_LOG_LENGTH, &mut len));
        let mut buf = [0].repeat(len as usize - 1);
        gl_function!(GetProgramInfoLog(
            resource,
            len,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut gl::types::GLchar,
        ));
        Err(std::str::from_utf8(&buf)
            .ok()
            .expect("ShaderInfoLog not valid utf8")
            .to_string())
    } else {
        Ok(())
    }
}

pub struct Program(gl::types::GLuint);

impl Program {
    pub fn new(shaders: Vec<Shader>) -> Result<Program, String> {
        let program = gl_function!(CreateProgram());
        for shader in shaders.iter() {
            gl_function!(AttachShader(program, shader.0));
        }
        gl_function!(LinkProgram(program));
        check_success(program, gl::LINK_STATUS)?;
        Ok(Program(program))
    }

    pub fn use_program(&self) {
        gl_function!(UseProgram(self.0));
    }

    pub fn set_uniform_f1(&self, uniform: &str, x: f32) {
        let location = self.find_uniform(uniform);
        gl_function!(Uniform1f(location, x));
    }

    pub fn set_uniform_v4(&self, uniform: &str, x: f32, y: f32, z: f32, w: f32) {
        let location = self.find_uniform(uniform);
        gl_function!(Uniform4f(location, x, y, z, w));
    }

    pub fn set_uniform_v3(&self, uniform: &str, vector: Vector3<f32>) {
        let location = self.find_uniform(uniform);
        gl_function!(Uniform3f(location, vector.data.0[0][0], vector.data.0[0][1], vector.data.0[0][2]));
    }

    pub fn set_uniform_i1(&self, uniform: &str, value: i32) {
        let location = self.find_uniform(uniform);
        gl_function!(Uniform1i(location, value));
    }

    pub fn set_uniform_matrix4(&self, uniform: &str, matrix: &Matrix4<f32>) {
        let location = self.find_uniform(uniform);
        gl_function!(UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr()));
    }

    fn find_uniform(&self, uniform: &str) -> gl::types::GLint {
        let c_str = CString::new(uniform).unwrap();
        let location = gl_function!(GetUniformLocation(
            self.0,
            std::mem::transmute(c_str.as_ptr())
        ));
        if location == -1 {
            warn!("Uniform {} does not exist", uniform);
        }
        location
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        gl_function!(DeleteProgram(self.0));
    }
}
