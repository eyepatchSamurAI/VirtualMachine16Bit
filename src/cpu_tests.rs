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
        cpu.view_memory_at(*cpu.get_register(&RegisterName::Ip)? as usize, 8);
        cpu.view_memory_at(0xffff - 1 - 42, 44); // minus 6 more bytes so we can see the 8 bytes at beginning of stack

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
    const R3: u8 = 4;
    const R4: u8 = 5;
    const R5: u8 = 6;
    const R6: u8 = 7;
    const R7: u8 = 8;
    const R8: u8 = 9;
    const SP: u8 = 10;
    const FP: u8 = 11;

    let sub_routine_address = 0x3000;

    let shared_memory = create_memory(256*256);

    let mut i = 0;

    let mut cpu = Cpu::new(shared_memory.clone());
    { 
        // Cannot mutably borrow the same RefCell more than once in the same scope. This is disallowed by RefCell to ensure that the borrow rules are not violated at runtime.
        // That's why we have to create a new scope
        let mut writable_bytes = shared_memory.borrow_mut();
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x33, 0x33);
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x22, 0x22);
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x11, 0x11);
        write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x12, 0x34, R1);
        write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x56, 0x78, R4);
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x00, 0x00); // Number of args to sub routine
        let call_lit_param_1: u8 = ((sub_routine_address & 0xff00) >> 8) as u8;
        let call_lit_param_2: u8 = (sub_routine_address & 0x00ff) as u8;
        write_instruction!(writable_bytes, i, Instruction::CallLiteral.into(), call_lit_param_1, call_lit_param_2);
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x44, 0x44);

        i = sub_routine_address;
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x01, 0x02);
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x03, 0x04);
        write_instruction!(writable_bytes, i, Instruction::PushLiteral.into(), 0x05, 0x06);
        write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x07, 0x08, R1);
        write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x09, 0x0A, R8);
        write_instruction!(writable_bytes, i, Instruction::Return.into());
    }

    step_instruction_forever(&mut cpu)?;

    Ok(())

}

// Here is the block of code I used before in testing. This is just a refernce to what I have done before so I cen review and compare
// { 
//     // Cannot mutably borrow the same RefCell more than once in the same scope. This is disallowed by RefCell to ensure that the borrow rules are not violated at runtime.
//     // That's why we have to create a new scope
//     let mut writable_bytes = shared_memory.borrow_mut();
//     write_instruction!(writable_bytes, i, Instruction::MoveMemoryToRegister.into(), 0x01, 0x00, R1);
//     write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x00, 0x01, R2);
//     write_instruction!(writable_bytes, i, Instruction::AddRegisterToRegister.into(), R1, R2);
//     write_instruction!(writable_bytes, i, Instruction::MoveRegisterToMemory.into(), ACC, 0x01,0x00);
//     write_instruction!(writable_bytes, i, Instruction::JumpNotEq.into(), 0x00,0x03,0x00,0x00);
// }


// Push and pop from stack
// { 
//     // Cannot mutably borrow the same RefCell more than once in the same scope. This is disallowed by RefCell to ensure that the borrow rules are not violated at runtime.
//     // That's why we have to create a new scope
//     let mut writable_bytes = shared_memory.borrow_mut();
//     write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x51, 0x51, R1);
//     write_instruction!(writable_bytes, i, Instruction::MoveLiteralToRegister.into(), 0x42, 0x42, R2);
//     write_instruction!(writable_bytes, i, Instruction::PushRegister.into(), R1);
//     write_instruction!(writable_bytes, i, Instruction::PushRegister.into(), R2);
//     write_instruction!(writable_bytes, i, Instruction::Pop.into(), R1);
//     write_instruction!(writable_bytes, i, Instruction::Pop.into(), R2);
// }