use r8_core::{Hardware, Instruction};

use sfml::system::Vector2f;
use sfml::graphics::{RenderWindow, Color, RenderTarget, Font, Text};
use sfml::graphics::{RectangleShape, Shape, Transformable, Image, Sprite, Texture};

const PLAY_AREA_SCALING_FACTOR: u32 = 8;
const PLAY_AREA_THICKNESS: u32 = 5;
const PLAY_AREA_START_X: u32 = 0 + PLAY_AREA_THICKNESS;
const PLAY_AREA_START_Y: u32 = 0 + PLAY_AREA_THICKNESS;
const PIXEL_ON_COLOR: &Color = &Color {r: 68, g: 68, b: 41, a: 255};
const PIXEL_OFF_COLOR: &Color = &Color {r: 115, g: 130, b: 92, a: 255};
const REGISTER_START_Y: u32 = 300;
const ADDRESS_SPACE_BORDER_THICKNESS: u32 = 5;
const ADDRESS_SPACE_START_X: u32 = 550 + ADDRESS_SPACE_BORDER_THICKNESS;
const ADDRESS_SPACE_START_Y: u32 = 30 + ADDRESS_SPACE_BORDER_THICKNESS;
const ADDRESS_DISPLAY_COUNT: u32 = 22;
const SPRITE_DISPLAY_START_X: u32 = 400;
const SPRITE_DISPLAY_START_Y: u32 = 300;

pub struct RenderState {
    lowest_visible_address: u16,
    highest_visible_address: u16,
}

impl RenderState {
    pub fn new() -> Self {
        RenderState {
            lowest_visible_address: 512,
            highest_visible_address: 512 + ADDRESS_DISPLAY_COUNT as u16,
        }
    }
}

