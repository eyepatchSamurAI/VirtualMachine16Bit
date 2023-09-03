use crate::{create_memory::create_memory, instructions::Instruction, cpu::Cpu};

pub fn test_cpu() {
    let shared_memory = create_memory(256);

    let mut cpu = Cpu::new(shared_memory.clone());
    { 
        // Cannot mutably borrow the same RefCell more than once in the same scope. This is disallowed by RefCell to ensure that the borrow rules are not violated at runtime.
        // That's why we have to create a new scope
        let mut writable_bytes = shared_memory.borrow_mut();
        writable_bytes[0] = Instruction::MoveLiteralR1.into();
        writable_bytes[1] = 0x12;
        writable_bytes[2] = 0x34;
    
        writable_bytes[3] = Instruction::MoveLiteralR2.into();
        writable_bytes[4] = 0xAB;
        writable_bytes[5] = 0xCD;
    
        writable_bytes[6] = Instruction::AddRegToReg.into();
        writable_bytes[7] = 2; // r1 index
        writable_bytes[8] = 3; // r2 index
    }

    cpu.debug();

    cpu.step();
    cpu.debug();

    cpu.step();
    cpu.debug();

    cpu.step();
    cpu.debug();

}