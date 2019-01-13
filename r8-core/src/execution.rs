use custom_error::custom_error;
use ::{Hardware, Instruction, Register};

custom_error!{pub ExecutionError
    InvalidRegisterForInstruction {instruction:Instruction} = "Invalid register was used for instruction: {instruction}",
    UnhandleableInstruction {instruction:Instruction} = "The instruction '{instruction}' is not known"
}

fn execute_instruction(instruction: Instruction, hardware: &mut Hardware) -> Result<(), ExecutionError> {
    match instruction {
        Instruction::AddFromValue {register, value} => {
            match register {
                Register::General(num) =>
                    hardware.gen_registers[num as usize] = hardware.gen_registers[num as usize] + value,

                _ => return Err(ExecutionError::InvalidRegisterForInstruction {
                    instruction: Instruction::AddFromValue {register, value}
                })
            }

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
}