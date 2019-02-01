#[macro_use] extern crate clap;
extern crate sfml;
extern crate r8_core;

mod rendering;
mod settings;
mod roms;

use std::{cmp, thread};
use std::time::{Duration, Instant};
use sfml::window::{Event, Style, Key};
use sfml::graphics::{RenderWindow, Font};

use r8_core::Hardware;
use crate::settings::Settings;

fn main() {
    let settings = Settings::from_cli_arguments();

    let mut hardware = Hardware::new();
    roms::load_from_file(&mut hardware, &settings).unwrap();

    let font = Font::from_file("cour.ttf").unwrap();
    let mut window = RenderWindow::new((800, 600), "R8 Runner - Chip 8", Style::CLOSE, &Default::default());
    //window.set_vertical_sync_enabled(true);

    println!("Starting paused: {}", settings.start_paused);
    println!("Instructions per second: {}", settings.instructions_per_second);
    println!("Frames per second: {}", settings.frames_per_second);

    let micros_between_instructions = ((1.0 / settings.instructions_per_second as f32) * 1000000.0) as u32;
    let micro_between_renders = ((1.0 / settings.frames_per_second as f32) * 1000000.0) as u32;
    let micro_between_timer_tick = ((1.0 / 60.0) * 1000000.0) as u32;

    let mut is_paused = settings.start_paused;
    let mut last_instruction_executed_at = Instant::now();
    let mut last_rendered_at = Instant::now();
    let mut last_timer_tick_at = Instant::now();

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

                            // If we are no longer paused, make sure we reset the instruction timer
                            last_instruction_executed_at = Instant::now();
                        }
                        else if code == Key::Return && is_paused {
                            // Since we are paused, enter being pressed means execute one instruction
                            execute_next_instruction(&mut hardware);

                            hardware.simulate_timer_tick(); // Since we are paused, a step should simulate a frame tick
                        }
                    }
                }
                _ => (),
            }
        }

        let should_run_instruction = time_expired(&last_instruction_executed_at, micros_between_instructions);
        if !is_paused && should_run_instruction {
            execute_next_instruction(&mut hardware);
            last_instruction_executed_at = Instant::now();
        }

        let should_tick = time_expired(&last_timer_tick_at, micro_between_timer_tick);
        if should_tick {
            hardware.simulate_timer_tick();
            last_timer_tick_at = Instant::now();
        }

        let should_render = time_expired(&last_rendered_at, micro_between_renders);
        if should_render {
            rendering::render(&mut window, &mut hardware, &font);
            last_rendered_at = Instant::now();
        }

        let now = Instant::now();
        let micro_till_next_render = (now - last_rendered_at).subsec_micros();
        let micro_till_next_instruction = (now - last_instruction_executed_at).subsec_micros();
        let micro_till_next_timer_tick = (now - last_instruction_executed_at).subsec_micros();
        let sleep_time = cmp::min(micro_till_next_instruction, cmp::min(micro_till_next_timer_tick, micro_till_next_render));
        thread::sleep(Duration::from_micros(sleep_time as u64));
    }
}

fn time_expired(start_time: &Instant, duration_in_microseconds: u32) -> bool {
    let elapsed = start_time.elapsed();

    // Shouldn't have overflow issues unless a time is really off
    let elapsed_microseconds = elapsed.as_secs() * 1000000 + elapsed.subsec_micros() as u64;

    return elapsed_microseconds >= duration_in_microseconds as u64;
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