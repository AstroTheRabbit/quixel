use std::time::{Duration, Instant};

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    error::OsError,
    event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct QuixelWindow<T>
where
    T: WindowProcessor,
{
    pub window: Window,
    pub pixels: Pixels,
    pub event_loop: EventLoop<()>,
    pub window_processor: T,
}

impl<T> QuixelWindow<T>
where
    T: WindowProcessor,
{
    pub fn new(
        window_builder: WindowBuilder,
        window_processor: T,
    ) -> Result<Self, WindowCreationError> {
        let event_loop = EventLoop::new();
        let window = match window_builder.build(&event_loop) {
            Ok(w) => w,
            Err(e) => return Err(WindowCreationError::WindowError(e)),
        };

        let size = window.inner_size();
        let texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = match Pixels::new(size.width, size.height, texture) {
            Ok(p) => p,
            Err(e) => return Err(WindowCreationError::PixelsError(e)),
        };

        Ok(Self {
            window,
            pixels,
            event_loop,
            window_processor,
        })
    }

    pub fn start(mut self) {
        self.window_processor
            .on_start(&mut self.window, &mut self.pixels);
        let mut dt = Instant::now();
        self.event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait();
            match event {
                Event::WindowEvent {
                    window_id: _,
                    event,
                } => match event {
                    WindowEvent::Resized(size) => {
                        self.pixels.resize_surface(size.width, size.height).unwrap();
                        self.pixels.resize_buffer(size.width, size.height).unwrap();
                        self.window.request_redraw();
                        self.window_processor.on_window_resize(size);
                    }
                    // WindowEvent::Moved(_) => todo!(),
                    WindowEvent::CloseRequested => {
                        self.window_processor.on_quit();
                        control_flow.set_exit();
                    }
                    WindowEvent::Destroyed => {
                        self.window_processor.on_quit();
                        control_flow.set_exit();
                    }
                    // WindowEvent::DroppedFile(_) => todo!(),
                    // WindowEvent::HoveredFile(_) => todo!(),
                    // WindowEvent::HoveredFileCancelled => todo!(),
                    // WindowEvent::ReceivedCharacter(_) => todo!(),
                    WindowEvent::Focused(gained_focus) => {
                        self.window_processor.on_window_focus(gained_focus)
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        self.window_processor.on_input(input)
                    }
                    // WindowEvent::ModifiersChanged(_) => todo!(),
                    // WindowEvent::Ime(ime) => todo!(),
                    WindowEvent::CursorMoved { position, .. } => {
                        self.window_processor.on_cursor_move(position)
                    }
                    WindowEvent::CursorEntered { .. } => {
                        self.window_processor.on_cursor_in_window_change(true)
                    }
                    WindowEvent::CursorLeft { .. } => {
                        self.window_processor.on_cursor_in_window_change(false)
                    }
                    WindowEvent::MouseWheel { delta, phase, .. } => {
                        self.window_processor.on_scroll(delta, phase)
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.window_processor.on_click(state, button)
                    }
                    // WindowEvent::TouchpadMagnify { delta, phase } => todo!(),
                    // WindowEvent::SmartMagnify {} => todo!(),
                    // WindowEvent::TouchpadRotate { delta, phase } => todo!(),
                    // WindowEvent::TouchpadPressure { pressure, stage } => todo!(),
                    // WindowEvent::AxisMotion { axis, value } => todo!(),
                    // WindowEvent::Touch(touch) => todo!(),
                    // WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => todo!(),
                    // WindowEvent::ThemeChanged(theme) => todo!(),
                    // WindowEvent::Occluded(_) => todo!(),
                    _ => (),
                },
                // Event::DeviceEvent { device_id: _, event } => match event {
                //     DeviceEvent::Added => todo!(),
                //     DeviceEvent::Removed => todo!(),
                //     DeviceEvent::MouseMotion { delta } => todo!(),
                //     DeviceEvent::MouseWheel { delta } => todo!(),
                //     DeviceEvent::Motion { axis, value } => todo!(),
                //     DeviceEvent::Button { button, state } => todo!(),
                //     DeviceEvent::Key(_) => todo!(),
                //     DeviceEvent::Text { codepoint } => todo!(),
                //     _ => (),
                // },
                // Event::UserEvent(_) => todo!(),
                // Event::Suspended => todo!(),
                // Event::Resumed => todo!(),
                Event::MainEventsCleared => {
                    self.window_processor
                        .on_update(&mut self.window, control_flow, dt.elapsed());
                    self.window.request_redraw();
                    dt = Instant::now();
                }
                Event::RedrawRequested(_) => {
                    self.window_processor
                        .on_render(&mut self.window, &mut self.pixels);
                    if let Err(e) = self.pixels.render() {
                        self.window_processor.on_render_error(e, control_flow);
                    }
                }
                // Event::RedrawEventsCleared => todo!(),
                Event::LoopDestroyed => {
                    control_flow.set_exit();
                    self.window_processor.on_quit();
                }
                misc => self.window_processor.on_misc_event(misc),
            }
        });
    }
}

pub trait WindowProcessor: 'static {
    fn on_start(&mut self, window: &mut Window, pixels: &mut Pixels);
    fn on_update(&mut self, window: &mut Window, control_flow: &mut ControlFlow, dt: Duration);
    fn on_render(&mut self, window: &mut Window, pixels: &mut Pixels);
    fn on_render_error(&mut self, error: Error, control_flow: &mut ControlFlow);
    fn on_input(&mut self, input: KeyboardInput);
    fn on_click(&mut self, state: ElementState, button: MouseButton);
    fn on_scroll(&mut self, scroll_delta: MouseScrollDelta, phase: TouchPhase);
    fn on_cursor_move(&mut self, new_position: PhysicalPosition<f64>);
    fn on_cursor_in_window_change(&mut self, cursor_in_window: bool);
    fn on_window_resize(&mut self, size: PhysicalSize<u32>);
    fn on_window_focus(&mut self, gained_focus: bool);
    fn on_window_close(&mut self);
    fn on_quit(&mut self);
    fn on_misc_event(&mut self, event: Event<()>);
}

#[derive(Debug)]
pub enum WindowCreationError {
    WindowError(OsError),
    PixelsError(Error),
}
