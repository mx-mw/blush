use bincode::{config, config::Configuration, Decode, Encode};
const CONFIG: Configuration = config::standard();

#[derive(Decode, Encode, Debug, PartialEq, Clone)]
pub enum Value {
    VString(String),
    VNumber(f32),
    VBool(bool),
}

impl From<Value> for Vec<u8> {
    fn from(value: Value) -> Vec<u8> {
        bincode::encode_to_vec(value, CONFIG).unwrap()
    }
}

use std::ops::*;

impl Add for Value {
    type Output = Option<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Some(Self::VNumber(n + r))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Mul for Value {
    type Output = Option<Self>;
    fn mul(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Some(Self::VNumber(n * r))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Sub for Value {
    type Output = Option<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Some(Self::VNumber(n - r))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Div for Value {
    type Output = Option<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        if let Self::VNumber(n) = self {
            if let Self::VNumber(r) = rhs {
                Some(Self::VNumber(n / r))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Neg for Value {
    type Output = Option<Self>;
    fn neg(self) -> Self::Output {
        if let Self::VNumber(n) = self {
            Some(Value::VNumber(-n))
        } else {
            None
        }
    }
}

impl Not for Value {
    type Output = Option<Self>;
    fn not(self) -> Self::Output {
        if let Self::VBool(b) = self {
            Some(Value::VBool(!b))
        } else {
            None
        }
    }
}
