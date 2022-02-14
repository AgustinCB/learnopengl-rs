use log::error;
use crate::render_buffer::RenderBuffer;
use crate::texture::{Texture, TextureType};

#[derive(Debug)]
pub struct FrameBuffer {
    _render_buffer: RenderBuffer,
    resource: gl::types::GLuint,
    pub texture: Texture,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> FrameBuffer {
        let mut frame_buffer = 0 as gl::types::GLuint;
        gl_function!(GenFramebuffers(1, &mut frame_buffer));
        gl_function!(BindFramebuffer(gl::FRAMEBUFFER, frame_buffer));

        let texture = Texture::new(TextureType::Texture2D);
        texture.just_bind();
        texture.allocate_space(width, height);
        gl_function!(TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _));
        gl_function!(TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _));

        gl_function!(FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, TextureType::Texture2D as u32, texture.0, 0));

        let render_buffer = RenderBuffer::new();
        render_buffer.bind();
        gl_function!(RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as _, height as _));
        gl_function!(FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, render_buffer.0));

        let status = gl_function!(CheckFramebufferStatus(gl::FRAMEBUFFER));
        if status != gl::FRAMEBUFFER_COMPLETE {
            error!("Error creating frame buffer, status code {}", status);
        }
        FrameBuffer::unbind();
        FrameBuffer {
            _render_buffer: render_buffer,
            texture: texture,
            resource: frame_buffer,
        }
    }

    pub fn bind(&self) {
        gl_function!(BindFramebuffer(gl::FRAMEBUFFER, self.resource));
    }

    pub fn unbind() {
        gl_function!(BindFramebuffer(gl::FRAMEBUFFER, 0));
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        gl_function!(DeleteFramebuffers(1, &self.resource));
    }
}