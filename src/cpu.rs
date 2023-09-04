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
    register_map: HashMap<RegisterName, usize>,
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
        ];
        let registers: Vec<u16> = create_registers(register_names.len());
        let register_map = register_names
            .iter()
            .enumerate()
            .map(|(i, x)| (x.to_owned(), i))
            .collect::<HashMap<_, _>>();
        Cpu {
            memory,
            registers,
            register_names,
            register_map,
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

    pub fn view_memory_at(&self, address: usize) {
        let memory_ref = self.memory.borrow();
        let next_eight_bytes: Vec<String> = (0..8)
            .map(|index| {
                let byte = memory_ref.get(address + index).unwrap();
                format!("0x{:02x}", byte)
            })
            .collect();

        println!("0x{:04x}: {}", address, next_eight_bytes.join(" "));
    }

    pub fn get_register(&self, name: &RegisterName) -> Result<&u16, CpuError> {
        let register_index = self
            .register_map
            .get(&name)
            .ok_or(CpuError::RegisterNameDoesNotExist)?;
        self.registers
            .get(*register_index)
            .ok_or(CpuError::RegisterOutOfBounds)
    }
    fn set_register(&mut self, name: &RegisterName, value: usize) -> Result<(), CpuError> {
        let register_index: &usize = self
            .register_map
            .get(&name)
            .ok_or(CpuError::RegisterNameDoesNotExist)?;
        self.registers[*register_index] = value as u16;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u8, CpuError> {
        let next_instruction_address = *self.get_register(&RegisterName::Ip)?;
        self.set_register(&RegisterName::Ip, next_instruction_address as usize + 1)?;
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
        self.set_register(&RegisterName::Ip, next_instruction_address as usize + 2)?;

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

    fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        match instruction {
            // Move literal value into r1 register. The literal will be the next 2 bytes in memory (16bit)
            Instruction::MoveLiteralToRegister => {
                let literal = self.fetch16()?;
                let register_index = (self.fetch()? % self.register_names.len() as u8);
                let register = self
                    .register_names
                    .get(register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                self.set_register(&register, literal as usize)?;
                Ok(())
            }
            // Move literal value into r2 register.
            Instruction::MoveRegisterToRegister => {
                let from_register_index = (self.fetch()? % self.register_names.len() as u8);
                let from_register = self
                    .register_names
                    .get(from_register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;

                let to_register_index = (self.fetch()? % self.register_names.len() as u8);
                let to_register = self
                    .register_names
                    .get(to_register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                let from_value = *self.get_register(&from_register)?;

                self.set_register(&to_register, from_value as usize)?;
                Ok(())
            }
            // Add register to register
            Instruction::MoveRegisterToMemory => {
                let register_index = (self.fetch()? % self.register_names.len() as u8); // Multiple my 2 because reg can hold 2 bytes
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

                let register_index = (self.fetch()? % self.register_names.len() as u8); // Multiple my 2 because reg can hold 2 bytes
                let register = self
                    .register_names
                    .get(register_index as usize)
                    .cloned()
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;

                let address_byte1 = self.memory.borrow()[address];
                let address_byte2 = self.memory.borrow()[address + 1];

                // let combined_bytes = ((address_byte1 as u16) << 8) | address_byte2 as u16; // Big
                let combined_bytes = ((address_byte2 as u16) << 8) | address_byte1 as u16; // Little
                self.set_register(&register, combined_bytes as usize)?;

                Ok(())
            }
            Instruction::AddRegisterToRegister => {
                let register1 = self.fetch()?;
                let register2 = self.fetch()?;
                let register_value1 = *self
                    .registers
                    .get((register1 as usize))
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                let register_value2 = *self
                    .registers
                    .get((register2 as usize))
                    .ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                self.set_register(
                    &RegisterName::Acc,
                    (register_value1 + register_value2) as usize,
                )?;
                Ok(())
            }
            Instruction::JumpNotEq => {
                let value = self.fetch16()?;
                let address = self.fetch16()?;
                let register_value = *self.get_register(&RegisterName::Acc)?;
                if value != register_value {
                    self.set_register(&RegisterName::Ip, address as usize)?;
                }

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
