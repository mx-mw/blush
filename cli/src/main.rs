mod error;
use error::*;
use std::{fs::{OpenOptions, read, read_to_string}, io::{ErrorKind, Write}};
use logos::Logos;
use blush::{Compiler, TokenKind, fileio};

fn main() -> CLIResult {
	let mut args= std::env::args();
	args.next(); // Ignore program name
	match args.next() {
		Some(arg) => {
			if &arg == "build" {
				let file = args.next().ok_or(CLIError::InsufficientArguments)?;
				let code = match read_to_string(&file) {
					Ok(s) => s,
					Err(e) => match e.kind() {
						ErrorKind::NotFound => return Err(CLIError::NotFound),
						_ => return Err(CLIError::ExternalError("io::Error".into(), e.to_string()))
					}
				};
				let lexer = TokenKind::lexer(&code);
				let mut compiler = Compiler {
					lexer,
					..Default::default()
				};
			
				match compiler.compile() {
					Ok(_) => {}
					Err(e) => return Err(CLIError::ExternalError("CompilerError".into(), e.to_string()))
				}
			
				let code = fileio::ser(&compiler).unwrap();
				let (save_as, _) = file.rsplit_once(".").unwrap_or((&file, ""));
				let mut file = OpenOptions::new()
					.write(true)
					.append(false)
					.create(true)
					.open(format!("{}.blc", save_as))
					.unwrap();
				file.write_all(&code).unwrap();
				Ok(())
			} else if &arg == "run" {
				let file = args.next().ok_or(CLIError::InsufficientArguments)?;
				let bytecode = match read(&file) {
					Ok(b) => b,
					Err(e) => match e.kind() {
						ErrorKind::NotFound => return Err(CLIError::NotFound),
						_ => return Err(CLIError::ExternalError("io::Error".into(), e.to_string()))
					}
				};

				let (bags, scope) = match fileio::de(bytecode) {
					Ok(res) => res,
					Err(e) => return Err(CLIError::ExternalError("FileIOError".to_string(), e.to_string()))
				};

				let mut runtime = blush::Runtime::new(bags, None, scope);
				runtime.exec().unwrap();
				Ok(())
			} else {
				Err(CLIError::UnkownArgument(arg))
			}
			
		}

		None => Err(CLIError::InsufficientArguments)
	}
}