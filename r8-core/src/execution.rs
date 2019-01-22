use custom_error::custom_error;
use ::{Hardware, Instruction, Register};
use ::hardware::{STACK_SIZE, MEMORY_SIZE, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};

custom_error!{pub ExecutionError
    InvalidRegisterForInstruction {instruction:Instruction} = "Invalid register was used for instruction: {instruction}",
    UnhandleableInstruction {instruction:Instruction} = "The instruction '{instruction}' is not known",
    StackOverflow = "Call exceeded maximum stack size",
    InvalidCallOrJumpAddress {address:u16} = "Call performed to invalid address {address}",
    EmptyStack = "Return was called with an empty stack",
    InvalidFontDigit {digit: u8} = "Font digit of {digit} is invalid, only 0-f is allowed",
}

pub fn execute_instruction(instruction: Instruction, hardware: &mut Hardware) -> Result<(), ExecutionError> {
    match instruction {
        Instruction::AddFromRegister {register1, register2} => {
            let reg1_num = match register1 {
                Register::General(x) => x,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::AddFromRegister {register1, register2}}),
            };

            let reg2_num = match register2 {
                Register::General(x) => x,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::AddFromRegister {register1, register2}}),
            };

            let reg1_value = hardware.gen_registers[reg1_num as usize];
            let reg2_value = hardware.gen_registers[reg2_num as usize];
            let will_wrap = reg1_value > 0 && std::u8::MAX - reg1_value < reg2_value;;

            hardware.gen_registers[reg1_num as usize] = reg1_value.wrapping_add(reg2_value);
            hardware.gen_registers[0xf] = if will_wrap { 1 } else { 0};
            hardware.program_counter += 2;
        }

        Instruction::AddFromValue {register, value} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::AddFromValue {register, value}}),
            };

            hardware.gen_registers[reg_num] = hardware.gen_registers[reg_num].wrapping_add(value);
            hardware.program_counter += 2;
        }

        Instruction::And {register1, register2} => {
            let reg_num1 = match register1 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::And {register1, register2}}),
            };

            let reg_num2 = match register2 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::And {register1, register2}}),
            };

            hardware.gen_registers[reg_num1] = hardware.gen_registers[reg_num1] & hardware.gen_registers[reg_num2];
            hardware.program_counter += 2;
        }

        Instruction::Call {address} => {
            if hardware.stack_pointer >= STACK_SIZE {
                return Err(ExecutionError::StackOverflow);
            }

            if address % 2 != 0 {
                return Err(ExecutionError::InvalidCallOrJumpAddress {address});
            }

            hardware.stack[hardware.stack_pointer] = hardware.program_counter;
            hardware.stack_pointer = hardware.stack_pointer + 1;
            hardware.program_counter = address;
        }

        Instruction::ClearDisplay => {
            for x in 0..hardware.framebuffer.len() {
                for y in 0..hardware.framebuffer[x].len() {
                    hardware.framebuffer[x][y] = 0;
                }
            }

            hardware.program_counter += 2;
        }

        Instruction::DrawSprite {x_register, y_register, height} => {
            let x_reg_num = match x_register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::DrawSprite {x_register, y_register, height}}),
            };

            let y_reg_num = match y_register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::DrawSprite {x_register, y_register, height}}),
            };

            let first_row = hardware.gen_registers[y_reg_num] as usize;
            let first_pixel = hardware.gen_registers[x_reg_num] as usize;
            let shift_amount = first_pixel % 8;

            let left_column_set = first_pixel / 8;

            // According to the Cowgod spec, if the right column set would be out of bounds it
            // wraps to the other side on the same row
            let right_column_set = (left_column_set + 1) % (FRAMEBUFFER_WIDTH / 8);

            let mut collisions_found = false;
            for x in 0..height as usize {
                let sprite_byte = hardware.memory[hardware.i_register as usize + x];
                let left_byte = sprite_byte >> shift_amount;

                // According to Cowgod spec, if we've gone past the screen in height then wrap to the top
                let row = (first_row + x) % FRAMEBUFFER_HEIGHT;

                // Detect if the xor will reset any already on pixels
                if hardware.framebuffer[row][left_column_set] & left_byte > 0 {
                    collisions_found = true;
                }

                // Update framebuffer
                hardware.framebuffer[row][left_column_set] ^= left_byte;

                // If we are affecting pixels across column set boundaries, repeat for the next byte
                if shift_amount > 0 {
                    let right_byte = (sprite_byte & 0b00000011) << 8 - shift_amount;

                    if hardware.framebuffer[row][right_column_set] & right_byte > 0 {
                        collisions_found = true;
                    }

                    hardware.framebuffer[row][right_column_set] ^= right_byte;
                }
            }

            hardware.program_counter += 2;
            hardware.gen_registers[0xf] = if collisions_found { 1 } else { 0 };
        }

        Instruction::JumpToAddress {address, add_register_0} => {
            let final_address = match add_register_0 {
                true => address + hardware.gen_registers[0] as u16,
                false => address
            };

            if final_address < 512 || final_address > MEMORY_SIZE as u16 || final_address % 2 != 0 {
                return Err(ExecutionError::InvalidCallOrJumpAddress {address: final_address});
            }

            hardware.program_counter = final_address;
        }

        Instruction::LoadAddressIntoIRegister {address} => {
            hardware.i_register = address;
            hardware.program_counter += 2;
        }

        Instruction::LoadBcdValue {source} => {
            let reg_num = match source {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadBcdValue {source}}),
            };

            let start_address = hardware.i_register as usize;
            let source_value = hardware.gen_registers[reg_num];

            hardware.memory[start_address] = (source_value / 100) % 10;
            hardware.memory[start_address + 1] = (source_value / 10) % 10;
            hardware.memory[start_address + 2] = source_value % 10;
            hardware.program_counter += 2;
        }

        Instruction::LoadFromKeyPress {destination} => {
            let reg_num = match destination {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromKeyPress {destination}}),
            };

            // According to specs I have found this instruction does not recognize a key if it's
            // currently down.  So it will wait (stay on the same program counter for our purposes)
            // until the user releases the key, at which point for one execution
            // `hardware.key_released_since_last_instruction` should have the key that was just released.

            if let Some(key_num) = hardware.key_released_since_last_instruction {
                hardware.gen_registers[reg_num] = key_num;
                hardware.program_counter += 2;
            }
        }

        Instruction::LoadFromMemory {last_register} => {
            let reg_num = match last_register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromMemory {last_register}}),
            };

            for index in 0..=reg_num {
                hardware.gen_registers[index] = hardware.memory[hardware.i_register as usize + index];
            }

            hardware.i_register = hardware.i_register + reg_num as u16 + 1;
            hardware.program_counter += 2;
        }

        Instruction::LoadFromRegister {destination, source} => {
            let dest_register_num = match destination {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromRegister {destination, source}}),
            };

            let source_register_num = match source {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromRegister {destination, source}}),
            };

            hardware.gen_registers[dest_register_num] = hardware.gen_registers[source_register_num];
            hardware.program_counter += 2;
        }

        Instruction::LoadFromValue {destination, value} => {
            let reg_num = match destination {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromValue {destination, value}}),
            };

            hardware.gen_registers[reg_num] = value;
            hardware.program_counter += 2;
        }

        Instruction::LoadIntoMemory {last_register} => {
            let reg_num = match last_register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadIntoMemory {last_register}}),
            };

            for index in 0..=reg_num {
                hardware.memory[hardware.i_register as usize + index] = hardware.gen_registers[index];
            }

            hardware.i_register = hardware.i_register + reg_num as u16 + 1;
            hardware.program_counter += 2;
        }

        Instruction::LoadSpriteLocation {sprite_digit} => {
            let reg_num = match sprite_digit {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadSpriteLocation {sprite_digit}}),
            };

            let digit = hardware.gen_registers[reg_num];
            if digit > 0xf {
                return Err(ExecutionError::InvalidFontDigit {digit});
            }

            hardware.i_register = hardware.font_addresses[&digit];
            hardware.program_counter += 2;
        }

        Instruction::Or {register1, register2} => {
            let reg_num1 = match register1 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Or {register1, register2}}),
            };

            let reg_num2 = match register2 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Or {register1, register2}}),
            };

            hardware.gen_registers[reg_num1] = hardware.gen_registers[reg_num1] | hardware.gen_registers[reg_num2];
            hardware.program_counter += 2;
        }

        Instruction::Return => {
            if hardware.stack_pointer == 0 {
                return Err(ExecutionError::EmptyStack);
            }

            hardware.program_counter = hardware.stack[hardware.stack_pointer - 1];
            hardware.stack_pointer = hardware.stack_pointer - 1;
        }

        Instruction::SetRandom {register, and_value} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SetRandom {register, and_value}}),
            };

            hardware.gen_registers[reg_num] = rand::random::<u8>() & and_value;
            hardware.program_counter += 2;
        }

        Instruction::ShiftLeft {register} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::ShiftLeft {register}}),
            };

            hardware.gen_registers[reg_num] = hardware.gen_registers[reg_num] << 1;
            hardware.program_counter += 2;
        }

        Instruction::ShiftRight {register} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::ShiftRight {register}}),
            };

            hardware.gen_registers[reg_num] = hardware.gen_registers[reg_num] >> 1;
            hardware.program_counter += 2;
        }

        Instruction::SkipIfEqual {register, value} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfEqual {register, value}}),
            };

            let increment = match hardware.gen_registers[reg_num] == value {
                true => 4,
                false => 2,
            };

            hardware.program_counter += increment;
        }

        Instruction::SkipIfKeyPressed {register} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfKeyPressed {register}}),
            };

            let increment = match hardware.current_key_down {
                Some(x) if x == hardware.gen_registers[reg_num] => 4,
                _ => 2,
            };

            hardware.program_counter += increment;
        }

        Instruction::SkipIfKeyNotPressed {register} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfKeyNotPressed {register}}),
            };

            let increment = match hardware.current_key_down {
                Some(x) if x == hardware.gen_registers[reg_num] => 2,
                _ => 4,
            };

            hardware.program_counter += increment;
        }

        Instruction::SkipIfNotEqual {register, value} => {
            let reg_num = match register {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfNotEqual {register, value}}),
            };

            let increment = match hardware.gen_registers[reg_num] == value {
                true => 2,
                false => 4,
            };

            hardware.program_counter += increment;
        }

        Instruction::SkipIfRegistersEqual {register1, register2} => {
            let reg_num1 = match register1 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfRegistersEqual {register1, register2}}),
            };

            let reg_num2 = match register2 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfRegistersEqual {register1, register2}}),
            };

            let increment = match hardware.gen_registers[reg_num1] == hardware.gen_registers[reg_num2]  {
                true => 4,
                false => 2,
            };

            hardware.program_counter += increment;
        }

        Instruction::SkipIfRegistersNotEqual {register1, register2} => {
            let reg_num1 = match register1 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfRegistersNotEqual {register1, register2}}),
            };

            let reg_num2 = match register2 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::SkipIfRegistersNotEqual {register1, register2}}),
            };

            let increment = match hardware.gen_registers[reg_num1] == hardware.gen_registers[reg_num2]  {
                true => 2,
                false => 4,
            };

            hardware.program_counter += increment;
        }

        Instruction::Subtract {minuend, subtrahend, stored_in} => {
            let minuend_reg = match minuend {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Subtract {minuend, subtrahend, stored_in}}),
            };

            let subtrahend_reg = match subtrahend {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Subtract {minuend, subtrahend, stored_in}}),
            };

            let stored_in_reg = match stored_in {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Subtract {minuend, subtrahend, stored_in}}),
            };

            let will_underflow = hardware.gen_registers[minuend_reg] < hardware.gen_registers[subtrahend_reg];
            let difference = hardware.gen_registers[minuend_reg].wrapping_sub(hardware.gen_registers[subtrahend_reg]);
            hardware.gen_registers[stored_in_reg] = difference;
            hardware.gen_registers[0xf] = if will_underflow { 1 } else { 0 };
            hardware.program_counter += 2;
        }

        Instruction::Xor {register1, register2} => {
            let reg_num1 = match register1 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Xor {register1, register2}}),
            };

            let reg_num2 = match register2 {
                Register::General(x) => x as usize,
                _ => return Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::Xor {register1, register2}}),
            };

            hardware.gen_registers[reg_num1] = hardware.gen_registers[reg_num1] ^ hardware.gen_registers[reg_num2];
            hardware.program_counter += 2;
        }

        _ => return Err(ExecutionError::UnhandleableInstruction{instruction})
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::{Hardware, Register};

    #[test]
    fn can_add_value_to_general_register() {
        const REGISTER_NUMBER: u8 = 3;
        let mut hardware = Hardware::new();
        hardware.gen_registers[REGISTER_NUMBER as usize] = 100;
        hardware.program_counter = 1000;

        let instruction = Instruction::AddFromValue {
            register: Register::General(REGISTER_NUMBER),
            value: 12,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.gen_registers[REGISTER_NUMBER as usize], 112, "Invalid register value");
        assert_eq!(hardware.program_counter, 1002, "Invalid resulting program counter");
    }

    #[test]
    fn can_add_value_to_general_register_that_overflows() {
        const REGISTER_NUMBER: u8 = 3;
        let mut hardware = Hardware::new();
        hardware.gen_registers[REGISTER_NUMBER as usize] = 100;
        hardware.program_counter = 1000;

        let instruction = Instruction::AddFromValue {
            register: Register::General(REGISTER_NUMBER),
            value: 165,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.gen_registers[REGISTER_NUMBER as usize], 9, "Invalid register value");
        assert_eq!(hardware.program_counter, 1002, "Invalid resulting program counter");
        assert_eq!(hardware.gen_registers[0xf], 0, "Add by value should not have caused carry mark");
    }

    #[test]
    fn can_add_value_from_general_register_without_carry() {
        const REGISTER1_NUMBER: u8 = 4;
        const REGISTER2_NUMBER: u8 = 6;
        let mut hardware = Hardware::new();
        hardware.gen_registers[REGISTER1_NUMBER as usize] = 100;
        hardware.gen_registers[REGISTER2_NUMBER as usize] = 55;
        hardware.program_counter = 1000;

        let instruction = Instruction::AddFromRegister {
            register1: Register::General(REGISTER1_NUMBER),
            register2: Register::General(REGISTER2_NUMBER),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.gen_registers[REGISTER1_NUMBER as usize], 155, "Invalid register value");
        assert_eq!(hardware.gen_registers[0xf], 0, "Invalid VF register value");
        assert_eq!(hardware.program_counter, 1002, "Invalid resulting program counter");
    }

    #[test]
    fn can_add_value_from_general_register_with_carry() {
        const REGISTER1_NUMBER: u8 = 4;
        const REGISTER2_NUMBER: u8 = 6;
        let mut hardware = Hardware::new();
        hardware.gen_registers[REGISTER1_NUMBER as usize] = 200;
        hardware.gen_registers[REGISTER2_NUMBER as usize] = 65;
        hardware.program_counter = 1000;

        let instruction = Instruction::AddFromRegister {
            register1: Register::General(REGISTER1_NUMBER),
            register2: Register::General(REGISTER2_NUMBER),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.gen_registers[REGISTER1_NUMBER as usize], 9, "Invalid register value");
        assert_eq!(hardware.gen_registers[0xf], 1, "Invalid VF register value");
        assert_eq!(hardware.program_counter, 1002, "Invalid resulting program counter");
    }

    #[test]
    fn can_execute_call_instruction() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.stack[0] = 567;
        hardware.stack[1] = 599;
        hardware.stack_pointer = 2;

        let instruction = Instruction::Call {address: 1654};

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.stack_pointer, 3, "Incorrect stack pointer");
        assert_eq!(hardware.stack[0], 567, "Incorrect address at stack 0");
        assert_eq!(hardware.stack[1], 599, "Incorrect address at stack 1");
        assert_eq!(hardware.stack[2], 1000, "Incorrect address at stack 2");
        assert_eq!(hardware.program_counter, 1654, "Incorrect program counter value");
    }

    #[test]
    fn stack_overflow_error_when_call_performed_at_max_stack() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.stack_pointer = 16;

        let instruction = Instruction::Call {address: 1654};
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::StackOverflow => (),
            x => panic!("Expected StackOverflow, instead got {:?}", x),
        }
    }

    #[test]
    fn cannot_call_to_odd_memory_address() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;

        let instruction = Instruction::Call {address: 1653};
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::InvalidCallOrJumpAddress {address: 1653} => (),
            x => panic!("Expected InvalidCallOrJumpAddress {{address: 1653}}, instead got {:?}", x),
        }
    }

    #[test]
    fn can_call_jump_to_address_without_add() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1002;
        hardware.gen_registers[0] = 10;
        hardware.stack_pointer = 1;
        hardware.stack[0] = 533;

        let instruction = Instruction::JumpToAddress {address: 2330, add_register_0: false};
        execute_instruction(instruction, &mut hardware).unwrap();

        assert_eq!(hardware.program_counter, 2330, "Incorrect program counter value");
        assert_eq!(hardware.stack_pointer, 1, "Incorrect stack pointer value"); // Make sure stack wasn't messed with
        assert_eq!(hardware.stack[0], 533, "Incorrect stack[0] value");
    }

    #[test]
    fn can_call_jump_address_with_add() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1002;
        hardware.gen_registers[0] = 10;
        hardware.stack_pointer = 1;
        hardware.stack[0] = 533;

        let instruction = Instruction::JumpToAddress {address: 2330, add_register_0: true};
        execute_instruction(instruction, &mut hardware).unwrap();

        assert_eq!(hardware.program_counter, 2340, "Incorrect program counter value");
        assert_eq!(hardware.stack_pointer, 1, "Incorrect stack pointer value"); // Make sure stack wasn't messed with
        assert_eq!(hardware.stack[0], 533, "Incorrect stack[0] value");
    }

    #[test]
    fn cannot_jump_to_odd_address() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1002;
        hardware.gen_registers[0] = 10;
        hardware.stack_pointer = 1;
        hardware.stack[0] = 533;

        let instruction = Instruction::JumpToAddress {address: 2331, add_register_0: false};
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::InvalidCallOrJumpAddress {address: 2331} => (),
            x => panic!("Expected InvalidCallOrJumpAddress {{address: 2331}}, instead got {:?}", x),
        }
    }

    #[test]
    fn cannot_jump_to_address_below_512() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1002;
        hardware.gen_registers[0] = 10;
        hardware.stack_pointer = 1;
        hardware.stack[0] = 533;

        let instruction = Instruction::JumpToAddress {address: 511, add_register_0: false};
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::InvalidCallOrJumpAddress {address: 511} => (),
            x => panic!("Expected InvalidCallOrJumpAddress {{address: 2331}}, instead got {:?}", x),
        }
    }

    #[test]
    fn cannot_jump_to_address_above_memory_size() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1002;
        hardware.gen_registers[0] = 10;
        hardware.stack_pointer = 1;
        hardware.stack[0] = 533;

        let address = MEMORY_SIZE as u16 + 1;
        let instruction = Instruction::JumpToAddress {address, add_register_0: false};
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::InvalidCallOrJumpAddress {address: _} => (),
            x => panic!("Expected InvalidCallOrJumpAddress {{address: {}}}, instead got {:?}", address, x),
        }
    }

    #[test]
    fn jump_to_machine_code_is_unhandled() {
        // According to specs, SYS instructions are ignored by modern interpreters.

        let mut hardware = Hardware::new();
        let instruction = Instruction::JumpToMachineCode {address: 123};
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::UnhandleableInstruction {instruction: _} => (),
            x => panic!("Expected UnhandleableInstruction, instead got {:?}", x),
        }
    }

    #[test]
    fn can_load_from_value_into_general_register() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 10;

        let instruction = Instruction::LoadFromValue {
            destination: Register::General(4),
            value: 123,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.gen_registers[4], 123, "Incorrect value in register");
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn can_load_from_register_into_general_register() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 10;
        hardware.gen_registers[5] = 122;

        let instruction = Instruction::LoadFromRegister {
            destination: Register::General(4),
            source: Register::General(5),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.gen_registers[4], 122, "Incorrect value in register");
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn load_from_key_press_does_not_progress_if_no_key_released() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 10;
        hardware.current_key_down = Some(0x4);
        hardware.key_released_since_last_instruction = None;

        let instruction = Instruction::LoadFromKeyPress {destination: Register::General(4)};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1000, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[4], 10, "Register 4 value should not have changed");
    }

    #[test]
    fn load_from_key_press_proceeds_if_key_was_released() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 10;
        hardware.current_key_down = None;
        hardware.key_released_since_last_instruction = Some(0x5);

        let instruction = Instruction::LoadFromKeyPress {destination: Register::General(4)};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[4], 5, "Incorrect value in register");
    }

    #[test]
    fn can_load_bcd_value_into_memory() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 235;
        hardware.i_register = 1500;

        let instruction = Instruction::LoadBcdValue {source: Register::General(5)};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.memory[1500], 2, "Incorrect bcd value #1");
        assert_eq!(hardware.memory[1501], 3, "Incorrect bcd value #2");
        assert_eq!(hardware.memory[1502], 5, "Incorrect bcd value #3");
    }

    #[test]
    fn can_load_register_values_into_memory() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[0] = 100;
        hardware.gen_registers[1] = 101;
        hardware.gen_registers[2] = 102;
        hardware.gen_registers[3] = 103;
        hardware.gen_registers[4] = 104;
        hardware.gen_registers[5] = 105;
        hardware.i_register = 933;

        let instruction = Instruction::LoadIntoMemory {last_register: Register::General(4)};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.memory[933], 100, "Incorrect value in memory location 0");
        assert_eq!(hardware.memory[934], 101, "Incorrect value in memory location 1");
        assert_eq!(hardware.memory[935], 102, "Incorrect value in memory location 2");
        assert_eq!(hardware.memory[936], 103, "Incorrect value in memory location 3");
        assert_eq!(hardware.memory[937], 104, "Incorrect value in memory location 4");
        assert_eq!(hardware.memory[938], 0, "Incorrect value in memory location 5");
        assert_eq!(hardware.i_register, 938, "Incorrect resulting I register");
    }

    #[test]
    fn can_load_memory_into_multiple_register_values() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.i_register = 933;
        hardware.memory[933] = 100;
        hardware.memory[934] = 101;
        hardware.memory[935] = 102;
        hardware.memory[936] = 103;
        hardware.memory[937] = 104;
        hardware.memory[938] = 105;

        let instruction = Instruction::LoadFromMemory {last_register: Register::General(4)};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[0], 100, "Incorrect value in register V0");
        assert_eq!(hardware.gen_registers[1], 101, "Incorrect value in register V1");
        assert_eq!(hardware.gen_registers[2], 102, "Incorrect value in register V2");
        assert_eq!(hardware.gen_registers[3], 103, "Incorrect value in register V3");
        assert_eq!(hardware.gen_registers[4], 104, "Incorrect value in register V4");
        assert_eq!(hardware.gen_registers[5], 0, "Incorrect value in register V5");
        assert_eq!(hardware.i_register, 938, "Incorrect resulting I register");
    }

    #[test]
    fn can_execute_return_instruction() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.stack_pointer = 2;
        hardware.stack[0] = 1500;
        hardware.stack[1] = 938;
        hardware.stack[2] = 1700; // residual from previous call

        let instruction = Instruction::Return;
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 938, "Incorrect program pointer");
        assert_eq!(hardware.stack_pointer, 1, "Incorrect stack pointer");
    }

    #[test]
    fn cannot_execute_return_with_empty_stack() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.stack_pointer = 0;
        hardware.stack[0] = 1500;
        hardware.stack[1] = 938;

        let instruction = Instruction::Return;
        match execute_instruction(instruction, &mut hardware).unwrap_err() {
            ExecutionError::EmptyStack => (),
            x => panic!("Expected EmptyStack instead got {:?}", x),
        }
    }

    #[test]
    fn skip_occurs_when_skip_if_equal_passes() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfEqual {
            register: Register::General(5),
            value: 23,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1004, "Incorrect program counter");
    }

    #[test]
    fn does_not_skip_when_skip_if_equal_fails() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfEqual {
            register: Register::General(5),
            value: 24,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn skip_occurs_when_skip_if_not_equal_passes() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfNotEqual {
            register: Register::General(5),
            value: 25,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1004, "Incorrect program counter");
    }

    #[test]
    fn does_not_skip_occurs_when_skip_if_not_equal_fails() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfNotEqual {
            register: Register::General(5),
            value: 23,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn skip_occurs_when_skip_if_register_equals_passes() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 23;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfRegistersEqual {
            register1: Register::General(5),
            register2: Register::General(4),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1004, "Incorrect program counter");
    }

    #[test]
    fn does_not_skip_occurs_when_skip_if_register_equals_fails() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 25;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfRegistersEqual {
            register1: Register::General(5),
            register2: Register::General(4),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn skip_occurs_when_skip_if_register_not_equals_passes() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 25;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfRegistersNotEqual {
            register1: Register::General(5),
            register2: Register::General(4),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1004, "Incorrect program counter");
    }

    #[test]
    fn does_not_skip_occurs_when_skip_if_register_not_equals_fails() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 23;
        hardware.gen_registers[5] = 23;

        let instruction = Instruction::SkipIfRegistersNotEqual {
            register1: Register::General(5),
            register2: Register::General(4),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn skip_occurs_when_skip_if_key_pressed_passes() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 10;
        hardware.current_key_down = Some(10);

        let instruction = Instruction::SkipIfKeyPressed {
            register: Register::General(5),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1004, "Incorrect program counter");
    }

    #[test]
    fn does_not_skip_occurs_when_skip_if_key_pressed_fails() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 10;
        hardware.current_key_down = Some(11);

        let instruction = Instruction::SkipIfKeyPressed {
            register: Register::General(5),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn skip_occurs_when_skip_if_key_not_pressed_passes() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 10;
        hardware.current_key_down = Some(11);

        let instruction = Instruction::SkipIfKeyNotPressed {
            register: Register::General(5),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1004, "Incorrect program counter");
    }

    #[test]
    fn does_not_skip_occurs_when_skip_if_key_not_pressed_fails() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[5] = 10;
        hardware.current_key_down = Some(10);

        let instruction = Instruction::SkipIfKeyNotPressed {
            register: Register::General(5),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
    }

    #[test]
    fn can_or_register_values_together() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[2] = 123;
        hardware.gen_registers[3] = 203;

        let instruction = Instruction::Or {
            register1: Register::General(3),
            register2: Register::General(2),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[2], 123, "Incorrect V2 value");
        assert_eq!(hardware.gen_registers[3], 203 | 123, "Incorrect v3 value");
    }

    #[test]
    fn can_and_register_values_together() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[2] = 123;
        hardware.gen_registers[3] = 203;

        let instruction = Instruction::And {
            register1: Register::General(3),
            register2: Register::General(2),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[2], 123, "Incorrect V2 value");
        assert_eq!(hardware.gen_registers[3], 203 & 123, "Incorrect v3 value");
    }

    #[test]
    fn can_xor_register_values_together() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[2] = 123;
        hardware.gen_registers[3] = 203;

        let instruction = Instruction::Xor {
            register1: Register::General(3),
            register2: Register::General(2),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[2], 123, "Incorrect V2 value");
        assert_eq!(hardware.gen_registers[3], 203 ^ 123, "Incorrect v3 value");
    }

    #[test]
    fn can_shift_register_value_right() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[3] = 203;

        let instruction = Instruction::ShiftRight {
            register: Register::General(3),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[3], 203 >> 1, "Incorrect v3 value");
    }

    #[test]
    fn can_shift_register_value_left() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[3] = 203;

        let instruction = Instruction::ShiftLeft {
            register: Register::General(3),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[3], 203 << 1, "Incorrect v3 value");
    }

    #[test]
    fn can_get_random_number() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[3] = 100;

        let instruction = Instruction::SetRandom {
            register: Register::General(3),
            and_value: 23,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");

        let value1 = hardware.gen_registers[3];

        let instruction = Instruction::SetRandom {
            register: Register::General(3),
            and_value: 23,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        let value2 = hardware.gen_registers[3];

        assert_ne!(value1, value2, "Values 1 and 2 were the same (possibly not random??)");
    }

    #[test]
    fn can_subtract_register_without_underflow() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 100;
        hardware.gen_registers[5] = 25;

        let instruction = Instruction::Subtract {
            minuend: Register::General(4),
            subtrahend: Register::General(5),
            stored_in: Register::General(4),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[4], 75, "Incorrect V4 register");
        assert_eq!(hardware.gen_registers[5], 25, "Incorrect V5 register");
        assert_eq!(hardware.gen_registers[0xf], 0, "Incorrect VF register");
    }

    #[test]
    fn can_subtract_register_with_underflow() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 100;
        hardware.gen_registers[5] = 25;

        let instruction = Instruction::Subtract {
            minuend: Register::General(5),
            subtrahend: Register::General(4),
            stored_in: Register::General(5),
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[4], 100, "Incorrect V4 register");
        assert_eq!(hardware.gen_registers[5], 181, "Incorrect V5 register");
        assert_eq!(hardware.gen_registers[0xf], 1, "Incorrect VF register");
    }

    #[test]
    fn can_clear_display() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        for x in 0..hardware.framebuffer.len() {
            for y in 0..hardware.framebuffer[x].len() {
                hardware.framebuffer[x][y] = 0xFF;
            }
        }

        let instruction = Instruction::ClearDisplay;
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");

        for x in 0..hardware.framebuffer.len() {
            for y in 0..hardware.framebuffer[x].len() {
                if hardware.framebuffer[x][y] != 0 {
                    panic!("Expected frame buffer by {}x{} to be 0", x, y);
                }
            }
        }
    }

    #[test]
    fn can_load_digit_sprite_location() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.gen_registers[4] = 0xa;

        let instruction = Instruction::LoadSpriteLocation {sprite_digit: Register::General(4)};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.i_register, hardware.font_addresses[&0xa], "Incorrect sprite address");
    }

    #[test]
    fn can_load_address_into_i_register() {
        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;

        let instruction = Instruction::LoadAddressIntoIRegister {address: 0x123};
        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.i_register, 0x123, "Incorrect I register value");
    }

    #[test]
    fn visible_8_x_3_sprite_to_screen_on_x_multiple_of_8_can_be_drawn() {
        const SPRITE_START_ADDRESS: usize = 1046;
        const X_POS: u8 = 16;
        const Y_POS: u8 = 2;

        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.i_register = SPRITE_START_ADDRESS as u16;
        hardware.gen_registers[4] = X_POS;
        hardware.gen_registers[3] = Y_POS;
        hardware.memory[SPRITE_START_ADDRESS] = 0b10101010;
        hardware.memory[SPRITE_START_ADDRESS + 1] = 0b01010101;
        hardware.memory[SPRITE_START_ADDRESS + 2] = 0b11001101;
        hardware.memory[SPRITE_START_ADDRESS + 3] = 0b11111111;

        let instruction = Instruction::DrawSprite {
            x_register: Register::General(4),
            y_register: Register::General(3),
            height: 3,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[0xf], 0, "Incorrect VF value");
        assert_eq!(hardware.framebuffer[2][2], 0b10101010, "Incorrect framebuffer value at row 2 column byte 2");
        assert_eq!(hardware.framebuffer[3][2], 0b01010101, "Incorrect framebuffer value at row 3 column byte 2");
        assert_eq!(hardware.framebuffer[4][2], 0b11001101, "Incorrect framebuffer value at row 4 column byte 2");
        assert_eq!(hardware.framebuffer[5][2], 0, "Incorrect framebuffer value at row 5 column byte 2");
    }

    #[test]
    fn visible_8_x_3_sprite_to_screen_on_x_non_multiple_of_8_can_be_drawn() {
        const SPRITE_START_ADDRESS: usize = 1046;
        const X_POS: u8 = 18;
        const Y_POS: u8 = 2;

        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.i_register = SPRITE_START_ADDRESS as u16;
        hardware.gen_registers[4] = X_POS;
        hardware.gen_registers[3] = Y_POS;
        hardware.memory[SPRITE_START_ADDRESS] = 0b10101010;
        hardware.memory[SPRITE_START_ADDRESS + 1] = 0b01010101;
        hardware.memory[SPRITE_START_ADDRESS + 2] = 0b11001101;
        hardware.memory[SPRITE_START_ADDRESS + 3] = 0b11111111;

        let instruction = Instruction::DrawSprite {
            x_register: Register::General(4),
            y_register: Register::General(3),
            height: 3,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[0xf], 0, "Incorrect VF value");
        assert_eq!(hardware.framebuffer[2][2], 0b00101010, "Incorrect framebuffer value at row 2 column byte 2");
        assert_eq!(hardware.framebuffer[2][3], 0b10000000, "Incorrect framebuffer value at row 2 column byte 3");
        assert_eq!(hardware.framebuffer[3][2], 0b00010101, "Incorrect framebuffer value at row 3 column byte 2");
        assert_eq!(hardware.framebuffer[3][3], 0b01000000, "Incorrect framebuffer value at row 3 column byte 3");
        assert_eq!(hardware.framebuffer[4][2], 0b00110011, "Incorrect framebuffer value at row 4 column byte 2");
        assert_eq!(hardware.framebuffer[4][3], 0b01000000, "Incorrect framebuffer value at row 4 column byte 3");
        assert_eq!(hardware.framebuffer[5][2], 0, "Incorrect framebuffer value at row 5 column byte 2");
        assert_eq!(hardware.framebuffer[5][3], 0, "Incorrect framebuffer value at row 5 column byte 3");
    }

    #[test]
    fn sprites_xor_existing_framebuffer_values() {
        const SPRITE_START_ADDRESS: usize = 1046;
        const X_POS: u8 = 18;
        const Y_POS: u8 = 2;

        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.i_register = SPRITE_START_ADDRESS as u16;
        hardware.gen_registers[4] = X_POS;
        hardware.gen_registers[3] = Y_POS;
        hardware.memory[SPRITE_START_ADDRESS] = 0b10101010;
        hardware.memory[SPRITE_START_ADDRESS + 1] = 0b01010101;
        hardware.memory[SPRITE_START_ADDRESS + 2] = 0b11001101;
        hardware.framebuffer[2][2] = 0xFF;
        hardware.framebuffer[2][3] = 0xFF;

        let instruction = Instruction::DrawSprite {
            x_register: Register::General(4),
            y_register: Register::General(3),
            height: 3,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[0xf], 1, "Incorrect VF value");
        assert_eq!(hardware.framebuffer[2][2], 0b00101010 ^ 0xFF, "Incorrect framebuffer value at row 2 column byte 2");
        assert_eq!(hardware.framebuffer[2][3], 0b10000000 ^ 0xFF, "Incorrect framebuffer value at row 2 column byte 3");
        assert_eq!(hardware.framebuffer[3][2], 0b00010101, "Incorrect framebuffer value at row 3 column byte 2");
        assert_eq!(hardware.framebuffer[3][3], 0b01000000, "Incorrect framebuffer value at row 3 column byte 3");
        assert_eq!(hardware.framebuffer[4][2], 0b00110011, "Incorrect framebuffer value at row 4 column byte 2");
        assert_eq!(hardware.framebuffer[4][3], 0b01000000, "Incorrect framebuffer value at row 4 column byte 3");
        assert_eq!(hardware.framebuffer[5][2], 0, "Incorrect framebuffer value at row 5 column byte 2");
        assert_eq!(hardware.framebuffer[5][3], 0, "Incorrect framebuffer value at row 5 column byte 3");
    }

    #[test]
    fn partially_visibe_sprite_wraps_across_both_axis() {
        const SPRITE_START_ADDRESS: usize = 1046;
        const X_POS: u8 = 58;
        const Y_POS: u8 = 30;

        let mut hardware = Hardware::new();
        hardware.program_counter = 1000;
        hardware.i_register = SPRITE_START_ADDRESS as u16;
        hardware.gen_registers[4] = X_POS;
        hardware.gen_registers[3] = Y_POS;
        hardware.memory[SPRITE_START_ADDRESS] = 0b10101010;
        hardware.memory[SPRITE_START_ADDRESS + 1] = 0b01010101;
        hardware.memory[SPRITE_START_ADDRESS + 2] = 0b11001101;
        hardware.memory[SPRITE_START_ADDRESS + 3] = 0b11111111;

        let instruction = Instruction::DrawSprite {
            x_register: Register::General(4),
            y_register: Register::General(3),
            height: 3,
        };

        execute_instruction(instruction, &mut hardware).unwrap();
        assert_eq!(hardware.program_counter, 1002, "Incorrect program counter");
        assert_eq!(hardware.gen_registers[0xf], 0, "Incorrect VF value");
        assert_eq!(hardware.framebuffer[30][7], 0b00101010, "Incorrect framebuffer value at row 30 column byte 7");
        assert_eq!(hardware.framebuffer[30][0], 0b10000000, "Incorrect framebuffer value at row 30 column byte 0");
        assert_eq!(hardware.framebuffer[31][7], 0b00010101, "Incorrect framebuffer value at row 31 column byte 7");
        assert_eq!(hardware.framebuffer[31][0], 0b01000000, "Incorrect framebuffer value at row 31 column byte 0");
        assert_eq!(hardware.framebuffer[0][7], 0b00110011, "Incorrect framebuffer value at row 0 column byte 7");
        assert_eq!(hardware.framebuffer[0][0], 0b01000000, "Incorrect framebuffer value at row 0 column byte 0");
        assert_eq!(hardware.framebuffer[1][7], 0, "Incorrect framebuffer value at row 1 column byte 7");
        assert_eq!(hardware.framebuffer[1][0], 0, "Incorrect framebuffer value at row 1 column byte 0");
    }
}