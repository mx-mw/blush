use crate::{Instruction, Value, error::bag::*};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Bag {
	pub constants: Vec<u8>,
	pub bytecode: Vec<u8>,
	num_constants: usize,
	num_bytes: usize
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZippedBag {
	pub constants: [u8;u8::MAX as usize],
	pub bytecode: [u8;u8::MAX as usize],
	pub consts_len: u8,
	pub bytes_len: u8,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct OpenedBag {
	pub constants: Vec<u8>,
	pub bytecode: Vec<u8>,
}

impl Default for ZippedBag {
	fn default() -> Self {
		Self { constants: [0;u8::MAX as usize], bytecode: [0;u8::MAX as usize], consts_len: 0, bytes_len: 0 }
	}
}

impl ZippedBag {
	pub fn unzip(&self) -> OpenedBag {
		let constants = self.constants[0..self.consts_len as usize].to_vec();
		let bytecode = self.bytecode[0..self.bytes_len as usize].to_vec();
		OpenedBag { constants, bytecode }
	}
}

impl From<Bag> for ZippedBag {
	fn from(b: Bag) -> Self {
		b.zip_up()
	}
}

impl From<ZippedBag> for OpenedBag {
	fn from(z: ZippedBag) -> Self {
		z.unzip()
	}
}

impl Bag {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn emit_byte(&mut self, instruction: Instruction, arguments: &Vec<u8>) -> BagResult {
		let num_instructions = 1 + arguments.len();
		self.check_length((num_instructions, 0))?;
		self.bytecode.push(instruction as u8);
		self.bytecode.extend(arguments);
		self.num_bytes += num_instructions;
		Ok(())
	}

	pub fn check_length(&mut self, increase: (usize, usize)) -> BagResult {
		let bytecode = (self.num_bytes as usize + increase.0) >= (u8::MAX as usize);
		let constants = (self.num_constants as usize + increase.1) >= (u8::MAX as usize);
		let culprit = if bytecode {
			BagItem::Bytecode
		} else if constants {
			BagItem::Constants
		} else {
			// If neither bytecode or constants overflow, this value won't be used.
			BagItem::Both
		};

		if constants || bytecode {
			Err(BagError::Full(culprit))
		} else {
			Ok(())
		}
	}

	pub fn emit_const(&mut self, value: &Value, store: u8) -> BagResult {
        // Get the index of the first byte in the instruction
        let idx = self.constants.len();
        let serialized = bincode::serialize(&value).unwrap();
        let len = serialized.len();

        self.constants.extend(serialized);
		self.num_constants += len;
        self.emit_byte(Instruction::Const, &vec![store, len as u8, idx as u8])?;
        Ok(())
	}

	pub fn populate(&mut self, bytecode: Vec<u8>, constants: Vec<u8>) -> BagResult {
		self.check_length((bytecode.len(), constants.len()))?;
		self.bytecode.extend(&bytecode);
		self.constants.extend(&constants);
		self.num_bytes += bytecode.len();
		self.num_constants += constants.len();
		Ok(())
	}

	// Clean up step implemented now in case extra data needs to be computed
	pub fn zip_up(self) -> ZippedBag {
		let mut _self = self.clone();
		_self.bytecode.extend(vec![0;u8::MAX as usize - _self.bytecode.len()]);
		_self.constants.extend(vec![0;u8::MAX as usize - _self.constants.len()]);
		let constants: [u8;u8::MAX as usize] = _self.constants.clone().try_into().unwrap();
		let bytecode: [u8;u8::MAX as usize] = _self.bytecode.clone().try_into().unwrap();
		ZippedBag { constants, bytecode, consts_len: self.num_constants as u8, bytes_len: self.num_bytes as u8 }
	}
}

#[test]
fn emit_overflow() {
	let mut block = Bag::new();
	block.populate(vec![0; (u8::MAX - 1) as usize], vec![0; (u8::MAX - 1) as usize]).unwrap();
	
	assert!(block.emit_byte(Instruction::Add, &vec![10u8, 100u8, 80u8]).is_err());
}
