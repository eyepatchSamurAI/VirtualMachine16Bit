use core::num;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    create_memory::{create_registers, Memory},
    instructions::Instruction,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RegisterName {
    Ip,  // Instruction Pointer
    Acc, // Accumulator, math values are added here
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    Sp, // Stack Pointer
    Fp, // Frame Pointer
}

#[derive(Debug)]
pub enum CpuError {
    RegisterNameDoesNotExist,
    RegisterOutOfBounds,
    MemoryOutOfBounds,
    InvalidInstruction,
}

pub struct Cpu {
    memory: Rc<RefCell<Memory>>,
    registers: Vec<u16>,
    register_names: Vec<RegisterName>,
    register_map: HashMap<RegisterName, usize>, // Set to u16 but it seems some of the addresses I push are bigger than that. I "attempt to add with overflow" on pop_state for the last set_register
    stack_frame_size: u16,
}

impl Cpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Self {
        let register_names = vec![
            RegisterName::Ip,
            RegisterName::Acc,
            RegisterName::R1,
            RegisterName::R2,
            RegisterName::R3,
            RegisterName::R4,
            RegisterName::R5,
            RegisterName::R6,
            RegisterName::R7,
            RegisterName::R8,
            RegisterName::Sp,
            RegisterName::Fp,
        ];
        let mut registers: Vec<u16> = create_registers(register_names.len());
        let register_map = register_names
            .iter()
            .enumerate()
            .map(|(i, x)| (x.to_owned(), i))
            .collect::<HashMap<_, _>>();
        // Set Frame Pointer position to be last in memory
        registers[register_names.len() - 1] = (memory.borrow().len() - 1 - 1) as u16;
        // SEt Stack Pointer position to last in memory
        registers[register_names.len() - 2] = (memory.borrow().len() - 1 - 1) as u16;

