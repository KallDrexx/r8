use custom_error::custom_error;
use ::{Hardware, Instruction, Register};
use ::hardware::{STACK_SIZE, MEMORY_SIZE};

custom_error!{pub ExecutionError
    InvalidRegisterForInstruction {instruction:Instruction} = "Invalid register was used for instruction: {instruction}",
    UnhandleableInstruction {instruction:Instruction} = "The instruction '{instruction}' is not known",
    StackOverflow = "Call exceeded maximum stack size",
    InvalidCallOrJumpAddress {address:u16} = "Call performed to invalid address {address}",
}

pub fn execute_instruction(instruction: Instruction, hardware: &mut Hardware) -> Result<(), ExecutionError> {
    match instruction {
        Instruction::AddFromValue {register, value} => {
            match register {
                Register::General(num) => {
                    hardware.gen_registers[num as usize] = hardware.gen_registers[num as usize] + value;
                    hardware.program_counter = hardware.program_counter + 2;
                }

                _ => return Err(ExecutionError::InvalidRegisterForInstruction {
                    instruction: Instruction::AddFromValue {register, value}
                })
            }
        }

        Instruction::AddFromRegister {register1, register2} => {
            let reg1_num = match register1 {
                Register::General(x) => x,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::AddFromRegister {register1, register2}}),
            };

            let reg2_num = match register2 {
                Register::General(x) => x,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::AddFromRegister {register1, register2}}),
            };

            let reg1_value = hardware.gen_registers[reg1_num as usize];
            let reg2_value = hardware.gen_registers[reg2_num as usize];

            hardware.gen_registers[reg1_num as usize] = reg1_value + reg2_value;
            hardware.program_counter = hardware.program_counter + 2;
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

        Instruction::LoadFromValue {destination, value} => {
            let reg_num = match destination {
                Register::General(x) => x as usize,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromValue {destination, value}}),
            };

            hardware.gen_registers[reg_num] = value;
            hardware.program_counter = hardware.program_counter + 2;
        }

        Instruction::LoadFromRegister {destination, source} => {
            let dest_register_num = match destination {
                Register::General(x) => x as usize,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromRegister {destination, source}}),
            };

            let source_register_num = match source {
                Register::General(x) => x as usize,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromRegister {destination, source}}),
            };

            hardware.gen_registers[dest_register_num] = hardware.gen_registers[source_register_num];
            hardware.program_counter = hardware.program_counter + 2;
        }

        Instruction::LoadFromKeyPress {destination} => {
            let reg_num = match destination {
                Register::General(x) => x as usize,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadFromKeyPress {destination}}),
            };

            // According to specs I have found this instruction does not recognize a key if it's
            // currently down.  So it will wait (stay on the same program counter for our purposes)
            // until the user releases the key, at which point for one execution
            // `hardware.key_released_since_last_instruction` should have the key that was just released.

            if let Some(key_num) = hardware.key_released_since_last_instruction {
                hardware.gen_registers[reg_num] = key_num;
                hardware.program_counter = hardware.program_counter + 2;
            }
        }

        Instruction::LoadBcdValue {source} => {
            let reg_num = match source {
                Register::General(x) => x as usize,
                _ => return  Err(ExecutionError::InvalidRegisterForInstruction {instruction: Instruction::LoadBcdValue {source}}),
            };

            let start_address = hardware.i_register as usize;
            let source_value = hardware.gen_registers[reg_num];

            hardware.memory[start_address] = (source_value / 100) % 10;
            hardware.memory[start_address + 1] = (source_value / 10) % 10;
            hardware.memory[start_address + 2] = source_value % 10;
            hardware.program_counter = hardware.program_counter + 2;
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
    fn can_add_value_from_general_register() {
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
}