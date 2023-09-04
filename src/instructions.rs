use crate::cpu::CpuError;

// $( ... )*: The * indicates zero or more repetitions of the pattern inside the parentheses. This allows the macro to accept any number of name => value pairs.

// $name:ident: This captures an identifier (like a variable name or enum variant) and stores it in a variable $name.

// =>: The literal "fat arrow" => syntax, which you are expected to include between the name and the value in each pair.

// $value:expr: This captures an expression and stores it in a variable $value.

// ,$(,)?: This allows for an optional trailing comma. The ? indicates zero or one occurrence of the preceding pattern (in this case, a comma ,).

// $name would capture MoveLiteralR1, MoveLiteralR2, and AddRegToReg.

// $value would capture 0x10, 0x11, and 0x12.
macro_rules! define_instruction {
    ( $( $name:ident => $value:expr ),* $(,)? ) => {
        #[derive(Debug, PartialEq, Eq)]
        pub enum Instruction {
            $( $name, )* // macro repetition
        }

        impl TryFrom<u8> for Instruction {
            type Error = CpuError;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $( $value => Ok(Instruction::$name), )*
                    _ => Err(CpuError::InvalidInstruction),
                }
            }

        }
        impl Into<u8> for Instruction {
            fn into(self) -> u8 {
                match self {
                    $( Instruction::$name => $value, )*
                }
            }
        }
    };
}

define_instruction! {
    MoveLiteralToRegister => 0x10,
    MoveRegisterToRegister => 0x11,
    MoveRegisterToMemory => 0x12,
    MoveMemoryToRegister => 0x13,
    AddRegisterToRegister => 0x14,
    JumpNotEq => 0x15,
}
