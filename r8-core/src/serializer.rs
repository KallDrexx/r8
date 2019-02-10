use custom_error::custom_error;
use crate::{Instruction, Register};

custom_error! {pub SerializationError
    UnserializableInstruction {instruction: Instruction} = "No known way to serialize instruction {instruction}",
    InvalidSubtractionStoredIn {instruction: Instruction} = "Subtraction requested with invalid storage register: {instruction}",
}

pub fn serialize_instruction(instruction: Instruction) -> Result<(u8, u8), SerializationError> {
    match instruction {
        Instruction::AddFromRegister { register1: Register::General(reg1_num), register2: Register::General(reg2_num) } => {
            Ok((0x80 + reg1_num, (reg2_num << 4) + 0x4))
        }

        Instruction::AddFromRegister { register1: Register::I, register2: Register::General(reg_num)} => {
            Ok((0xf0 + reg_num, 0x1e))
        }

        Instruction::AddFromValue { register: Register::General(reg_num), value } => {
            Ok((0x70 + reg_num, value))
        }

        Instruction::And { register1: Register::General(reg1_num), register2: Register::General(reg2_num) } => {
            Ok((0x80 + reg1_num, (reg2_num << 4) + 0x2))
        }

        Instruction::Call { address } => {
            Ok((0x20 + (address >> 8) as u8, address as u8))
        }

        Instruction::ClearDisplay => {
            Ok((0x00, 0xe0))
        }

        Instruction::DrawSprite { x_register: Register::General(x_reg), y_register: Register::General(y_reg), height } => {
            Ok((0xd0 + x_reg, (y_reg << 4) + height))
        }

        Instruction::JumpToAddress { address, add_register_0 } => {
            match add_register_0 {
                true => Ok((0xb0 + (address >> 8) as u8, address as u8)),
                false => Ok((0x10 + (address >> 8) as u8, address as u8)),
            }
        }

        Instruction::JumpToMachineCode { address } => {
            Ok(((address >> 8) as u8, address as u8))
        }

        Instruction::LoadAddressIntoIRegister { address } => {
            Ok((0xa0 + (address >> 8) as u8, address as u8))
        }

        Instruction::LoadBcdValue { source: Register::General(reg_num) } => {
            Ok((0xf0 + reg_num, 0x33))
        }

        Instruction::LoadFromKeyPress { destination: Register::General(reg_num) } => {
            Ok((0xf0 + reg_num, 0x0a))
        }

        Instruction::LoadFromMemory { last_register: Register::General(reg_num) } => {
            Ok((0xf0 + reg_num, 0x65))
        }

        Instruction::LoadFromRegister { source: Register::DelayTimer, destination: Register::General(dest_num) } => {
            Ok((0xf0 + dest_num, 0x07))
        }

        Instruction::LoadFromRegister { source: Register::General(source_num), destination: Register::DelayTimer} => {
            Ok((0xf0 + source_num , 0x15))
        }

        Instruction::LoadFromRegister { source: Register::General(source_num), destination: Register::SoundTimer } => {
            Ok((0xf0 + source_num, 0x18))
        }

        Instruction::LoadFromRegister { source: Register::General(source_num), destination: Register::General(dest_num) } => {
            Ok((0x80 + dest_num, source_num << 4))
        }

        Instruction::LoadFromValue { destination: Register::General(reg_num), value } => {
            Ok((0x60 + reg_num, value))
        }

        Instruction::LoadIntoMemory { last_register: Register::General(reg_num) } => {
            Ok((0xf0 + reg_num, 0x55))
        }

        Instruction::LoadSpriteLocation { sprite_digit: Register::General(reg_num) } => {
            Ok((0xf0 + reg_num, 0x29))
        }

        Instruction::Or { register1: Register::General(reg1_num), register2: Register::General(reg2_num) } => {
            Ok((0x80 + reg1_num, (reg2_num << 4) + 0x1))
        }

        Instruction::Return => {
            Ok((0x00, 0xee))
        }

        Instruction::SetRandom { register: Register::General(reg_num), and_value } => {
            Ok((0xc0 + reg_num, and_value))
        }

        Instruction::ShiftLeft { register: Register::General(reg_num)} => {
            Ok((0x80 + reg_num, 0x0e))
        }

        Instruction::ShiftRight { register: Register::General(reg_num)} => {
            Ok((0x80 + reg_num, 0x06))
        }

        Instruction::SkipIfEqual { register: Register::General(reg_num), value} => {
            Ok((0x30 + reg_num, value))
        }

        Instruction::SkipIfKeyNotPressed { register: Register::General(reg_num) } => {
            Ok((0xe0 + reg_num, 0xa1))
        }

        Instruction::SkipIfKeyPressed { register: Register::General(reg_num) } => {
            Ok((0xe0 + reg_num, 0x9e))
        }

        Instruction::SkipIfNotEqual { register: Register::General(reg_num), value} => {
            Ok((0x40 + reg_num, value))
        }

        Instruction::SkipIfRegistersEqual { register1: Register::General(reg1_num), register2: Register::General(reg2_num) } => {
            Ok((0x50 + reg1_num, reg2_num << 4))
        }

        Instruction::SkipIfRegistersNotEqual { register1: Register::General(reg1_num), register2: Register::General(reg2_num) } => {
            Ok((0x90 + reg1_num, reg2_num << 4))
        }

        Instruction::Subtract { minuend: Register::General(min_num), subtrahend: Register::General(sub_num), stored_in: Register::General(stored_num)} => {
            match stored_num {
                x if x == min_num => Ok((0x80 + min_num, (sub_num << 4) + 0x5)),
                x if x == sub_num => Ok((0x80 + sub_num, (min_num << 4) + 0x7)),
                _ => Err(SerializationError::InvalidSubtractionStoredIn{instruction: Instruction::Subtract { minuend: Register::General(min_num), subtrahend: Register::General(sub_num), stored_in: Register::General(stored_num)}}),
            }
        }

        Instruction::Unknown { bytes } => {
            Ok(((bytes >> 8) as u8, bytes as u8))
        }

        Instruction::Xor { register1: Register::General(reg1_num), register2: Register::General(reg2_num)} => {
            Ok((0x80 + reg1_num, (reg2_num << 4) + 0x3))
        }

        instruction => Err(SerializationError::UnserializableInstruction {instruction})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Register;

    #[test]
    fn can_serialize_unknown_instruction() {
        let instruction = Instruction::Unknown {bytes: 0x1234};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x12, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x34, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_jump_to_machine_instruction() {
        let instruction = Instruction::JumpToMachineCode {address: 0x123};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x01, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x23, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_clear_screen_instruction() {
        let instruction = Instruction::ClearDisplay;
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x00, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xe0, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_return_instruction() {
        let instruction = Instruction::Return;
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x00, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xee, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_jump_to_address_without_add_instruction() {
        let instruction = Instruction::JumpToAddress {address: 0x345, add_register_0: false};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x13, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x45, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_call_instruction() {
        let instruction = Instruction::Call {address: 0x345};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x23, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x45, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_skip_if_equal_instruction() {
        let instruction = Instruction::SkipIfEqual {register: Register::General(3), value: 0xab};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x33, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xab, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_skip_if_not_equal_instruction() {
        let instruction = Instruction::SkipIfNotEqual {register: Register::General(3), value: 0xab};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x43, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xab, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_skip_if_registers_equal_instruction() {
        let instruction = Instruction::SkipIfRegistersEqual {register1: Register::General(3), register2: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x53, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x40, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_from_byte_instruction() {
        let instruction = Instruction::LoadFromValue {destination: Register::General(4), value: 0x8f};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x64, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x8f, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_add_from_value_instruction() {
        let instruction = Instruction::AddFromValue {register: Register::General(3), value: 0xab};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x73, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xab, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_from_register_instruction() {
        let instruction = Instruction::LoadFromRegister {source: Register::General(3), destination: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x84, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x30, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_or_instruction() {
        let instruction = Instruction::Or {register1: Register::General(3), register2: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x41, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_and_instruction() {
        let instruction = Instruction::And {register1: Register::General(3), register2: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x42, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_xor_instruction() {
        let instruction = Instruction::Xor {register1: Register::General(3), register2: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x43, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_add_from_register_instruction() {
        let instruction = Instruction::AddFromRegister {register1: Register::General(3), register2: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x44, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_subtract_stored_in_minuend_instruction() {
        let instruction = Instruction::Subtract {minuend: Register::General(3), subtrahend: Register::General(4), stored_in: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x45, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_shift_right_instruction() {
        let instruction = Instruction::ShiftRight {register: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x06, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_subtract_stored_in_subtrahend_instruction() {
        let instruction = Instruction::Subtract {minuend: Register::General(3), subtrahend: Register::General(4), stored_in: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x84, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x37, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_shift_left_instruction() {
        let instruction = Instruction::ShiftLeft {register: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x83, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x0e, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_skip_if_registers_not_equal_instruction() {
        let instruction = Instruction::SkipIfRegistersNotEqual {register1: Register::General(3), register2: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0x93, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x40, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_address_into_i_register() {
        let instruction = Instruction::LoadAddressIntoIRegister {address: 0x123};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xa1, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x23, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_jump_with_add_instruction() {
        let instruction = Instruction::JumpToAddress {address: 0x123, add_register_0: true};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xb1, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x23, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_get_random_instruction() {
        let instruction = Instruction::SetRandom {register: Register::General(5), and_value: 0xef};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xc5, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xef, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_draw_instruction() {
        let instruction = Instruction::DrawSprite {x_register: Register::General(3), y_register: Register::General(4), height: 0xf};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xd3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x4f, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_skip_if_key_pressed_instruction() {
        let instruction = Instruction::SkipIfKeyPressed {register: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xe4, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x9e, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_skip_if_key_not_pressed_instruction() {
        let instruction = Instruction::SkipIfKeyNotPressed {register: Register::General(4)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xe4, "Incorrect byte 1 value");
        assert_eq!(byte2, 0xa1, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_from_delay_timer_instruction() {
        let instruction = Instruction::LoadFromRegister {source: Register::DelayTimer, destination: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x07, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_wait_for_keypress_instruction() {
        let instruction = Instruction::LoadFromKeyPress {destination: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x0a, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_into_delay_timer_instruction() {
        let instruction = Instruction::LoadFromRegister {destination: Register::DelayTimer, source: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x15, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_into_sound_timer_instruction() {
        let instruction = Instruction::LoadFromRegister {source: Register::General(4), destination: Register::SoundTimer};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf4, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x18, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_add_to_i_register() {
        let instruction = Instruction::AddFromRegister {register1: Register::I, register2: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x1e, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_sprite_digit_instruction() {
        let instruction = Instruction::LoadSpriteLocation {sprite_digit: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x29, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_store_bcd_value_instruction() {
        let instruction = Instruction::LoadBcdValue {source: Register::General(3)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf3, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x33, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_into_memory_instruction() {
        let instruction = Instruction::LoadIntoMemory {last_register: Register::General(5)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf5, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x55, "Incorrect byte 2 value");
    }

    #[test]
    fn can_serialize_load_from_memory_instruction() {
        let instruction = Instruction::LoadFromMemory {last_register: Register::General(5)};
        let (byte1, byte2) = serialize_instruction(instruction).unwrap();

        assert_eq!(byte1, 0xf5, "Incorrect byte 1 value");
        assert_eq!(byte2, 0x65, "Incorrect byte 2 value");
    }
}