pub fn render(window: &mut RenderWindow, hardware: &Hardware, font: &Font, mut last_render_state: RenderState, is_paused: bool) -> RenderState {
    window.set_active(true);
    window.clear(&Color::BLACK);

    render_framebuffer(window, &hardware);
    render_registers(window, &hardware, font);
    render_assembly_display(window, hardware, font, &mut last_render_state, is_paused);
    render_next_sprite_display(window, hardware, font);

    window.display();

    last_render_state
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
    shape.set_outline_thickness(PLAY_AREA_THICKNESS as f32);
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
                        let color = if pixel_on { PIXEL_ON_COLOR } else { PIXEL_OFF_COLOR };
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

fn render_registers(window: &mut RenderWindow, hardware: &Hardware, font: &Font) {
    let mut current_x = 0;
    let mut current_y = REGISTER_START_Y;
    for gen_reg in 0..hardware.gen_registers.len() {
        let str = format!("V{:x}: {:0>2x}", gen_reg, hardware.gen_registers[gen_reg]);
        render_register_value(window, font, str, &mut current_x, &mut current_y);
    }

    let dt_string = format!("DT: {:0>2x}", hardware.delay_timer);
    let st_string = format!("ST: {:0>2x}", hardware.sound_timer);
    let i_string = format!("I : {:0>3x}", hardware.i_register);

    let key_down_code = if let Some(x) = hardware.current_key_down {format!("{:x}", x)} else { "".to_owned() };
    let input_string = format!("Key: {}", key_down_code);

    render_register_value(window, font, dt_string, &mut current_x, &mut current_y);
    render_register_value(window, font, st_string, &mut current_x, &mut current_y);
    render_register_value(window, font, i_string, &mut current_x, &mut current_y);
    render_register_value(window, font, input_string, &mut current_x, &mut current_y);
}

fn render_register_value(window: &mut RenderWindow, font: &Font, display: String, current_x: &mut u32, current_y: &mut u32) {
    draw_text(window, font, 25, display.as_ref(), *current_x as f32, *current_y as f32);

    *current_y += 30;
    if *current_y + 25 > 600 {
        *current_y = REGISTER_START_Y;
        *current_x += 125;
    }
}

fn render_assembly_display(window: &mut RenderWindow, hardware: &Hardware, font: &Font, render_state: &mut RenderState, is_paused: bool) {
    const FONT_SIZE: u32 = 20;
    const FONT_SPACING: f32 = 5.0;
    const MIN_HIGH_ADDRESS_BUFFER: u16 = 3;

    if is_paused {
        let mut text = Text::new("DEBUGGING", &font, FONT_SIZE);
        text.set_fill_color(&Color::RED);

        let text_x = ADDRESS_SPACE_START_X as f32;
        text.set_position(Vector2f::new(text_x, 0.0));
        window.draw(&text);
    }

    let first_memory_address = if render_state.lowest_visible_address % 2 != hardware.program_counter % 2 {
        // We changed even vs odd, so reset boundaries
        hardware.program_counter
    } else if render_state.lowest_visible_address > hardware.program_counter {
        // new location is outside the previous boundary
        hardware.program_counter
    } else if render_state.highest_visible_address < hardware.program_counter {
        // new location is outside the previous boundary
        hardware.program_counter
    } else if render_state.highest_visible_address - hardware.program_counter < MIN_HIGH_ADDRESS_BUFFER * 2 {
        // Make sure we always have a buffer between the current instruction and the next
        let gap = (render_state.highest_visible_address - hardware.program_counter) / 2;
        render_state.lowest_visible_address + (gap * 2)
    } else {
        // We are still in range, so keep the same range
        render_state.lowest_visible_address
    };

    let width = window.size().x - ADDRESS_SPACE_START_X - ADDRESS_SPACE_BORDER_THICKNESS;
    for x in 0..ADDRESS_DISPLAY_COUNT {
        let address = first_memory_address as usize + (x * 2) as usize;

        let byte1 = hardware.memory[address];
        let byte2 = hardware.memory[address + 1];
        let instruction = r8_core::get_instruction(byte1, byte2);
        let display = format!("{:0>3x}: {}", address, instruction);

        let mut text = Text::new(display.as_ref(), &font, FONT_SIZE);
        let text_x = ADDRESS_SPACE_START_X as f32;
        let text_y = (x as f32 * (FONT_SIZE as f32 + FONT_SPACING)) + ADDRESS_SPACE_START_Y as f32;
        text.set_position(Vector2f::new(text_x, text_y));

        if address as u16 == hardware.program_counter {
            let mut highlight = RectangleShape::new();
            highlight.set_size(Vector2f::new(width as f32, FONT_SIZE as f32 + (FONT_SPACING / 2.0)));
            highlight.set_position(Vector2f::new(text_x, text_y));
            highlight.set_fill_color(&Color::CYAN);
            window.draw(&highlight);

            text.set_fill_color(&Color::BLACK);
        }

        window.draw(&text);
    }

    render_state.lowest_visible_address = first_memory_address;
    render_state.highest_visible_address = first_memory_address + (ADDRESS_DISPLAY_COUNT as u16 * 2);
}

fn render_next_sprite_display(window: &mut RenderWindow, hardware: &Hardware, font: &Font) {
    const FONT_SIZE: u32 = 17;
    const Y_SPACING: f32 = 0.0;

    let (byte1, byte2) = hardware.get_current_instruction_bytes();
    if let Instruction::DrawSprite {x_register: _, y_register: _, height} = r8_core::get_instruction(byte1, byte2) {
        let start_memory_address = hardware.i_register;
        let mut current_y = SPRITE_DISPLAY_START_Y as f32;

        draw_text(window, font, FONT_SIZE, "Next Sprite:", SPRITE_DISPLAY_START_X as f32, current_y);
        current_y += Y_SPACING + FONT_SIZE as f32;

        for x in 0..height {
            let value = hardware.memory[start_memory_address as usize + x as usize];
            let display = format!("{:0>2x}: {:0>8b}", x, value);
            draw_text(window, font, FONT_SIZE, display.as_ref(), SPRITE_DISPLAY_START_X as f32, current_y);
            current_y += Y_SPACING + FONT_SIZE as f32;
        }
    }
}

fn draw_text(window: &mut RenderWindow, font: &Font, font_size: u32, string: &str, x: f32, y: f32) {
    let mut text = Text::new(string, &font, font_size);
    text.set_position(Vector2f::new(x, y));
    window.draw(&text);
}