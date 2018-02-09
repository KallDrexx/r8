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
            let address: u16 = ((byte1 as u16) * 16 * 16) + (byte2 as u16);
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
        }

        _ => Instruction::Unknown
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
}
