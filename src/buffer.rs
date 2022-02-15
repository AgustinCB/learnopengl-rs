use super::gl_function;
use gl;
use std::mem::{size_of, transmute};
use std::ptr;

#[derive(Debug)]
pub struct Buffer(gl::types::GLuint, gl::types::GLenum);

impl Buffer {
    pub fn new(buffer_type: gl::types::GLenum) -> Buffer {
        let mut buffer = 0 as gl::types::GLuint;
        gl_function!(GenBuffers(1, &mut buffer));
        Buffer(buffer, buffer_type)
    }

    pub fn allocate_data<T>(&self, size: usize) {
        gl_function!(BufferData(
            self.1,
            (size_of::<T>() * size) as isize,
            ptr::null(),
            gl::STATIC_DRAW
        ))
    }

    pub fn set_sub_data<T>(&self, from: usize, to: usize, data: &[T]) {
        gl_function!(BufferSubData(
            self.1, (size_of::<T>() * from) as isize, (size_of::<T>() * to) as isize, transmute(&data[0]),
        ));
    }

    pub fn set_data<T>(&self, data: &[T], drawing_mode: gl::types::GLenum) {
        gl_function!(BufferData(
            self.1,
            (size_of::<T>() * data.len()) as isize,
            transmute(&data[0]),
            drawing_mode
        ));
    }

    pub fn link_to_binding_point(&self, binding_point: usize, from: usize, to: usize) {
        gl_function!(BindBufferRange(self.1, binding_point as _, self.0, from as _, to as _));
    }

    pub fn bind(&self) {
        gl_function!(BindBuffer(self.1, self.0));
    }

    pub fn unbind(&self) {
        gl_function!(BindBuffer(self.1, 0));
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        gl_function!(DeleteBuffers(1, &mut self.0));
    }
}
