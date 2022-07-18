/*
    # Virtual Machine
    Code is executed by matching bytes to instructions inside of a loop. The loop consumes the next byte,
    then executes the instruction (if the byte is not a valid instruction, an error is reported).
	
    ## Bytecode Execution
	Bytecode instructions are matched to a function which consumes later bytes as arguments.
	Ex. Binary arithmetic instructions have 3 arguments in the 3 following bytes
*/

use crate::{Value, ZippedBag, OpenedBag, error::runtime::*};

mod environment;
pub use environment::*;


pub struct Runtime {
	pub scope: RuntimeScope,
    pub ic: usize,
	pub compiler_scope: CompilerScope,
    pub registers: Vec<Value>,
	pub baggage: Vec<OpenedBag>,
	pub current_bag: usize,
}

macro_rules! operation {
	($self:ident.$op:tt, A) => {{ // Arithmetic
		let lhs = $self.at_next()?;
		let rhs = $self.at_next()?;
		$self.set_next((lhs $op rhs)?)?;
		Ok(())
	}};

	($self:ident.$op:tt, C) => {{ // Comparison
		let lhs = $self.at_next()?;
		let rhs = $self.at_next()?;
		if lhs $op rhs {
			$self.ic+=2;
		}
		Ok(())
	}};

	($self:ident.$op:tt, U) => {{ // Unary
		let idx = $self.next()?;
		let value = $self.registers[idx as usize].clone();
		$self.set(idx, ($op value)?);
		Ok(())
	}}
}

impl Runtime {
    pub fn new(baggage: Vec<ZippedBag>, scope: Option<RuntimeScope>, compiler_scope: CompilerScope) -> RuntimeResult<Self> {
        Ok(Self {
			scope: scope.unwrap_or(compiler_scope.clone().into()),
			compiler_scope,
            ic: 0,
            registers: vec![Value::VBool(false); u8::MAX.into()],
			baggage: baggage.into_iter().map(|i| i.unzip()).collect(),
			current_bag: 0,
        })
    }

    pub fn exec(&mut self) -> RuntimeResult {
        loop {
            let current: u8 = self.current();
            match current {
				0  /*Const*/ => {self.constant()?;}
				1  /*Add*/   => {self.add()?;}
				2  /*Sub*/   => {self.sub()?;}
				3  /*Mul*/   => {self.mul()?;}
				4  /*Div*/   => {self.div()?;}
				5  /*Eq*/    => {self.eq()?;} 
				6  /*Ne*/    => {self.ne()?;}
				7  /*Lt*/    => {self.lt()?;}
				8  /*Le*/    => {self.le()?;}
				9  /*Not*/   => {self.not()?;}
				10 /*Neg*/   => {self.neg()?;}
				11 /*Let*/   => {self.let_declr()?;}
				12 /*Read*/  => {}
				13 /*Set*/   => {}
				14 /*Move*/  => {self.ic = self.next()? as usize;}
				_ => return malformed_bytecode!(self.bytecode(), self.ic, "Unexpected byte")
			}
            self.ic += 1;
            if self.ic == self.bytecode().len() {
                break;
            }
        }
        Ok(())
    }

    fn next(&mut self) -> RuntimeResult<u8> {
        self.ic += 1;
        if self.ic >= u8::MAX as usize {
            malformed_bytecode!(self.bytecode(), self.ic, "Block is too large")
        } else {
            Ok(self.current())
        }
    }

	fn current_block(&self) -> &OpenedBag {
		&self.baggage[self.current_bag]
	}
	fn bytecode(&self) -> &Vec<u8> {
		&self.current_block().bytecode
	}

	fn constants(&self) -> &Vec<u8> {
		&self.current_block().constants
	}

	fn current(&self) -> u8 {
		self.bytecode()[self.ic]
	}

    fn at_next(&mut self) -> RuntimeResult<Value> {
        let idx = self.next()? as usize;
        Ok(self.registers[idx].clone())
    }

    fn set(&mut self, idx: u8, value: Value) {
        self.registers[idx as usize] = value;
    }

    fn set_next(&mut self, value: Value) -> RuntimeResult {
        let idx = self.next()?;
        self.set(idx, value);
		Ok(())
    }

    pub fn constant(&mut self) -> RuntimeResult {
        let idx = self.next()?;
        let len = self.next()?;
        let data = self.constants()[(idx as usize)..(idx + len) as usize].to_vec();
        let value: Value = bincode::deserialize(&data).unwrap();
        self.set_next(value)?;
        Ok(())
    }

    pub fn add(&mut self) -> RuntimeResult {
        operation!(self.+, A)
    }

