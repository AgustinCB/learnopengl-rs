use std::ffi::CString;
use gl;
use std::ptr;
use crate::shader::Shader;
use super::gl_function;


fn check_success(resource: gl::types::GLuint, success_type: gl::types::GLenum) -> Result<(), String> {
    let mut status = gl::FALSE as gl::types::GLint;
    gl_function!(GetProgramiv(resource, success_type, &mut status));

    if status != (gl::TRUE as gl::types::GLint) {
        let mut len = 0;
        gl_function!(GetProgramiv(resource, gl::INFO_LOG_LENGTH, &mut len));
        let mut buf = Vec::with_capacity(len as usize - 1);
        gl_function!(GetProgramInfoLog(
            resource,
            len,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut gl::types::GLchar,
        ));
        Err(std::str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8").to_string())
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

    pub fn set_uniform_v4(
        &self, uniform: &str, x: f32, y: f32, z: f32, w: f32,
    ) {
        let c_str = CString::new(uniform).unwrap();
        let location = gl_function!(GetUniformLocation(self.0, std::mem::transmute(c_str.as_ptr())));
        eprintln!("{}", location);
        gl_function!(Uniform4f(location, x, y, z, w));
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        gl_function!(DeleteProgram(self.0));
    }
}
