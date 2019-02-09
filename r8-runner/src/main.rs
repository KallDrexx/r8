#[macro_use] extern crate clap;
extern crate sfml;
extern crate r8_core;

mod rendering;
mod settings;
mod roms;

use std::time::{Duration, Instant};
use sfml::window::{Event, Style, Key};
use sfml::graphics::{RenderWindow, Font};

use r8_core::Hardware;
use crate::settings::Settings;
use crate::rendering::RenderState;

fn main() {
    let settings = Settings::from_cli_arguments();

    let mut hardware = Hardware::new();
    roms::load_from_file(&mut hardware, &settings).unwrap();

    let font = Font::from_file("cour.ttf").unwrap();
    let mut window = RenderWindow::new((800, 600), "R8 Runner - Chip 8", Style::CLOSE, &Default::default());

    println!("Starting paused: {}", settings.start_paused);
    println!("Instructions per second: {}", settings.instructions_per_second);
    println!("Frames per second: {}", settings.frames_per_second);

    let tick_hz = 100 as f32;
    let tick_micro = 1000 as f32 / tick_hz;
    let instruction_micro = 1000 as f32 / settings.instructions_per_second as f32;

    let render_tick_rate = Duration::from_micros((1000 as f32 / settings.frames_per_second as f32) as u64);
    let timer_tick_rate = Duration::from_micros((1000 as f32 / 60 as f32) as u64);
    let instruction_tick_rate = Duration::from_micros(tick_micro as u64);
    let instructions_per_tick = (tick_micro / instruction_micro).round() as u32;

    let mut is_paused = settings.start_paused;
    let mut render_state = RenderState::new();
    let mut last_rendered_at = Instant::now();
    let mut last_timer_tick_at = Instant::now();
    let mut last_instruction_at = Instant::now();

    let mut history_stack = Vec::with_capacity(10);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed {code, alt: _, ctrl: _, shift: _, system: _} => handle_key_pressed(&mut hardware, code),
                Event::KeyReleased {code, alt: _, ctrl: _, shift: _, system: _} => {
                    if !handle_key_released(&mut hardware, code) {
                        // Unmapped key was pressed, so see if this is a non-chip8 key
                        if code == Key::Space {
                            is_paused = !is_paused;

                            if !is_paused {
                                history_stack.clear();
                            }
                        } else if code == Key::Return && is_paused {
                            history_stack.push(hardware.clone());

                            // Since we are paused, enter being pressed means execute one instruction
                            execute_next_instruction(&mut hardware);

                            hardware.simulate_timer_tick(); // Since we are paused, a step should simulate a frame tick
                        } else if code == Key::BackSpace && is_paused && history_stack.len() > 0 {
                            hardware = history_stack.pop().unwrap();
                        }
                    }
                }
                _ => (),
            }
        }


        if !is_paused && last_instruction_at.elapsed() >= instruction_tick_rate {
            for _ in 0..instructions_per_tick {
                execute_next_instruction(&mut hardware);
            }

            last_instruction_at = Instant::now();
        }

        if last_timer_tick_at.elapsed() >= timer_tick_rate{
            hardware.simulate_timer_tick();
            last_timer_tick_at = Instant::now();
        }

        if last_rendered_at.elapsed() >= render_tick_rate {
            render_state = rendering::render(&mut window, &mut hardware, &font, render_state);
            last_rendered_at = Instant::now();
        }
    }
}

fn execute_next_instruction(hardware: &mut Hardware) {
    let (byte1, byte2) = hardware.get_current_instruction_bytes();
    let instruction = r8_core::get_instruction(byte1, byte2);
    r8_core::execute_instruction(instruction, hardware).unwrap();
}

fn handle_key_pressed(hardware: &mut Hardware, key: Key) {
    match get_key_value(key) {
        Some(x) => hardware.current_key_down = Some(x),
        None => (),
    }
}

fn handle_key_released(hardware: &mut Hardware, key: Key) -> bool {
    match get_key_value(key) {
        Some(x) => {
            if hardware.current_key_down == Some(x) {
                hardware.current_key_down = None;
                hardware.key_released_since_last_instruction = Some(x);
            }

            true // valid key was pressed
        },

        None => false, // unmapped key was pressed
    }
}

fn get_key_value(key: Key) -> Option<u8> {
    match key {
        Key::Num1 => Some(1),
        Key::Num2 => Some(2),
        Key::Num3 => Some(3),
        Key::Q => Some(4),
        Key::W => Some(5),
        Key::E => Some(6),
        Key::A => Some(7),
        Key::S => Some(8),
        Key::D => Some(9),
        Key::Z => Some(0xa),
        Key::X => Some(0),
        Key::C => Some(0xb),
        Key::Num4 => Some(0xc),
        Key::R => Some(0xd),
        Key::F => Some(0xe),
        Key::V => Some(0xf),
        _ => None,
    }
}