use std::slice::Iter;

use crate::{ZippedBag, BLUSH_VER, error::fileio::*, OpenedBag};

const BLUSHPROGRAM: &'static str = "BLUSHPROGRAM";
const PROGSTART: &'static str = "PROGSTART";
const PROGEND: &'static str = "PROGEND";


pub fn ser(input: Vec<ZippedBag>) -> Vec<u8> {
	let mut output = vec![];
	output.extend(format!("{}\n{}\n", BLUSHPROGRAM, BLUSH_VER).as_bytes()); // Blush program header
	output.push(input.len() as u8); // Number of bags to consume
	output.extend(format!("{}\n", PROGSTART).as_bytes()); // Indicate start of bytecode and constant declarations

	for i in input {
		output.push(i.bytes_len);
		output.push(i.consts_len);
		output.extend(i.bytecode);
		output.extend(i.constants);
	}

	output.extend(format!("\n{}", PROGEND).as_bytes()); // Indicate end of bytecode 

	output
}

pub fn de(input: Vec<u8>) -> FileIOResult<Vec<OpenedBag>> {
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
	Ok(bags)
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
    use crate::{Value, runtime::tests::util::make_bag};
	#[test]
	fn decode() {
		let v1 = Value::VNumber(33.2);
		let v1s: Vec<u8> = bincode::serialize(&v1).unwrap();
		assert!(bincode::deserialize::<Value>(&v1s).is_ok());
		let v2 = Value::VNumber(234.0);
		let v2s: Vec<u8> = bincode::serialize(&v2).unwrap();
		assert!(bincode::deserialize::<Value>(&v2s).is_ok());
		let mut constants = vec![];
		constants.extend(v1s);
		constants.extend(v2s);
		let instructions = vec![2u8, 234u8, 8u8, 34u8, 34u8];
		let bag = make_bag(instructions, constants);
		let binary = ser(vec![bag.clone()]);
		let res = de(binary);
		assert_eq!(res, Ok(vec![bag.unzip()]))
	}
}