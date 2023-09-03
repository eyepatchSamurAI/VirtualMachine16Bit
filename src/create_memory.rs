use std::{cell::RefCell, rc::Rc};

pub type Memory = Vec<u8>;
pub type Registers = Vec<u16>;

pub fn create_memory(size: usize) -> Rc<RefCell<Memory>> {
    Rc::new(RefCell::new(vec![0; size]))
}
pub fn create_registers(size_in_bytes: usize) -> Registers {
    vec![0u16; size_in_bytes]
}