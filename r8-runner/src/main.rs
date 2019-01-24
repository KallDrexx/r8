extern crate sfml;
extern crate r8_core;

mod rendering;

use sfml::window::{Event, Style};
use sfml::graphics::{RenderWindow, Font};

use r8_core::{Hardware, Instruction, Register};

fn main() {
    let mut hardware = Hardware::new();
    setup_hardware(&mut hardware);

    let font = Font::from_file("cour.ttf").unwrap();
    let mut window = RenderWindow::new((800, 600), "R8 Runner - Chip 8", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }

        hardware.simulate_timer_tick();
        rendering::render(&mut window, &mut hardware, &font);
    }
}

fn setup_hardware(hardware: &mut Hardware) {
    draw_digit(hardware, 1, 0, 0);
    draw_digit(hardware, 2, 8, 0);
    draw_digit(hardware, 3, 16, 0);
    draw_digit(hardware, 4, 24, 0);
    draw_digit(hardware, 5, 32, 0);
    draw_digit(hardware, 6, 40, 0);
    draw_digit(hardware, 7, 48, 0);
    draw_digit(hardware, 8, 56, 0);
    draw_digit(hardware, 9, 0, 10);
    draw_digit(hardware, 0, 8, 10);
    draw_digit(hardware, 0xa, 16, 10);
    draw_digit(hardware, 0xb, 24, 10);
    draw_digit(hardware, 0xc, 32, 10);
    draw_digit(hardware, 0xd, 40, 10);
    draw_digit(hardware, 0xe, 48, 10);
    draw_digit(hardware, 0xf, 56, 10);

    // Set up some assembly in memory to test memory display
    hardware.memory[512] = 0x00; hardware.memory[513] = 0xe0; // cls
    hardware.memory[514] = 0x74; hardware.memory[515] = 0x1f; // add
    hardware.memory[516] = 0x63; hardware.memory[517] = 0x4a; // ld vx, byte
    hardware.memory[518] = 0x84; hardware.memory[519] = 0x30; // ld vx, vy
    hardware.memory[520] = 0x84; hardware.memory[521] = 0x32; // and
    hardware.memory[522] = 0x84; hardware.memory[523] = 0x35; // sub
    hardware.memory[524] = 0xb1; hardware.memory[525] = 0x23; // jp
    hardware.memory[526] = 0xd4; hardware.memory[527] = 0x39; // draw
    hardware.memory[528] = 0x00; hardware.memory[529] = 0xe0; // cls
    hardware.memory[530] = 0x74; hardware.memory[531] = 0x1f; // add
    hardware.memory[532] = 0x63; hardware.memory[533] = 0x4a; // ld vx, byte
    hardware.memory[534] = 0x84; hardware.memory[535] = 0x30; // ld vx, vy
    hardware.memory[536] = 0x84; hardware.memory[537] = 0x32; // and
    hardware.memory[538] = 0x84; hardware.memory[539] = 0x35; // sub
    hardware.memory[540] = 0xb1; hardware.memory[541] = 0x23; // jp
    hardware.memory[542] = 0xd4; hardware.memory[543] = 0x39; // draw

    hardware.program_counter = 518;
    hardware.i_register = 0x1F2;
    hardware.delay_timer = 0xFF;
    hardware.sound_timer = 0xFF;
}

fn draw_digit(hardware: &mut Hardware, digit: u8, start_x: u8, start_y: u8) {
    let load_digit_value = Instruction::LoadFromValue {destination: Register::General(0), value: digit};
    let load_digit = Instruction::LoadSpriteLocation {sprite_digit: Register::General(0)};
    let load_start_x = Instruction::LoadFromValue {destination: Register::General(1), value: start_x};
    let load_start_y = Instruction::LoadFromValue {destination: Register::General(2), value: start_y};
    let draw = Instruction::DrawSprite {x_register: Register::General(1), y_register: Register::General(2), height: 5};

    r8_core::execute_instruction(load_digit_value, hardware).unwrap();
    r8_core::execute_instruction(load_digit, hardware).unwrap();
    r8_core::execute_instruction(load_start_x, hardware).unwrap();
    r8_core::execute_instruction(load_start_y, hardware).unwrap();
    r8_core::execute_instruction(draw, hardware).unwrap();
}
