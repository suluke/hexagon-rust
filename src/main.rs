mod app;
mod constants;
mod controls;
mod model;
mod renderer;

use glutin::event::{DeviceEvent, ElementState, Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use renderer::Renderer;

fn main() {
    let a_event_loop = EventLoop::new();
    let a_winbuilder = WindowBuilder::new().with_title("Libre Hexagon");

    let a_win_ctx = ContextBuilder::new()
        .build_windowed(a_winbuilder, &a_event_loop)
        .unwrap();
    let a_win_ctx = unsafe { a_win_ctx.make_current().unwrap() };

    // We give an initial size of 1 by 1 because there will be a resize event anyways after window opening
    let mut a_app = {
        let a_game = model::GameState::new();
        let a_renderer = renderer::OGLRenderer::new(&a_game, &a_win_ctx.context(), 1, 1);
        let a_controls = controls::Controls::new();
        app::App::new(a_game, a_controls, a_renderer)
    };

    let mut a_time_last_upd = std::time::Instant::now();

    a_event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let a_controls = a_app.get_controls();

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => {
                // println!("{:?}", event);
                match event {
                    WindowEvent::Resized(the_size) => {
                        a_win_ctx.resize(the_size);
                        a_app
                            .get_renderer_mut()
                            .resize(the_size.width, the_size.height);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(_) => {
                let a_time_old = a_time_last_upd;
                a_time_last_upd = std::time::Instant::now();
                let a_delta = a_time_last_upd - a_time_old;

                a_app.tick(a_win_ctx.window(), a_delta);
                a_win_ctx.swap_buffers().unwrap();

                a_win_ctx.window().request_redraw();
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Key(the_input) => match the_input.state {
                    ElementState::Pressed => a_controls.key_pressed(the_input.scancode),
                    ElementState::Released => a_controls.key_released(the_input.scancode),
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                let a_time_frame_end = std::time::Instant::now();
                let a_frame_duration = a_time_frame_end - a_time_last_upd;
                let a_time_remaining =
                    constants::TARGET_TICK_TIME - (a_frame_duration.as_micros() as f32 / 1000.);
                if a_time_remaining > 0. {
                    std::thread::sleep(std::time::Duration::from_micros(
                        (a_time_remaining * 1000.) as u64,
                    ));
                }
            }
            _ => (),
        }
    });
}
