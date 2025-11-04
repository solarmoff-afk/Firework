use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use glam::Vec4;
use moonwalk::MoonWalk;
use std::sync::{Arc, Mutex};

pub struct MoonBridge {
    event_loop: EventLoop<()>,
    moonwalk: Arc<Mutex<MoonWalk>>,
    _window_ptr: *mut Window,
}

impl MoonBridge {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Firework")
            .with_inner_size(winit::dpi::LogicalSize::new(600, 800))
            .build(&event_loop)?;

        let window_box = Box::new(window);
        let leaked_window: &'static Window = Box::leak(window_box);

        let moonwalk = MoonWalk::new(leaked_window)?;
        let moonwalk = Arc::new(Mutex::new(moonwalk));

        Ok(Self {
            event_loop,
            moonwalk,
            _window_ptr: leaked_window as *const Window as *mut Window,
        })
    }

    pub fn moonwalk(&self) -> std::sync::MutexGuard<MoonWalk> {
        self.moonwalk.lock().unwrap()
    }

    pub fn run(self) {
        let moonwalk = self.moonwalk.clone();
        let window_ptr = self._window_ptr;

        self.event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Wait);

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(size) => {
                            moonwalk.lock().unwrap().set_viewport(size.width, size.height);
                        }
                        _ => {}
                    }

                    Event::AboutToWait => {
                        let _ = moonwalk
                            .lock()
                            .unwrap()
                            .render_frame(Vec4::new(0.1, 0.2, 0.3, 1.0));
                    }

                    Event::LoopExiting => {
                        unsafe { drop(Box::from_raw(window_ptr)) }
                        std::process::exit(0);
                    }

                    _ => {}
                }
            })
            .expect("Event loop failed");
    }
}