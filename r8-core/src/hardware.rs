pub const STACK_SIZE: usize = 16;
pub const MEMORY_SIZE: usize = 0xFFF;

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
}

impl Hardware {
    pub fn new() -> Hardware {
        Hardware {
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
        }
    }
}