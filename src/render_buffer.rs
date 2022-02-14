#[derive(Debug)]
pub struct RenderBuffer(pub(crate) gl::types::GLuint);

impl RenderBuffer {
    pub fn new() -> RenderBuffer {
        let mut buffer = 0 as gl::types::GLuint;
        gl_function!(GenRenderbuffers(1, &mut buffer));
        RenderBuffer(buffer)
    }

    pub fn bind(&self) {
        gl_function!(BindRenderbuffer(gl::RENDERBUFFER, self.0));
    }

    pub fn unbind() {
        gl_function!(BindRenderbuffer(gl::RENDERBUFFER, 0));
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        gl_function!(DeleteRenderbuffers(1, &mut self.0));
    }
}
