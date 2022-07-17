mod bag;
mod compiler;
mod instruction;
mod scanner;
mod value;
mod runtime;
mod error;

pub use bag::*;
pub use compiler::Compiler;
pub use instruction::Instruction;
pub use value::Value;
pub use runtime::Runtime;
// pub use vm::{Environment, VM};
pub use scanner::TokenKind;

use logos::Logos;

pub fn run(source: &str) {
    let mut compiler = Compiler {
        lexer: TokenKind::lexer(source),
        ..Default::default()
    };
    compiler.compile().unwrap();
}