    pub fn sub(&mut self) -> RuntimeResult {
        operation!(self.-, A)
    }

    pub fn mul(&mut self) -> RuntimeResult {
        operation!(self.*, A)
    }

    pub fn div(&mut self) -> RuntimeResult {
        operation!(self./, A)
    }

    pub fn eq(&mut self) -> RuntimeResult {
        operation!(self.==, C)
    }

    pub fn ne(&mut self) -> RuntimeResult {
        operation!(self.!=, C)
    }

    pub fn lt(&mut self) -> RuntimeResult {
        operation!(self.<, C)
    }

    pub fn le(&mut self) -> RuntimeResult {
        operation!(self.<=, C)
    }

    pub fn not(&mut self) -> RuntimeResult {
        operation!(self.!, U)
    }

    pub fn neg(&mut self) -> RuntimeResult {
        operation!(self.-, U)
    }
	
	pub fn let_declr(&mut self) -> RuntimeResult { // 11 LET   L A    Vv(L) = R(A)
		let local_idx = self.next()?;
		let v = self.at_next()?;
		self.scope.vars[local_idx as usize].value = v;
		Ok(())
	}
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
	use crate::{runtime::Variable, Bag, ZippedBag};
    pub(crate) mod util {
        use super::*;

		pub fn make_bag(bytecode: Vec<u8>, constants: Vec<u8>) -> ZippedBag {
			let mut bag = Bag::new();
			bag.populate(bytecode, constants).unwrap();
			bag.zip_up()
		}
        pub fn runtime(baggage: Vec<ZippedBag>, scope: Option<CompilerScope>) -> Runtime {
            let mut runtime = Runtime::new(
				baggage,
				None, 
				scope.unwrap_or(CompilerScope::default())
			).unwrap();
            runtime.exec().unwrap();
            runtime
        }

        #[macro_export]
        macro_rules! binop_test {
            ($op:tt, $i:expr) => {
                let v1 = Value::VNumber(33.2);
                let v1s: Vec<u8> = bincode::serialize(&v1).unwrap();
				assert!(bincode::deserialize::<Value>(&v1s).is_ok());
                let v2 = Value::VNumber(234.0);
                let v2s: Vec<u8> = bincode::serialize(&v2).unwrap();
				assert!(bincode::deserialize::<Value>(&v2s).is_ok());
                let mut constants = vec![];
                constants.extend(v1s.clone());
                constants.extend(v2s.clone());
                let instructions = vec![
                    Instruction::Const as u8, 0, v1s.len() as u8, 0,
                    Instruction::Const as u8, v1s.len() as u8, v2s.len() as u8, 1,
                    $i as u8, 0, 1, 2,
                ];
                let v1v = bincode::deserialize::<Value>(&constants[0..v1s.len()]);
                let v2v =
                    bincode::deserialize::<Value>(&constants[v1s.len()..v1s.len() + v2s.len()]);
                assert!(v1v.is_ok());
                assert!(v2v.is_ok());
				let bag = util::make_bag(instructions, constants);
                let runtime = runtime(vec![bag], None);
                let mut registers = vec![Value::VBool(false); u8::MAX.into()];
                registers[0] = v1.clone();
                registers[1] = v2.clone();
                registers[2] = (v1 $op v2).unwrap();
                assert_eq!(runtime.registers, registers)
            };
        }
    }
    use crate::{binop_test, Instruction};
    use util::*;

    #[test]
    fn add() {
        binop_test!(+, Instruction::Add);
    }

    #[test]
    fn sub() {
        binop_test!(-, Instruction::Sub);
    }

    #[test]
    fn div() {
        binop_test!(/, Instruction::Div);
    }

    #[test]
    fn mul() {
        binop_test!(*, Instruction::Mul);
    }

	#[test]
	fn let_declr() {
		let v1 = Value::VNumber(33.2);
        let v1s: Vec<u8> = bincode::serialize(&v1).unwrap();
		let scope = RawScope {
			depth: 0,
			num_vars: 1,
			vars: vec![
				Local {
					depth: 0,
					name: "asdf".into()
				}
			]
		};
		let bag = make_bag(vec![
			0, 0, v1s.len() as u8, 0,
			11, 0, 0
		], v1s);
		let runtime = runtime(vec![bag], Some(scope));
		
		assert_eq!(runtime.scope, RuntimeScope {
			depth: 0,
			num_vars: 1,
			vars: vec![
				Variable {
					depth: 0,
					name: "asdf".into(),
					value: v1
				}
			]
		})
	}
}
