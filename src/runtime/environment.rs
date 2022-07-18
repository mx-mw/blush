use crate::Value;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Variable {
    pub name: String,
    pub value: Value,
    pub depth: u8,
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct Local {
    pub name: String,
    pub depth: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct RawScope<T: Default> {
    pub vars: Vec<T>,
    pub num_vars: u8,
    pub depth: u8,
}

pub type CompilerScope = RawScope<Local>;
pub type RuntimeScope = RawScope<Variable>;

impl From<CompilerScope> for RuntimeScope {
    fn from(cs: CompilerScope) -> Self {
        return RawScope {
            vars: cs
                .vars
                .iter()
                .map(|i| Variable {
                    name: i.name.clone(),
                    depth: i.depth,
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
            num_vars: cs.num_vars,
            depth: 0,
        };
    }
}
