use crate::gl_function;
use gl;
use std::mem::{size_of, transmute};
use std::ptr;

#[derive(Debug)]
pub struct VertexArray(gl::types::GLuint);

impl VertexArray {
    pub fn new() -> VertexArray {
        let mut vertex_array = 0 as gl::types::GLuint;
        gl_function!(GenVertexArrays(1, &mut vertex_array));
        VertexArray(vertex_array)
    }

    pub fn bind(&self) {
        gl_function!(BindVertexArray(self.0));
    }

    pub fn set_vertex_attrib<T>(
        gl_type: gl::types::GLenum,
        attribute: u32,
        size: u32,
        normalized: bool,
    ) {
        let normalized = if normalized { gl::TRUE } else { gl::FALSE };
        gl_function!(VertexAttribPointer(
            attribute,
            size as _,
            gl_type,
            normalized,
            size as i32 * size_of::<T>() as i32,
            ptr::null()
        ));
        gl_function!(EnableVertexAttribArray(attribute));
    }

    pub fn set_vertex_attrib_with_padding<T>(
        gl_type: gl::types::GLenum,
        attribute: u32,
        size: u32,
        padding: u32,
        start: u32,
        normalized: bool,
    ) {
        let normalized = if normalized { gl::TRUE } else { gl::FALSE };
        gl_function!(VertexAttribPointer(
            attribute,
            padding as _,
            gl_type,
            normalized,
            size as i32 * size_of::<T>() as i32,
            transmute(start as usize * size_of::<T>())
        ));
        gl_function!(EnableVertexAttribArray(attribute));
    }

    pub fn unbind() {
        gl_function!(BindVertexArray(0));
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        gl_function!(DeleteVertexArrays(1, &mut self.0))
    }
}
