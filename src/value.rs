use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Value {
    VString(String),
    VNumber(f32),
    VBool(bool),
}

impl Default for Value {
	fn default() -> Self {
		Self::VBool(false)
	}
}
impl From<Value> for Vec<u8> {
    fn from(value: Value) -> Vec<u8> {
        bincode::serialize(&value).unwrap()
    }
}

use std::ops::*;

use crate::vm::{ArithmeticError, RuntimeError, RuntimeResult};

impl Add for Value {
    type Output = RuntimeResult<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Ok(Self::VNumber(n + r))
            } else {
                Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
            }
        } else {
            Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
        }
    }
}

impl Mul for Value {
    type Output = RuntimeResult<Self>;
    fn mul(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Ok(Self::VNumber(n * r))
            } else {
                Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
            }
        } else {
            Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
        }
    }
}

impl Sub for Value {
    type Output = RuntimeResult<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Ok(Self::VNumber(n - r))
            } else {
                Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
            }
        } else {
            Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
        }
    }
}

impl Div for Value {
    type Output = RuntimeResult<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Ok(Self::VNumber(n / r))
            } else {
                Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
            }
        } else {
            Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
        }
    }
}

impl Neg for Value {
    type Output = RuntimeResult<Self>;
    fn neg(self) -> Self::Output {
        if let Self::VNumber(n) = self {
            Ok(Value::VNumber(-n))
        } else {
            Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
        }
    }
}

impl Not for Value {
    type Output = RuntimeResult<Self>;
    fn not(self) -> Self::Output {
        if let Self::VBool(b) = self {
            Ok(Value::VBool(!b))
        } else {
            Err(RuntimeError::Arithmetic(ArithmeticError::TypeConflict))
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = other {
                n.partial_cmp(r)
            } else {
                None
            }
        } else {
            None
        }
    }
}
