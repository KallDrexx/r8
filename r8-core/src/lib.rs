extern crate custom_error;
extern crate rand;

mod hardware;
mod parser;
mod execution;
mod serializer;

use std::fmt;

pub use hardware::Hardware;
pub use parser::get_instruction;
pub use execution::execute_instruction;
pub use serializer::serialize_instruction;

// Info sourced from http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#Fx33
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Register {
    General(u8),
    I,
    SoundTimer,
    DelayTimer,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Instruction {
    AddFromRegister { register1: Register, register2: Register },
    AddFromValue { register: Register, value: u8 },
    And { register1: Register, register2: Register },
    Call { address: u16 },
    ClearDisplay,
    DrawSprite { x_register: Register, y_register: Register, height: u8 },
    JumpToAddress { address: u16, add_register_0: bool },
    JumpToMachineCode { address: u16 },
    LoadAddressIntoIRegister { address: u16 },
    LoadBcdValue { source: Register },
    LoadFromKeyPress { destination: Register },
    LoadFromMemory { last_register: Register },
    LoadFromRegister { destination: Register, source: Register },
    LoadFromValue { destination: Register, value: u8 },
    LoadIntoMemory { last_register: Register },
    LoadSpriteLocation { sprite_digit: Register },
    Or { register1: Register, register2: Register },
    Return,
    SetRandom { register: Register, and_value: u8 },
    ShiftLeft { register: Register},
    ShiftRight { register: Register },
    SkipIfEqual { register: Register, value: u8 },
    SkipIfKeyNotPressed { register: Register },
    SkipIfKeyPressed { register: Register },
    SkipIfNotEqual { register: Register, value: u8 },
    SkipIfRegistersEqual { register1: Register, register2: Register },
    SkipIfRegistersNotEqual { register1: Register, register2: Register },
    Subtract { minuend: Register, subtrahend: Register, stored_in: Register },
    Unknown { bytes: u16 },
    Xor { register1: Register, register2: Register },
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register::General(num) => write!(f, "V{:x}", num),
            Register::I => write!(f, "I"),
            Register::SoundTimer => write!(f, "ST"),
            Register::DelayTimer => write!(f, "DT"),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Unknown {bytes} => write!(f, "UNK 0x{:0>4x}", bytes),
            Instruction::AddFromValue {register, value} => write!(f, "ADD {}, {:x}", register, value),
            Instruction::AddFromRegister {register1, register2} => write!(f, "ADD {}, {}", register1, register2),
            Instruction::Call {address} => write!(f, "CALL {:x}", address),
            Instruction::ClearDisplay => write!(f, "CLS"),
            Instruction::JumpToAddress {address, add_register_0} => match add_register_0 {
                true => write!(f, "JP v0, {:x}", address),
                false => write!(f, "JP {:x}", address)
            },

            Instruction::JumpToMachineCode {address} => write!(f, "SYS {:x}", address),
            Instruction::LoadFromValue {destination, value} => write!(f, "LD {}, {:x}", destination, value),
            Instruction::LoadFromRegister {destination, source} => write!(f, "LD {}, {}", destination, source),
            Instruction::LoadFromKeyPress {destination} => write!(f, "LD {}, K", destination),
            Instruction::LoadSpriteLocation {sprite_digit} => write!(f, "LD F, {}", sprite_digit),
            Instruction::LoadBcdValue {source} => write!(f, "LD B, {}", source),
            Instruction::LoadIntoMemory {last_register} => write!(f, "LD [I], {}", last_register),
            Instruction::LoadFromMemory {last_register} => write!(f, "LD {}, [I]", last_register),
            Instruction::LoadAddressIntoIRegister {address} => write!(f, "LD I, {:x}", address),
            Instruction::Return => write!(f, "RET"),
            Instruction::SkipIfEqual {register, value} => write!(f, "SE {}, {:x}", register, value),
            Instruction::SkipIfNotEqual {register, value} => write!(f, "SNE {}, {:x}", register, value),
            Instruction::SkipIfRegistersEqual {register1, register2} => write!(f, "SE {}, {}", register1, register2),
            Instruction::SkipIfRegistersNotEqual {register1, register2} => write!(f, "SNE {}, {}", register1, register2),
            Instruction::SkipIfKeyPressed {register} => write!(f, "SKP {}", register),
            Instruction::SkipIfKeyNotPressed {register} => write!(f, "SKNP {}", register),
            Instruction::Subtract {minuend, subtrahend, stored_in} => if stored_in == minuend {
                write!(f, "SUB {}, {}", minuend, subtrahend)
            } else {
                write!(f, "SUBN {}, {}", minuend, subtrahend)
            },

            Instruction::Or {register1, register2} => write!(f, "OR {}, {}", register1, register2),
            Instruction::And {register1, register2} => write!(f, "AND {}, {}", register1, register2),
            Instruction::Xor {register1, register2} => write!(f, "XOR {}, {}", register1, register2),
            Instruction::ShiftRight {register} => write!(f, "SHR {}", register),
            Instruction::ShiftLeft {register} => write!(f, "SHL {}", register),
            Instruction::SetRandom {register, and_value} => write!(f, "RND {}, {:x}", register, and_value),
            Instruction::DrawSprite {x_register, y_register, height} => write!(f, "DRW {}, {}, {:x}", x_register, y_register, height),
        }
    }
}