#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    Bytecode(BytecodeError),
    Arithmetic(ArithmeticError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError {
    TypeConflict,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BytecodeError {
    Malformed(Vec<u8>, usize),
}

macro_rules! malformed_bytecode {
    ($i:expr, $p:expr) => {
        Err(RuntimeError::Bytecode(BytecodeError::Malformed(
            $i.clone(),
            $p,
        )))
    };
}

pub(crate) use malformed_bytecode;

pub type RuntimeResult<T> = Result<T, RuntimeError>;
