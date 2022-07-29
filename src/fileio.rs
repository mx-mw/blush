use std::slice::Iter;

use crate::{BLUSH_VER, error::fileio::*, OpenedBag, Compiler, runtime::{CompilerScope}};

const BLUSHPROGRAM: &'static str = "BLUSHPROGRAM";
const PROGSTART: &'static str = "PROGSTART";
const PROGEND: &'static str = "PROGEND";
const SCOPESTART: &'static str = "SCOPESTART";


pub fn ser(compiler: &Compiler) -> FileIOResult<Vec<u8>> {
	let baggage = compiler.clone().baggage;
	let mut output = vec![];
	output.extend(format!("{}\n{}\n", BLUSHPROGRAM, BLUSH_VER).as_bytes()); // Blush program header
	output.push(baggage.len() as u8); // Number of bags to consume
	output.extend(format!("{}\n", PROGSTART).as_bytes()); // Indicate start of bytecode and constant declarations

	for i in baggage {
		output.push(i.bytes_len);
		output.push(i.consts_len);
		output.extend(i.bytecode);
		output.extend(i.constants);
	}

	output.extend(format!("\n{}", PROGEND).as_bytes()); // Indicate end of bytecode 
	output.extend(format!("\n{}\n", SCOPESTART).as_bytes()); // Indicate start of scope encoding
	let scope_bytes = match bincode::serialize(&compiler.scope) {
		Ok(b) => b,
		Err(e) => return Err(FileIOError::ExternalError("bincode::ErrorKind".into(), e.to_string()))
	};
	output.extend(scope_bytes);

	Ok(output)
}

pub fn de(input: Vec<u8>) -> FileIOResult<(Vec<OpenedBag>, CompilerScope)> {
	let mut input = input.iter();
	consume(
		&mut input, 
		format!("{}\n{}\n", BLUSHPROGRAM, BLUSH_VER).as_str(),
		MalformedHeaderError::BlushProgramDecl
	)?;

	let num_bags = input.next().ok_or(FileIOError::MalformedHeader(MalformedHeaderError::NumBags))?;
	consume(&mut input, format!("{}\n", PROGSTART).as_str(), MalformedHeaderError::ProgEnd)?;
	
	let mut bags = Vec::<OpenedBag>::new();
	for _ in 0..(*num_bags) as usize {
		let eof = FileIOError::MalformedBytecode(MalformedBytecodeError::UnexpectedEof);
		let missing_len_dec = FileIOError::MalformedBytecode(MalformedBytecodeError::MissingLenghtDecl);
		let bytes_len = *input.next().ok_or(missing_len_dec.clone())? as usize;
		let consts_len = *input.next().ok_or(missing_len_dec)? as usize;
		let mut bytecode = vec![];
		for _ in 0..bytes_len {
			bytecode.push(*input.next().ok_or(eof.clone())?);
		}
		for _ in 0..(u8::MAX as usize) - bytes_len {
			input.next().ok_or(eof.clone())?;
		}

		let mut constants = vec![];
		for _ in 0..consts_len {
			constants.push(*input.next().ok_or(eof.clone())?);
		}
		for _ in 0..(u8::MAX as usize) - consts_len {
			input.next().ok_or(eof.clone())?;
		}
		bags.push(OpenedBag {
			bytecode,
			constants
		})
	}
	consume(&mut input, format!("\n{}", PROGEND).as_str(), MalformedHeaderError::ProgEnd)?;
	consume(&mut input, format!("\n{}\n", SCOPESTART).as_str(), MalformedHeaderError::ScopeStart)?;
	let scope: CompilerScope = bincode::deserialize(input.as_slice()).unwrap();
	Ok((bags, scope))
}

fn consume(input: &mut Iter<u8>, expected: &str, kind: MalformedHeaderError) -> FileIOResult<()> {
	for i in expected.as_bytes() {
		if input.next() != Some(i) {
			return Err(FileIOError::MalformedHeader(kind))
		}
	}

	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn decode() {
		let mut compiler = Compiler::new("1 + 1;");
		compiler.compile().unwrap();
		let binary = ser(&compiler).unwrap();
		let res = de(binary);
		assert_eq!(res, Ok((vec![compiler.baggage[0].unzip()], compiler.scope)))
	}
}