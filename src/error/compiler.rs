use std::fmt;
use super::BlushError;
use crate::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerError {
	ExternalError(String, String),
	TokenError(TokenError),
	RegisterError(RegisterError)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenError {
	ExpectedToken {
		reason: &'static str,
		expected: Option<TokenKind>,
		recieved: Option<TokenKind>
	},
	EarlyEof
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterError {
	NoEmptyRegisters
}

impl BlushError for CompilerError {}

impl fmt::Display for CompilerError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       fmt::Debug::fmt(&self, f)
    }
}

pub type CompilerResult<O=()> = Result<O, CompilerError>;
