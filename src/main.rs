use std::time::Instant;

use glium::glutin;
use glium::{Display, Rect};
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};

mod audio;
mod chip8;
mod cpu;
mod display;
mod input;
mod renderer;

use chip8::Chip8;
use renderer::Renderer;

const ASPECT_RATIO: f32 = 2.0 / 1.0;

const CYCLE_HZ: u32 = 720;
const TICK_HZ: u32 = 60;

const CYCLE_TIME: f32 = 1.0 / CYCLE_HZ as f32;
const TICK_TIME: f32 = 1.0 / TICK_HZ as f32;

// const PROGRAM: &[u8] = include_bytes!("../roms/games/Lunar Lander (Udo Pernisz, 1979).ch8");
// const PROGRAM: &[u8] = include_bytes!("../roms/games/Tetris [Fran Dachille, 1991].ch8");
// const PROGRAM: &[u8] = include_bytes!("../roms/glitchGhost.ch8");
const PROGRAM: &[u8] = include_bytes!("../roms/games/Pong [Paul Vervalin, 1990].ch8");

fn main() {
    let event_loop = EventLoop::new();
    let monitor = event_loop.primary_monitor();
    let window_builder = WindowBuilder::new()
        .with_visible(false)
        .with_title("chip8")
        .with_fullscreen(Some(Fullscreen::Borderless(monitor)));
    let context_builder = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true);
    let display = Display::new(window_builder, context_builder, &event_loop).unwrap();

    let mut chip8 = Chip8::new(PROGRAM);
    let mut renderer = Renderer::new(display);

    let mut prev_t = Instant::now();
    let mut tick_dt = 0.0;
    let mut cycle_dt = 0.0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => match (state, key) {
                    (ElementState::Pressed, VirtualKeyCode::Escape) => {
                        *control_flow = ControlFlow::Exit
                    }
                    (ElementState::Pressed, key) => {
                        if let Some(k) = keymap(key) {
                            chip8.key_pressed(k)
                        }
                    }
                    (ElementState::Released, key) => {
                        if let Some(k) = keymap(key) {
                            chip8.key_released(k)
                        }
                    }
                },
                WindowEvent::Resized(window_size) => {
                    let height = (ASPECT_RATIO.recip() * window_size.width as f32) as u32;
                    renderer.set_viewport(Rect {
                        left: 0,
                        bottom: (window_size.height - height) / 2,
                        width: window_size.width,
                        height: height,
                    });
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let now = Instant::now();
                let dt = (now - prev_t).as_secs_f32();
                prev_t = now;
                tick_dt += dt;
                cycle_dt += dt;

                while cycle_dt > CYCLE_TIME {
                    chip8.cycle();
                    cycle_dt -= CYCLE_TIME;
                }

                while tick_dt > TICK_TIME {
                    chip8.tick();
                    tick_dt -= TICK_TIME;
                }

                renderer.render(chip8.display());
            }
            _ => {}
        }
    });
}

fn keymap(key: VirtualKeyCode) -> Option<u8> {
    use VirtualKeyCode::*;
    [X, Key1, Key2, Key3, Q, W, E, A, S, D, Z, C, Key4, R, F, V]
        .iter()
        .position(|&k| k == key)
        .map(|i| i as u8)
}
