use super::BlushError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum BagError {
    ExternalError(String, String),
    Full(BagItem),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BagItem {
    Bytecode,
    Constants,
    Both,
}

impl fmt::Display for BagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl BlushError for BagError {}

pub type BagResult<O = ()> = Result<O, BagError>;
