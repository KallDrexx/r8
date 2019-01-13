pub struct Hardware {
    pub memory: [u8; 4096],
    pub gen_registers: [u8; 16],
    pub i_register: u16,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub program_counter: u16,
    pub stack: [u16; 16],
    pub stack_pointer: u8,
}

impl Hardware {
    pub fn new() -> Hardware {
        Hardware {
            memory: [0_u8; 4096],
            gen_registers: [0_u8; 16],
            i_register: 0,
            sound_timer: 0,
            delay_timer: 0,
            program_counter: 512, // First accessible memory location
            stack: [0; 16],
            stack_pointer: 0,
        }
    }
}