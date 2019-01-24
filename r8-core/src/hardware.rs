use std::collections::HashMap;

const FONT_MEMORY_START_ADDRESS: u16 = 0x0;
pub const STACK_SIZE: usize = 16;
pub const MEMORY_SIZE: usize = 0xFFF;
pub const FRAMEBUFFER_WIDTH: usize = 64;
pub const FRAMEBUFFER_HEIGHT: usize = 32;

pub struct Hardware {
    pub memory: [u8; MEMORY_SIZE],
    pub gen_registers: [u8; 16],
    pub i_register: u16,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub program_counter: u16,
    pub stack: [u16; STACK_SIZE],
    pub stack_pointer: usize,
    pub current_key_down: Option<u8>,
    pub key_released_since_last_instruction: Option<u8>,
    pub framebuffer: [[u8; FRAMEBUFFER_WIDTH / 8]; FRAMEBUFFER_HEIGHT], // up to 64x32 pixel resolution,
    pub font_addresses: HashMap<u8, u16>,
}

impl Hardware {
    pub fn new() -> Hardware {
        let mut hardware = Hardware {
            memory: [0_u8; MEMORY_SIZE],
            gen_registers: [0_u8; 16],
            i_register: 0,
            sound_timer: 0,
            delay_timer: 0,
            program_counter: 512, // First accessible memory location
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            current_key_down: None,
            key_released_since_last_instruction: None,
            framebuffer: [[0; FRAMEBUFFER_WIDTH / 8]; FRAMEBUFFER_HEIGHT],
            font_addresses: HashMap::new(),
        };

        hardware.load_fonts();
        hardware
    }

    pub fn simulate_timer_tick(&mut self) {
        self.delay_timer = if self.delay_timer > 0 { self.delay_timer - 1 } else { 0 };
        self.sound_timer = if self.sound_timer > 0 { self.sound_timer - 1 } else { 0 };
    }

    fn load_fonts(&mut self) {
        let zero = [0xf0, 0x90, 0x90, 0x90, 0xf0];
        let one = [0x20, 0x60, 0x20, 0x20, 0x70];
        let two = [0xf0, 0x10, 0xf0, 0x80, 0xf0];
        let three = [0xf0, 0x10, 0xf0, 0x10, 0xf0];
        let four = [0x90, 0x90, 0xf0, 0x10, 0x10];
        let five = [0xf0, 0x80, 0xf0, 0x10, 0xf0];
        let six = [0xf0, 0x80, 0xf0, 0x90, 0xf0];
        let seven = [0xf0, 0x10, 0x20, 0x40, 0x40];
        let eight = [0xf0, 0x90, 0xf0, 0x90, 0xf0];
        let nine = [0xf0, 0x90, 0xf0, 0x10, 0xf0];
        let a = [0xf0, 0x90, 0xf0, 0x90, 0x90];
        let b = [0xe0, 0x90, 0xe0, 0x90, 0xe0];
        let c = [0xf0, 0x80, 0x80, 0x80, 0xf0];
        let d = [0xe0, 0x90, 0x90, 0x90, 0xe0];
        let e = [0xf0, 0x80, 0xf0, 0x80, 0xf0];
        let f = [0xf0, 0x80, 0xf0, 0x80, 0x80];

        let fonts = [
            zero, one, two, three, four, five, six, seven, eight, nine, a, b, c, d, e, f
        ];

        let mut current_address = FONT_MEMORY_START_ADDRESS;
        for x in 0..fonts.len() {
            self.font_addresses.insert(x as u8, current_address);
            for y in 0..fonts[x].len() {
                self.memory[current_address as usize] = fonts[x][y];
                current_address = current_address + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_font_0_loaded_at_hardware_creation() {
        let hardware = Hardware::new();        
        let start_address = hardware.font_addresses[&0x0] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0x90, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x90, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_1_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x1] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0x20, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x60, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0x20, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x20, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0x70, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_2_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x2] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x10, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x80, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_3_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x3] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x10, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x10, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_4_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x4] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0x90, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x10, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0x10, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_5_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x5] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x80, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x10, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_6_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x6] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x80, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x90, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_7_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x7] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x10, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0x20, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x40, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0x40, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_8_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x8] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x90, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_9_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0x9] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x10, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_a_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0xa] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x90, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0x90, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_b_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0xb] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xe0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xe0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x90, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xe0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_c_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0xc] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x80, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0x80, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x80, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_d_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0xd] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xe0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x90, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0x90, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x90, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xe0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_e_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0xe] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x80, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x80, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0xf0, "Incorrect byte 5 value");
    }

    #[test]
    fn sprite_font_f_loaded_at_hardware_creation() {
        let hardware = Hardware::new();
        let start_address = hardware.font_addresses[&0xf] as usize;
        assert_eq!(hardware.memory[start_address + 0], 0xf0, "Incorrect byte 1 value");
        assert_eq!(hardware.memory[start_address + 1], 0x80, "Incorrect byte 2 value");
        assert_eq!(hardware.memory[start_address + 2], 0xf0, "Incorrect byte 3 value");
        assert_eq!(hardware.memory[start_address + 3], 0x80, "Incorrect byte 4 value");
        assert_eq!(hardware.memory[start_address + 4], 0x80, "Incorrect byte 5 value");
    }
}