use gl;
use sdl2::{EventPump, Sdl, TimerSubsystem, VideoSubsystem};
use sdl2::event::EventPollIterator;
use sdl2::video::{GLContext, GLProfile, Window as SDL2Window};

pub struct Window {
    events: Option<EventPump>,
    gl_context: GLContext,
    sdl_context: Sdl,
    timer: TimerSubsystem,
    video: VideoSubsystem,
    window: SDL2Window,
    now: usize,
    last: usize,
}

impl Window {
    pub fn new(name: &str, width: usize, height: usize) -> Result<Window, String> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        let attrs = video.gl_attr();

        attrs.set_context_major_version(4);
        attrs.set_context_minor_version(1);
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
        let sdl_timer = sdl_context.timer().unwrap();
        Ok(Window {
            events: None,
            last: 0,
            now: 0,
            timer: sdl_timer,
            sdl_context,
            gl_context,
            video,
            window,
        })
    }

    pub fn start_timer(&mut self) {
        self.now = self.timer.performance_counter() as _;
    }

    pub fn delta_time(&mut self) -> f32 {
        self.last = self.now;
        self.now = self.timer.performance_counter() as _;
        (self.now - self.last) as f32 * 1000f32 / self.timer.performance_frequency() as f32
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

    pub fn get_pumper(&mut self) -> EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    pub fn events(&mut self) -> EventPollIterator {
        if self.events.is_none() {
            self.events = Some(self.get_pumper())
        }
        self.events.as_mut().unwrap().poll_iter()
    }
}