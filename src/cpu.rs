use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{create_memory::{Memory, create_registers}, instructions::Instruction};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum RegisterName {
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
    InvalidInstruction
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
        let registers: Vec<u16> = create_registers(&register_names.len() * 2);
        let register_map = register_names
            .iter()
            .enumerate()
            .map(|(i, x)| (x.to_owned(), i * 2))
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

    fn get_register(&self, name: &RegisterName) -> Result<&u16, CpuError> {
        let register_index = self.register_map.get(&name).ok_or(CpuError::RegisterNameDoesNotExist)?;
        self.registers.get(*register_index).ok_or(CpuError::RegisterOutOfBounds)
    }
    fn set_register(&mut self, name: RegisterName, value: usize) -> Result<(), CpuError> {
        let register_index: &usize = self.register_map.get(&name).ok_or(CpuError::RegisterNameDoesNotExist)?;
        self.registers[*register_index] = value as u16;
        Ok(())
    }

    fn fetch(&mut self) -> Result<u8, CpuError> {
        let next_instruction_address = *self.get_register(&RegisterName::Ip)?;
        self.set_register(RegisterName::Ip, next_instruction_address as usize + 1)?;
        let instruction = self.memory.borrow().get(next_instruction_address as usize).copied().ok_or(CpuError::MemoryOutOfBounds);
        return instruction;
    }

    fn fetch16(&mut self) -> Result<u16, CpuError> {
        let next_instruction_address = *self.get_register(&RegisterName::Ip)?;
        self.set_register(RegisterName::Ip, next_instruction_address as usize + 2)?;

        let byte1 = *self.memory.borrow().get(next_instruction_address as usize).ok_or(CpuError::MemoryOutOfBounds)?;
        let byte2 = *self.memory.borrow().get(next_instruction_address as usize + 1).ok_or(CpuError::MemoryOutOfBounds)?;
        // let instruction = ( byte2 as u16) << 8 | (byte1 as u16); // Little-Endian Version
        let instruction = ((byte1 as u16) << 8) | (byte2 as u16); // Big-Endian Version

        Ok(instruction)
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        match instruction {
            // Move literal value into r1 register. The literal will be the next 2 bytes in memory (16bit)
            Instruction::MoveLiteralR1 => {
                let literal = self.fetch16()?;
                self.set_register(RegisterName::R1, literal as usize)?;
                Ok(())

            },
            // Move literal value into r2 register.
            Instruction::MoveLiteralR2 => {
                let literal = self.fetch16()?;
                self.set_register(RegisterName::R2, literal as usize)?;
                Ok(())

            },
            // Add register to register
            Instruction::AddRegToReg => {
                let register1 = self.fetch()?;
                let register2 = self.fetch()?;
                let register_value1 = *self.registers.get((register1 as usize) * 2 ).ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                let register_value2 = *self.registers.get((register2 as usize) * 2 ).ok_or_else(|| CpuError::RegisterOutOfBounds)?;
                self.set_register(RegisterName::Acc, (register_value1 + register_value2) as usize)?;
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
