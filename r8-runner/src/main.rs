extern crate sfml;
extern crate r8_core;

use sfml::window::{Event, Style};
use sfml::system::Vector2f;
use sfml::graphics::{RenderWindow, Color, RenderTarget};
use sfml::graphics::{RectangleShape, Shape, Transformable, Image, Sprite, Texture};
use sfml::graphics::{Font, Text};

use r8_core::{Hardware, Instruction, Register};

const PLAY_AREA_SCALING_FACTOR: u32 = 8;
const PLAY_AREA_START_X: u32 = 50;
const PLAY_AREA_START_Y: u32 = 0;

fn main() {
    let mut hardware = Hardware::new();
    setup_hardware(&mut hardware);

    let mut window = RenderWindow::new((800, 600), "R8 Runner - Chip 8", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }

        window.set_active(true);
        window.clear(&Color::BLACK);
        render_framebuffer(&mut window, &hardware);
        render_registers(&mut window, &hardware);

        window.display();
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

fn render_framebuffer(window: &mut RenderWindow, hardware: &Hardware) {
    let width = hardware.framebuffer[0].len() as u32 * 8 * PLAY_AREA_SCALING_FACTOR; // * 8 to expand byte compaction
    let height = hardware.framebuffer.len() as u32 * PLAY_AREA_SCALING_FACTOR;

    // First display the play area and border
    let mut shape = RectangleShape::new();
    shape.set_size(Vector2f::new(width as f32, height as f32));
    shape.set_position(Vector2f::new(PLAY_AREA_START_X as f32, PLAY_AREA_START_Y as f32));
    shape.set_fill_color(&Color::BLACK);
    shape.set_outline_color(&Color::BLUE);
    shape.set_outline_thickness(5_f32);
    window.draw(&shape);

    // Now add the actual framebuffer
    let mut image = Image::new(width, height);
    let mut current_y = 0;
    let mut current_x = 0;
    for row in 0..hardware.framebuffer.len() {
        for column_set in 0..hardware.framebuffer[row].len() {
            const BIT_MASK: u8 = 0b0000001;
            let byte = hardware.framebuffer[row][column_set];
            for shift in 0..8 {
                let pixel_on = if (byte >> (7 - shift)) & BIT_MASK == 1 { true } else { false };
                for scale_y in 0..PLAY_AREA_SCALING_FACTOR {
                    for scale_x in 0..PLAY_AREA_SCALING_FACTOR {
                        let color = if pixel_on { &Color::GREEN } else { &Color::BLACK };
                        image.set_pixel(scale_x + current_x, scale_y + current_y, color);
                    }
                }

                current_x += PLAY_AREA_SCALING_FACTOR;
            }
        }

        current_y += PLAY_AREA_SCALING_FACTOR;
        current_x = 0;
    }

    let texture = Texture::from_image(&image).unwrap();
    let mut sprite = Sprite::new();
    sprite.set_texture(&texture, false);
    sprite.set_position(Vector2f::new(PLAY_AREA_START_X as f32, PLAY_AREA_START_Y as f32));
    window.draw(&sprite);
}

fn render_registers(window: &mut RenderWindow, hardware: &Hardware) {
    let font = Font::from_file("sansation.ttf").unwrap();

    const START_Y: u32 = 300;
    let mut current_x = 0;
    let mut current_y = START_Y;
    for gen_reg in 0..hardware.gen_registers.len() {
        let str = format!("V{:x}: 0x{:x}", gen_reg, hardware.gen_registers[gen_reg]);
        let mut text = Text::new(str.as_ref(), &font, 25);
        text.set_position(Vector2f::new(current_x as f32, current_y as f32));
        window.draw(&text);

        current_y += 30;

        if current_y + 25 > 600 {
            current_y = START_Y;
            current_x += 125;
        }
    }
}
