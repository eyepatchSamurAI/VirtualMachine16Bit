use std::io;

use crate::{create_memory::create_memory, instructions::Instruction, cpu::{Cpu, CpuError, RegisterName}};

macro_rules! write_instruction {
    ($writable_bytes:ident, $i:ident, $($data:expr),* $(,)? ) => {
        $(
            $writable_bytes[$i] = $data;
            $i += 1;
        )*
    };
}

fn step_instruction_forever(cpu: &mut Cpu) ->Result<(), CpuError> {
    let stdin: io::Stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        cpu.debug();
        cpu.view_memory_at(*cpu.get_register(&RegisterName::Ip)? as usize);
        cpu.view_memory_at(0x0100);

        println!("Press any key to continue...");
        io::stdin().read_line(&mut "".to_string())
                   .ok()
                   .expect("Failed to read line");
        cpu.step()?;
    }
}

pub fn test_cpu() -> Result<(), CpuError>{
    // Easier Variables
    const IP: u8 = 0;
    const ACC: u8 = 1;
    const R1: u8 = 2;
    const R2: u8 = 3;

    let shared_memory = create_memory(256*256);

    let mut i = 0;

    let mut cpu = Cpu::new(shared_memory.clone());
    { 
        // Cannot mutably borrow the same RefCell more than once in the same scope. This is disallowed by RefCell to ensure that the borrow rules are not violated at runtime.
        // That's why we have to create a new scope
        let mut writable_bytes = shared_memory.borrow_mut();
        write_instruction!(writable_bytes, i, Instruction::MoveMemoryToRegister.into(), 0x01, 0x00, R1);
        write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x00, 0x01, R2);
        write_instruction!(writable_bytes, i, Instruction::AddRegisterToRegister.into(), R1, R2);
        write_instruction!(writable_bytes, i, Instruction::MoveRegisterToMemory.into(), ACC, 0x01,0x00);
        write_instruction!(writable_bytes, i, Instruction::JumpNotEq.into(), 0x00,0x03,0x00,0x00);
    }

    step_instruction_forever(&mut cpu)?;

    Ok(())

}