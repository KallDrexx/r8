mod hardware;
mod parser;

pub use hardware::Hardware;
pub use parser::get_instruction;

// Info sourced from http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#Fx33
#[derive(Eq, PartialEq, Debug)]
pub enum Register {
    General(u8),
    I,
    SoundTimer,
    DelayTimer,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Instruction {
    Unknown,
    AddFromValue { register: Register, value: u8 },
    AddFromRegister { register1: Register, register2: Register },
    Call { address: u16 },
    ClearDisplay,
    JumpToAddress { address: u16, add_register_0: bool },
    JumpToMachineCode { address: u16 },
    LoadFromValue { destination: Register, value: u8 },
    LoadFromRegister { destination: Register, source: Register },
    LoadFromKeyPress { destination: Register },
    LoadSpriteLocation { destination: Register, sprite_digit: Register },
    LoadBcdValue { source: Register },
    LoadIntoMemory { last_register: Register },
    LoadFromMemory { last_register: Register },
    LoadAddressIntoIRegister { address: u16 },
    Return,
    SkipIfEqual { register: Register, value: u8 },
    SkipIfNotEqual { register: Register, value: u8 },
    SkipIfRegistersEqual { register1: Register, register2: Register },
    SkipIfRegistersNotEqual { register1: Register, register2: Register },
    SkipIfKeyPressed { register: Register },
    SkipIfKeyNotPressed { register: Register },
    Subtract { register1: Register, register2: Register, stored_in: Register },
    Or { register1: Register, register2: Register },
    And { register1: Register, register2: Register },
    Xor { register1: Register, register2: Register },
    ShiftRight { register: Register },
    ShiftLeft { register: Register},
    SetRandom { register: Register, and_value: u8 },
    DrawSprite { x_register: Register, y_register: Register, height: u8 },
}