        Cpu {
            memory,
            registers,
            register_names,
            register_map,
            stack_frame_size: 0,
        }
    }

    pub fn debug(&self) {
        self.register_names.iter().for_each(|reg_name| {
            let reg_value = self.get_register(reg_name).unwrap();
            let padded_hex_value = format!("{:04x}", reg_value);
            println!("{:?}: 0x{}", reg_name, padded_hex_value);
        });
        println!();
    }

    pub fn view_memory_at(&self, address: usize, n: usize) {
        let memory_ref = self.memory.borrow();
        let next_n_bytes: Vec<String> = (0..n)
            .map(|index| {
                let byte = memory_ref.get(address + index).unwrap();
                format!("0x{:02x}", byte)
            })
            .collect();

        println!("0x{:04x}: {}", address, next_n_bytes.join(" "));
    }

    pub fn get_register(&self, name: &RegisterName) -> Result<&u16, CpuError> {
        let register_index = self
            .register_map
            .get(&name)
            .ok_or(CpuError::RegisterNameDoesNotExist)?;
        self.registers
            .get(*register_index as usize)
            .ok_or(CpuError::RegisterOutOfBounds)
    }

    fn set_register(&mut self, name: &RegisterName, value: u16) -> Result<(), CpuError> {
        let register_index = self
            .register_map
            .get(&name)
            .ok_or(CpuError::RegisterNameDoesNotExist)?;
        self.registers[*register_index as usize] = value;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u8, CpuError> {
        let next_instruction_address = *self.get_register(&RegisterName::Ip)?;
        self.set_register(&RegisterName::Ip, next_instruction_address + 1)?;
        let instruction = self
            .memory
            .borrow()
            .get(next_instruction_address as usize)
            .copied()
            .ok_or(CpuError::MemoryOutOfBounds);
        return instruction;
    }

    fn fetch16(&mut self) -> Result<u16, CpuError> {
        let next_instruction_address = *self.get_register(&RegisterName::Ip)?;
        self.set_register(&RegisterName::Ip, next_instruction_address + 2)?;

        let byte1 = *self
            .memory
            .borrow()
            .get(next_instruction_address as usize)
            .ok_or(CpuError::MemoryOutOfBounds)?;
        let byte2 = *self
            .memory
            .borrow()
            .get(next_instruction_address as usize + 1)
            .ok_or(CpuError::MemoryOutOfBounds)?;
        // let instruction = ( byte2 as u16) << 8 | (byte1 as u16); // Little-Endian Version
        let instruction = ((byte1 as u16) << 8) | (byte2 as u16); // Big-Endian Version

        Ok(instruction)
    }

    pub fn get_memory_16(&self, instruction_address: u16) -> Result<u16, CpuError> {
        let byte1 = *self
            .memory
            .borrow()
            .get(instruction_address as usize)
            .ok_or(CpuError::MemoryOutOfBounds)?;
        let byte2 = *self
            .memory
            .borrow()
            .get(instruction_address as usize + 1)
            .ok_or(CpuError::MemoryOutOfBounds)?;
        let instruction = ((byte1 as u16) << 8) | (byte2 as u16); // Big-Endian Version

        Ok(instruction)
    }

    fn set_memory16(&mut self, address: u16, value: u16) -> Result<(), CpuError> {
        let first_byte = (value & 0xFF) as u8;
        let second_byte = ((value >> 8) & 0xFF) as u8;

        self.memory.borrow_mut()[address as usize] = first_byte; // Big Edian way
        self.memory.borrow_mut()[(address + 1) as usize] = second_byte;

        Ok(())
    }

    fn fetch_register_index(&mut self) -> Result<u8, CpuError> {
        Ok(self.fetch()? % self.register_names.len() as u8)
    }

    fn get_register_name(&self, register_index: u8) -> Result<RegisterName, CpuError> {
        let register = self
            .register_names
            .get(register_index as usize)
            .cloned()
            .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
        Ok(register)
    }

    fn push(&mut self, value: u16) -> Result<(), CpuError> {
        let sp_address = *self.get_register(&RegisterName::Sp)?;
        self.set_memory16(sp_address, value)?;
        self.set_register(&RegisterName::Sp, sp_address - 2)?;
        self.stack_frame_size += 2;

        Ok(())
    }

    fn pop(&mut self) -> Result<u16, CpuError> {
        let next_sp_address = self.get_register(&RegisterName::Sp)? + 2;
        self.set_register(&RegisterName::Sp, next_sp_address)?;
        self.stack_frame_size -= 2;
        let value = self.get_memory_16(next_sp_address)?;
        Ok(value)
    }

    fn push_state(&mut self) -> Result<(), CpuError> {
        self.push(*self.get_register(&RegisterName::R1)?)?;
        self.push(*self.get_register(&RegisterName::R2)?)?;
        self.push(*self.get_register(&RegisterName::R3)?)?;
        self.push(*self.get_register(&RegisterName::R4)?)?;
        self.push(*self.get_register(&RegisterName::R5)?)?;
        self.push(*self.get_register(&RegisterName::R6)?)?;
        self.push(*self.get_register(&RegisterName::R7)?)?;
        self.push(*self.get_register(&RegisterName::R8)?)?;
        self.push(*self.get_register(&RegisterName::Ip)?)?;
        self.push(self.stack_frame_size + 2)?;

        let stack_pointer_value = *self.get_register(&RegisterName::Sp)?;
        self.set_register(&RegisterName::Fp, stack_pointer_value)?; // Move frame pointer to what stack currently points at
        self.stack_frame_size = 0; // So it can be tracked again
        Ok(())
    }

    fn pop_state(&mut self) -> Result<(), CpuError> {
        let frame_pointer_address = *self.get_register(&RegisterName::Fp)?;
        self.set_register(&RegisterName::Sp, frame_pointer_address)?;

        self.stack_frame_size = self.pop()?;
        let mut stack_frame_size = self.stack_frame_size;

        let ip_pop = self.pop()?;
        self.set_register(&RegisterName::Ip, ip_pop)?;
        let r8_pop = self.pop()?;
        self.set_register(&RegisterName::R8, r8_pop)?;
        let r7_pop = self.pop()?;
        self.set_register(&RegisterName::R7, r7_pop)?;
        let r6_pop = self.pop()?;
        self.set_register(&RegisterName::R6, r6_pop)?;
        let r5_pop = self.pop()?;
        self.set_register(&RegisterName::R5, r5_pop)?;
        let r4_pop = self.pop()?;
        self.set_register(&RegisterName::R4, r4_pop)?;
        let r3_pop = self.pop()?;
        self.set_register(&RegisterName::R3, r3_pop)?;
        let r2_pop = self.pop()?;
        self.set_register(&RegisterName::R2, r2_pop)?;
        let r1_pop = self.pop()?;
        self.set_register(&RegisterName::R1, r1_pop)?;

        let number_of_args = self.pop()?; // number of args passed to subroutine
        for _ in 0..number_of_args {
            self.pop()?;
        }
        stack_frame_size =  stack_frame_size >> 8; // I am doing this because somewhere I have the edianess mixed up
        println!("DEBUG: {}, {}",frame_pointer_address, stack_frame_size);
        self.set_register(&RegisterName::Fp, frame_pointer_address + stack_frame_size)?;

        // Set FP to current FP value plus step frame size

        Ok(())
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        match instruction {
            // Move literal value into r1 register. The literal will be the next 2 bytes in memory (16bit)
            Instruction::MoveLiteralToRegister => {
                let literal = self.fetch16()?;
                let register_index = self.fetch_register_index()?;
                let register = self
                    .register_names
                    .get(register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                self.set_register(&register, literal)?;
                Ok(())
            }
            // Move literal value into r2 register.
            Instruction::MoveRegisterToRegister => {
                let from_register_index = self.fetch_register_index()?;
                let from_register = self
                    .register_names
                    .get(from_register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                let from_value = *self.get_register(&from_register)?;

                let to_register_index = self.fetch_register_index()?;
                let to_register = self
                    .register_names
                    .get(to_register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;

                self.set_register(&to_register, from_value)?;
                Ok(())
            }
            // Add register to register
            Instruction::MoveRegisterToMemory => {
                let register_index = self.fetch_register_index()?;
                let register = self
                    .register_names
                    .get(register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                let register_value = *self.get_register(&register)?;

                let first_byte = (register_value & 0xFF) as u8;
                let second_byte = ((register_value >> 8) & 0xFF) as u8;

                let address = self.fetch16()? as usize;
                self.memory.borrow_mut()[address] = first_byte; // Big Edian way
                self.memory.borrow_mut()[address + 1] = second_byte;
                // self.memory.borrow_mut()[address] = second_byte; // Little Edian way
                // self.memory.borrow_mut()[address + 1] = first_byte;

                Ok(())
            }
            Instruction::MoveMemoryToRegister => {
                let address = self.fetch16()? as usize;

                let register_index = self.fetch_register_index()?;
                let register = self
                    .register_names
                    .get(register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;

                let address_byte1 = self.memory.borrow()[address];
                let address_byte2 = self.memory.borrow()[address + 1];

                // let combined_bytes = ((address_byte1 as u16) << 8) | address_byte2 as u16; // Big
                let combined_bytes = ((address_byte2 as u16) << 8) | address_byte1 as u16; // Little
                self.set_register(&register, combined_bytes)?;

                Ok(())
            }
            Instruction::AddRegisterToRegister => {
                let register1 = self.fetch_register_index()?;
                let register2 = self.fetch_register_index()?;
                let register_value1 = *self
                    .registers
                    .get((register1 as usize))
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                let register_value2 = *self
                    .registers
                    .get((register2 as usize))
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                self.set_register(&RegisterName::Acc, (register_value1 + register_value2))?;
                Ok(())
            }
            Instruction::JumpNotEq => {
                let value = self.fetch16()?;
                let address = self.fetch16()?;
                let register_value = *self.get_register(&RegisterName::Acc)?;
                if value != register_value {
                    self.set_register(&RegisterName::Ip, address)?;
                }

                Ok(())
            }
            Instruction::PushLiteral => {
                let value = self.fetch16()?;
                self.push(value)?;

                Ok(())
            }
            Instruction::PushRegister => {
                let register_index = self.fetch_register_index()?;
                let register_name = self.get_register_name(register_index)?;
                let register_value = self.get_register(&register_name)?;
                self.push(*register_value)?;
                Ok(())
            }
            Instruction::Pop => {
                let register_index = self.fetch_register_index()?;
                let value = self.pop()?;
                self.registers[register_index as usize] = value;
                Ok(())
            }
            Instruction::CallLiteral => {
                let address = self.fetch16()?;
                self.push_state()?;
                self.set_register(&RegisterName::Ip, address)?; // Set to subroutune process we need to jump to

                Ok(())
            }
            Instruction::CallRegister => {
                let register_index = self.fetch_register_index()?;
                let register_name = self.get_register_name(register_index)?;
                let address_value = *self.get_register(&register_name)?;
                self.push_state()?;
                self.set_register(&RegisterName::Ip, address_value)?;
                Ok(())
            }
            Instruction::Return => {
                self.pop_state()?;
                Ok(())
            }
        }
    }

    pub fn step(&mut self) -> Result<(), CpuError> {
        let instruction_as_byte = self.fetch()?;
        let instruction: Instruction = instruction_as_byte.try_into()?;
        self.execute(instruction)
    }
}
