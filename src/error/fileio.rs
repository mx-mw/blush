use std::fmt;
use super::BlushError;

#[derive(Debug, Clone, PartialEq)]
pub enum FileIOError {
	ExternalError(String, String),
	MalformedBytecode(MalformedBytecodeError),
	MalformedHeader(MalformedHeaderError)
}

#[derive(Debug, Clone, PartialEq)]
pub enum MalformedBytecodeError {
	ValueDeser(String),
	MissingLenghtDecl, 
	UnexpectedEof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MalformedHeaderError {
	BlushProgramDecl,
	ProgStart,
	ProgEnd,
	NumBags,
}

impl fmt::Display for FileIOError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       fmt::Debug::fmt(&self, f)
    }
}

impl BlushError for FileIOError { }

pub type FileIOResult<O=()> = Result<O, FileIOError>;
