use crate::{Instruction, Value};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
	constants: Vec<u8>,
	bytecode: Vec<u8>,
	num_constants: u8,
	num_bytes: u8
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SealedBlock {
	pub constants: Vec<u8>,
	pub bytecode: Vec<u8>,
}

impl Block {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn emit_byte(&mut self, instruction: Instruction, arguments: &Vec<u8>) -> Result<(), ()> {
		let num_instructions = 1 + arguments.len();
		if self.check_bytes_length(num_instructions) {
			return Err(())
		}
		self.bytecode.push(instruction as u8);
		self.bytecode.extend(arguments);
		self.num_bytes += num_instructions as u8;
		Ok(())
	}

	pub fn check_consts_length(&self, increase: usize) -> bool {
		(self.num_constants as usize + increase) >= (u8::MAX as usize)
	}

	pub fn check_bytes_length(&self, increase: usize) -> bool {
		(self.num_bytes as usize + increase) >= (u8::MAX as usize)
	}

	pub fn emit_const(&mut self, value: &Value, store: u8) -> Result<(), ()> {
        // Get the index of the first byte in the instruction
        let idx = self.constants.len();
        let serialized = bincode::serialize(&value).unwrap();
        let len = serialized.len();
		// Make sure the chunk does not grow too big
		if self.check_consts_length(len) || self.check_bytes_length(4) {
			return Err(())
		}
        self.constants.extend(serialized);
        self.bytecode
            .extend(vec![Instruction::Const as u8, store, len as u8, idx as u8]);
        Ok(())
	}

	// Clean up step implemented now in case extra data needs to be computed
	pub fn seal(self) -> SealedBlock {
		SealedBlock { constants: self.constants, bytecode: self.bytecode }
	}
}

#[test]
fn emit_const_overflow() {
	let mut block = Block::new();
	block.num_bytes = u8::MAX-1;
	assert!(block.emit_const(&Value::VBool(false), 0).is_err());

}
