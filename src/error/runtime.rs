use std::fmt;
use super::BlushError;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    ExternalError(String, String),
    Bytecode(BytecodeError),
    Arithmetic(ArithmeticError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError {
    TypeConflict,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BytecodeError {
    Malformed(Vec<u8>, usize, &'static str),
}

impl BlushError for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

macro_rules! malformed_bytecode {
    ($i:expr, $p:expr, $why:expr) => {
        Err(RuntimeError::Bytecode(BytecodeError::Malformed(
            $i.clone(),
            $p,
            $why,
        )))
    };
}

pub(crate) use malformed_bytecode;

pub type RuntimeResult<T=()> = Result<T, RuntimeError>;
