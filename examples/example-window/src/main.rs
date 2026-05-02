use firework_ui::ui;
use firework_ui::{AdapterClickPhase, AdapterCommand, AdapterEvent, AdapterResult};

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::cell::RefCell;

thread_local! {
    static LAST_MOUSE_POS: RefCell<(u32, u32)> = RefCell::new((0, 0));
    static MOUSE_BUTTON_DOWN: RefCell<bool> = RefCell::new(false);
}

#[ui]
fn test_screen() {
    let mut spark1 = spark!(0u32);
    let mut spark2 = spark!(0u32);

    spark1 += spark2;

    effect!(spark1, {
        println!("Update spark1: {}", spark1);
        spark2 = 10;
    });

    spark2 = 10;
}

fn my_adapter(command: AdapterCommand) -> AdapterResult {
    match command {
        AdapterCommand::RemoveAll => {
            println!("Remove all");
        }

        AdapterCommand::RunLoop {
            title,
            width,
            height,
            listener,
        } => {
            let event_loop = EventLoop::new();
            let window = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(LogicalSize::new(width, height))
                .build(&event_loop)
                .unwrap();

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            listener(AdapterEvent::CloseRequest);
                            *control_flow = ControlFlow::Exit;
                        }

                        WindowEvent::CursorMoved { position, .. } => {
                            let x = position.x as u32;
                            let y = position.y as u32;

                            LAST_MOUSE_POS.with(|pos| *pos.borrow_mut() = (x, y));
                        }

                        WindowEvent::MouseInput { state, button, .. }
                            if button == MouseButton::Left =>
                        {
                            let (x, y) = LAST_MOUSE_POS.with(|pos| *pos.borrow());
                            let phase = match state {
                                ElementState::Pressed => {
                                    MOUSE_BUTTON_DOWN.with(|down| *down.borrow_mut() = true);
                                    AdapterClickPhase::Began
                                }

                                ElementState::Released => {
                                    MOUSE_BUTTON_DOWN.with(|down| *down.borrow_mut() = false);
                                    AdapterClickPhase::Ended
                                }
                            };

                            listener(AdapterEvent::Touch(x, y, phase));
                        }

                        WindowEvent::ReceivedCharacter(ch) => {
                            listener(AdapterEvent::Key(ch as u32));
                        }

                        _ => (),
                    },

                    Event::MainEventsCleared => {
                        listener(AdapterEvent::Tick);
                        window.request_redraw();
                    }

                    _ => (),
                }
            });
        }

        AdapterCommand::Render => {}

        _ => todo!(),
    }

    AdapterResult::Void
}

fn main() {
    firework_ui::run_with_adapter(my_adapter, test_screen);
}
