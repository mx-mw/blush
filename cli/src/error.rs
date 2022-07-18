use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CLIError {
	ExternalError(String, String),
	UnkownArgument(String),
	InsufficientArguments,
	NotFound,
}

impl fmt::Display for CLIError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       fmt::Debug::fmt(&self, f)
    }
}


pub type CLIResult<O=()> = Result<O, CLIError>;
