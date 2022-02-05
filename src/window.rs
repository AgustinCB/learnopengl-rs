use gl;
use sdl2::{EventPump, TimerSubsystem, VideoSubsystem};
use sdl2::event::EventPollIterator;
use sdl2::video::{GLContext, GLProfile, Window as SDL2Window};

pub struct Window {
    events: EventPump,
    gl_context: GLContext,
    timer: TimerSubsystem,
    video: VideoSubsystem,
    window: SDL2Window,
}

impl Window {
    pub fn new(name: &str, width: usize, height: usize) -> Result<Window, String> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        let attrs = video.gl_attr();

        attrs.set_context_major_version(3);
        attrs.set_context_minor_version(3);
        attrs.set_context_profile(GLProfile::Core);
        #[cfg(target_os = "macos")]
            attrs.set_context_flags().forward_compatible().set();

        let window = video
            .window(name, width as _, height as _)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);
        let event_pump = sdl_context.event_pump()?;
        let sdl_timer = sdl_context.timer().unwrap();
        Ok(Window {
            events: event_pump,
            timer: sdl_timer,
            gl_context,
            video,
            window,
        })
    }

    pub fn ticks(&self) -> u32 {
        self.timer.ticks()
    }

    pub fn swap_buffers(&self) {
        self.window.gl_swap_window()
    }

    pub fn delay(&mut self, ms: usize) {
        self.timer.delay(ms as _);
    }

    pub fn events(&mut self) -> EventPollIterator {
        self.events.poll_iter()
    }
}