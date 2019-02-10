use super::{Instruction, Register};

pub fn get_instruction(byte1: u8, byte2: u8) -> Instruction {
    let value1 = byte1 >> 4;
    let value2 = byte1 & 0x0F;
    let value3 = byte2 >> 4;
    let value4 = byte2 & 0x0F;

    match (value1, value2, value3, value4) {
        (0x0, 0x0, 0xe, 0x0) => {
            Instruction::ClearDisplay
        },

        (0x0, 0x0, 0xe, 0xe) => {
            Instruction::Return
        },

        (0x0, _, _, _) => {
            let address: u16 = ((value2 as u16) * 16 * 16) + (byte2 as u16);
            Instruction::JumpToMachineCode {address}
        },

        (0x1, _, _, _) => {
            let address = ((value2 as u16) * 16 * 16) + (byte2 as u16);
            Instruction::JumpToAddress {address, add_register_0: false}
        },

        (0x2, _, _, _) => {
            let address = ((value2 as u16) * 16 * 16) + (byte2 as u16);
            Instruction::Call {address}
        },

        (0x3, _, _, _) => {
            Instruction::SkipIfEqual {
                register: Register::General(value2),
                value: byte2,
            }
        },

        (0x4, _, _, _) => {
            Instruction::SkipIfNotEqual {
                register: Register::General(value2),
                value: byte2,
            }
        },

        (0x5, _, _, _) => {
            Instruction::SkipIfRegistersEqual {
                register1: Register::General(value2),
                register2: Register::General(value3),
            }
        },

        (0x6, _, _, _) => {
            Instruction::LoadFromValue {
                destination: Register::General(value2),
                value: byte2,
            }
        },

        (0x7, _, _, _) => {
            Instruction::AddFromValue {
                register: Register::General(value2),
                value: byte2,
            }
        },

        (0x8, _, _, 0x0) => {
            Instruction::LoadFromRegister {
                source: Register::General(value3),
                destination: Register::General(value2),
            }
        },

        (0x8, _, _, 0x1) => {
            Instruction::Or {
                register1: Register::General(value2),
                register2: Register::General(value3),
            }
        },

        (0x8, _, _, 0x2) => {
            Instruction::And {
                register1: Register::General(value2),
                register2: Register::General(value3),
            }
        },

        (0x8, _, _, 0x3) => {
            Instruction::Xor {
                register1: Register::General(value2),
                register2: Register::General(value3),
            }
        },

        (0x8, _, _, 0x4) => {
            Instruction::AddFromRegister {
                register1: Register::General(value2),
                register2: Register::General(value3),
            }
        }

        (0x8, _, _, 0x5) => {
            Instruction::Subtract {
                minuend: Register::General(value2),
                subtrahend: Register::General(value3),
                stored_in: Register::General(value2),
            }
        },

        (0x8, _, _, 0x6) => {
            Instruction::ShiftRight {
                register: Register::General(value2)
            }
        },

        (0x8, _, _, 0x7) => {
            Instruction::Subtract {
                minuend: Register::General(value3),
                subtrahend: Register::General(value2),
                stored_in: Register::General(value2),
            }
        },

        (0x8, _, _, 0xe) => {
            Instruction::ShiftLeft {
                register: Register::General(value2),
            }
        },

        (0x9, _, _, 0x0) => {
            Instruction::SkipIfRegistersNotEqual {
                register1: Register::General(value2),
                register2: Register::General(value3),
            }
        },

        (0xa, _, _, _) => {
            let address: u16 = ((value2 as u16) * 16 * 16) + (byte2 as u16);
            Instruction::LoadAddressIntoIRegister { address }
        },

        (0xb, _, _, _) => {
            let address: u16 = ((value2 as u16) * 16 * 16) + (byte2 as u16);
            Instruction::JumpToAddress { address, add_register_0: true }
        },

        (0xc, _, _, _) => {
            Instruction::SetRandom {
                register: Register::General(value2),
                and_value: byte2,
            }
        },

        (0xd, _, _, _) => {
            Instruction::DrawSprite {
                x_register: Register::General(value2),
                y_register: Register::General(value3),
                height: value4,
            }
        },

        (0xe, _, 0x9, 0xe) => {
            Instruction::SkipIfKeyPressed {
                register: Register::General(value2)
            }
        },

        (0xe, _, 0xa, 0x1) => {
            Instruction::SkipIfKeyNotPressed {
                register: Register::General(value2)
            }
        },

        (0xf, _, 0x0, 0x7) => {
            Instruction::LoadFromRegister {
                source: Register::DelayTimer,
                destination: Register::General(value2),
            }
        },

        (0xf, _, 0x0, 0xa) => {
            Instruction::LoadFromKeyPress {
                destination: Register::General(value2),
            }
        },

        (0xf, _, 0x1, 0x5) => {
            Instruction::LoadFromRegister {
                source: Register::General(value2),
                destination: Register::DelayTimer,
            }
        },

        (0xf, _, 0x1, 0x8) => {
            Instruction::LoadFromRegister {
                source: Register::General(value2),
                destination: Register::SoundTimer,
            }
        },

        (0xf, _, 0x1, 0xe) => {
            Instruction::AddFromRegister {
                register1: Register::I,
                register2: Register::General(value2),
            }
        },

        (0xf, _, 0x2, 0x9) => {
            Instruction::LoadSpriteLocation {
                sprite_digit: Register::General(value2),
            }
        },

        (0xf, _, 0x3, 0x3) => {
            Instruction::LoadBcdValue {
                source: Register::General(value2),
            }
        },

        (0xf, _, 0x5, 0x5) => {
            Instruction::LoadIntoMemory {
                last_register: Register::General(value2),
            }
        },

        (0xf, _, 0x6, 0x5) => {
            Instruction::LoadFromMemory {
                last_register: Register::General(value2),
            }
        },

        _ => Instruction::Unknown {bytes: ((byte1 as u16) << 8) + byte2 as u16 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Register;

    #[test]
    fn can_read_sys_instruction() {
        let expected = Instruction::JumpToMachineCode {address: 291};
        let result = get_instruction(0x01, 0x23);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_clear_screen_instruction() {
        let expected = Instruction::ClearDisplay;
        let result = get_instruction(0x00, 0xe0);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_return_instruction() {
        let expected = Instruction::Return;
        let result = get_instruction(0x00, 0xee);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_jump_to_address_instruction_without_addition() {
        let expected = Instruction::JumpToAddress {address: 291, add_register_0: false};
        let result = get_instruction(0x11, 0x23);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_call_instruction() {
        let expected = Instruction::Call {address: 291};
        let result = get_instruction(0x21, 0x23);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_skip_if_equal_to_value_instruction() {
        let expected = Instruction::SkipIfEqual {
            register: Register::General(0xa),
            value: 200
        };

        let result = get_instruction(0x3a, 0xc8);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_skip_if_not_equal_to_value_instruction() {
        let expected = Instruction::SkipIfNotEqual {
            register: Register::General(0xa),
            value: 200
        };

        let result = get_instruction(0x4a, 0xc8);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_skip_if_registers_equal_instruction() {
        let expected = Instruction::SkipIfRegistersEqual {
            register1: Register::General(0xa),
            register2: Register::General(0xb),
        };

        let result = get_instruction(0x5a, 0xb0);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_from_value_instruction() {
        let expected = Instruction::LoadFromValue {
            destination: Register::General(0xb),
            value: 0x3f,
        };

        let result = get_instruction(0x6b, 0x3f);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_add_from_value_instruction() {
        let expected = Instruction::AddFromValue {
            register: Register::General(0xb),
            value: 0xcd,
        };

        let result = get_instruction(0x7b, 0xcd);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_from_register_instruction() {
        let expected = Instruction::LoadFromRegister {
            source: Register::General(0xc),
            destination: Register::General(0xb)
        };

        let result = get_instruction(0x8b, 0xc0);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_or_instruction() {
        let expected = Instruction::Or {
            register1: Register::General(0xb),
            register2: Register::General(0xc),
        };

        let result = get_instruction(0x8b, 0xc1);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_and_instruction() {
        let expected = Instruction::And {
            register1: Register::General(0xb),
            register2: Register::General(0xc)
        };

        let result = get_instruction(0x8b, 0xc2);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_xor_instruction() {
        let expected = Instruction::Xor {
            register1: Register::General(0xb),
            register2: Register::General(0xc)
        };

        let result = get_instruction(0x8b, 0xc3);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_add_from_register_instruction() {
        let expected = Instruction::AddFromRegister {
            register1: Register::General(0xb),
            register2: Register::General(0xc),
        };

        let result = get_instruction(0x8b, 0xc4);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_subtract_register_y_from_x_instruction() {
        let expected = Instruction::Subtract {
            minuend: Register::General(0xb),
            subtrahend: Register::General(0xc),
            stored_in: Register::General(0xb)
        };

        let result = get_instruction(0x8b, 0xc5);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_subtract_register_x_from_y_instruction() {
        let expected = Instruction::Subtract {
            minuend: Register::General(0xc),
            subtrahend: Register::General(0xb),
            stored_in: Register::General(0xb)
        };

        let result = get_instruction(0x8b, 0xc7);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_shift_right_instruction() {
        let expected = Instruction::ShiftRight {
            register: Register::General(0xb),
        };

        let result = get_instruction(0x8b, 0x06);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_shift_left_instruction() {
        let expected = Instruction::ShiftLeft {
            register: Register::General(0xb),
        };

        let result = get_instruction(0x8b, 0x0e);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_skip_if_registers_not_equal_instruction() {
        let expected = Instruction::SkipIfRegistersNotEqual {
            register1: Register::General(0xb),
            register2: Register::General(0xc),
        };

        let result = get_instruction(0x9b, 0xc0);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_load_from_memory_to_i_register_instruction() {
        let expected = Instruction::LoadAddressIntoIRegister {
            address: 0x0123,
        };

        let result = get_instruction(0xa1, 0x23);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_jump_to_address_with_v0_instruction() {
        let expected = Instruction::JumpToAddress {
            address: 0x0123,
            add_register_0: true,
        };

        let result = get_instruction(0xB1, 0x23);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_set_random_byte_instruction() {
        let expected = Instruction::SetRandom {
            register: Register::General(0xb),
            and_value: 0x12
        };

        let result = get_instruction(0xCb, 0x12);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_draw_sprite_instruction() {
        let expected = Instruction::DrawSprite {
            x_register: Register::General(0xb),
            y_register: Register::General(0xc),
            height: 5,
        };

        let result = get_instruction(0xdb, 0xc5);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_skip_if_key_pressed_instruction() {
        let expected = Instruction::SkipIfKeyPressed {
            register: Register::General(0xb),
        };

        let result = get_instruction(0xeb, 0x9e);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_skip_if_key_not_pressed_instruction() {
        let expected = Instruction::SkipIfKeyNotPressed {
            register: Register::General(0xb),
        };

        let result = get_instruction(0xeb, 0xa1);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_register_from_delay_timer_instruction() {
        let expected = Instruction::LoadFromRegister {
            destination: Register::General(0xb),
            source: Register::DelayTimer,
        };

        let result = get_instruction(0xfb, 0x07);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_wait_for_key_press_instruction() {
        let expected = Instruction::LoadFromKeyPress {
            destination: Register::General(0xb)
        };

        let result = get_instruction(0xfb, 0x0a);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_delay_timer_from_register_instruction() {
        let expected = Instruction::LoadFromRegister {
            source: Register::General(0xb),
            destination: Register::DelayTimer,
        };

        let result = get_instruction(0xfb, 0x15);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_sound_timer_from_register_instruction() {
        let expected = Instruction::LoadFromRegister {
            source: Register::General(0xb),
            destination: Register::SoundTimer,
        };

        let result = get_instruction(0xfb, 0x18);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_add_to_i_register_instruction() {
        let expected = Instruction::AddFromRegister {
            register1: Register::I,
            register2: Register::General(0xb)
        };

        let result = get_instruction(0xfb, 0x1e);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_from_sprite_location_instruction() {
        let expected = Instruction::LoadSpriteLocation {
            sprite_digit: Register::General(0xb),
        };

        let result = get_instruction(0xfb, 0x29);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_bcd_value_instruction() {
        let expected = Instruction::LoadBcdValue {
            source: Register::General(0xb),
        };

        let result = get_instruction(0xfb, 0x33);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_into_memory_instruction() {
        let expected = Instruction::LoadIntoMemory {
            last_register: Register::General(0xb),
        };

        let result = get_instruction(0xfb, 0x55);
        assert_eq!(result, expected);
    }

    #[test]
    fn can_read_load_from_memory_instruction() {
        let expected = Instruction::LoadFromMemory {
            last_register: Register::General(0xb),
        };

        let result = get_instruction(0xfb, 0x65);
        assert_eq!(result, expected);
    }
}
