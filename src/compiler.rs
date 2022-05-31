use crate::{instruction::Instruction, scanner::TokenKind, value::Value};
use logos::{Lexer, Logos};
pub type CompileResult<T> = Result<T, (CompileError, String)>;

#[derive(Debug)]
pub enum CompileError {
    TokenError, // A token was not in the correct position
    RegisterError, // Any error involving registers
}

macro_rules! compile_error {
    ($kind:expr, $str:tt, $($arg:tt)*) => ({
        eprintln!("Compile Error: {} @ {}", format!($str, $($arg)*), std::line!());
        return Err(($kind, format!($str, $($arg)*)));
    });
    ($kind:expr, $str:tt) => ({
        eprintln!("Compile Error: {} @ {}", $str, std::line!());
        return Err(($kind, $str.to_string()));
    });
}

#[derive(Clone)]
pub struct Compiler<'src> {
    pub lexer: Lexer<'src, TokenKind>,
    pub instructions: Vec<u8>,
    pub constants: Vec<u8>,
    pub registers: Vec<u8>,
    pub previous: Option<TokenKind>,
    pub current: Option<TokenKind>,
}

impl Default for Compiler<'_> {
    fn default() -> Self {
        Self {
            lexer: TokenKind::lexer(""),
            instructions: vec![],
            constants: vec![],
            registers: (0..16).collect(),
            previous: None,
            current: None,
        }
    }
}

impl<'s> Compiler<'s> {
    /// Take the array of tokens and generate bytecode
    pub fn compile(&mut self) -> CompileResult<()> {
        self.expression()?;
        self.consume(Some(TokenKind::Semicolon), "Expected ';' at end of expression")?;
        // self.consume(None, "Expected end of expression")?;
        Ok(())
    }

    /// Wrapper around [Logos::Lexer::next]
    /// Consume and return the next token if it exists
    fn next(&mut self) -> Option<TokenKind> {
        self.previous = self.current.clone();
        self.current = self.lexer.next();

        self.current.clone()
    }

    fn previous(&self) -> Option<TokenKind> {
        self.previous.clone()
    }

    /// Consume a token and expect to equal `kind`
    /// If it did not match, throw an error with `why` as the message
    fn consume(&mut self, kind: Option<TokenKind>, why: &'static str) -> CompileResult<()> {
        match self.next() {
            // Match kind
            k if k == kind => { Ok(()) }
            // Give a slightly more verbose error showing the token it saw
            Some(k) => compile_error!(CompileError::TokenError,
                                                "{} (Expected {:?}; got {:?})", why, kind, k),
            // None was not expected
            None => compile_error!(CompileError::TokenError, "{} (Expected {:?})", why, kind),
        }
    }

    /// Get the next token without advancing
    fn peek(&self) -> Option<TokenKind> {

        self.lexer.clone().peekable().peek().cloned()
    }

    /// Get the next available register to store a value in
    fn use_register(&mut self) -> CompileResult<u8> {
        if self.registers.is_empty() {
            compile_error!(CompileError::RegisterError, "No empty registers")
        } else {
            Ok(self.registers.remove(0))
        }
    }

    /// Free a register
    fn free_register(&mut self, register: u8) {
        self.registers.push(register)
    }

    /// Emit an [Instruction] and it's arguments
    /// Converts an [Instruction] to a u8, and pushes it along with it's arguments onto the end of the
    /// instructions vector
    fn emit_byte(&mut self, instruction: Instruction, arguments: Vec<u8>) {
        // Push the instruction as a byte onto the vec
        self.instructions.push(instruction as u8);
        // Extend the `instructions` vec with the `arguments` vec
        self.instructions.extend(arguments)
    }

    /// Store a constant value and append the appropriate bytes to the bytecode
    /// Specifically, encode the value as bytes and append those to the constants vector, then emit
    /// a [Instruction::Const] and the starting index of the vector
    fn emit_const(&mut self, value: Value) -> (usize, usize) {
        // Get the index of the first byte of the value
        let idx = self.constants.len();
        // Convert the value to a byte
        let value: Vec<u8> = value.into();
        // Emit the byte with the starting index and the length of the value as the arguments
        self.emit_byte(
            Instruction::Const,
            vec![idx as u8, value.len() as u8],
        );
        // Append the byteified value onto the `constants` vec
        self.constants.extend(value.clone());
        (idx, value.len())
    }

    /// Write a const and load it into a register
    /// Wraps [Compiler::emit_const]
    fn store_const(&mut self, value: Value) -> CompileResult<u8> {
        let (idx, len) = self.emit_const(value);
        let store = self.use_register()?;
        self.emit_byte(Instruction::Load, vec![idx as u8, len as u8, store]);
        Ok(store)
    }

    /// Check if the next token is expected
    fn tag(&mut self, expected: &TokenKind) -> bool {
        if self.peek() == Some(expected.clone()) {
            self.next();
            true
        } else {
            false
        }
    }

    /// Wrapper around `tag` for multiple values of `expected`
    fn tag_any(&mut self, expected: Vec<TokenKind>) -> Option<usize> {
        for (idx, token) in expected.iter().enumerate() {
            if self.tag(token) {
                return Some(idx)
            }
        }
        None
    }

    /// Parse a grouping (stuff in parentheses) expression
    fn grouping(&mut self) -> CompileResult<u8> {
        let idx = self.expression()?;
        self.consume(
            Some(TokenKind::RightParen),
            "Expected ')' following expression."
        )?;
        Ok(idx)
    }

    /// Parse expressions and generate bytecode
    /// Root method for parsing expressions
    fn expression(&mut self) -> CompileResult<u8> {
        self.term()
    }

    /// Parse a term expression
    /// i.e. parse `x + y` or `x - y`
    fn term(&mut self) -> CompileResult<u8> {
        self.binop(
            Self::factor,
            true,
            vec![
                (TokenKind::Plus,  Instruction::Add),
                (TokenKind::Minus, Instruction::Sub)
            ]
        )
    }

    /// Parse a factor expression
    /// i.e. parse `x * y` or `x / y`
    fn factor(&mut self) -> CompileResult<u8> {
        self.binop(
            Self::unary,
            true,
            vec![
                (TokenKind::Star, Instruction::Mul),
                (TokenKind::Slash, Instruction::Div)
            ]
        )
    }

    /// Parse a unary expression
    /// i.e. parse `!x` or `-x`
    fn unary(&mut self) -> CompileResult<u8> {
        let op_type = self.previous();

        let rhs = self.primitive()?;

        match op_type {
            // Negate number
            Some(TokenKind::Minus) => {
                self.emit_byte(Instruction::Neg, vec![rhs as u8])
            }
            // Invert boolean
            Some(TokenKind::Bang) => {
                self.emit_byte(Instruction::Not, vec![rhs as u8])
            }
            _ => { }
        };
        Ok(rhs)
    }

    /// Compile primitive expressions
    /// i.e. take a value in blush code such as a number or string, and produce Const instructions
    /// according to the TokenKind and slice
    fn primitive(&'_ mut self) -> CompileResult<u8> {
        // Peek the next byte
        let next = self.next();

        // If we reached EOF
        if next.is_none() {
            compile_error!(CompileError::TokenError, "Expected expression, reached EOF")
        }
        // Cannot be None
        let n = next.unwrap();
        use TokenKind::*;
        // Check if the token was a primitive datatype
        let res = match n {
            Number(n) => {
                self.store_const(Value::VNumber(n))
            }
            Bool(b) => {
                self.store_const(Value::VBool(b))
            }
            LeftParen => {
                self.grouping()
            }
            _ => {
                compile_error!(CompileError::TokenError, "Expected expression, got {:?}.", n)
            },
        };
        res
    }

    fn binop(
        &mut self,
        next: fn(&mut Self) -> CompileResult<u8>,
        store: bool,
        expected: Vec<(TokenKind, Instruction)>
    ) -> CompileResult<u8> {
        // Get the left hand side register idx
        let lhs = next(self)? as u8;
        // Check if the next token is any of the expected operators
        if let Some(idx) = self.tag_any(
            expected.iter().map(|i| i.0.clone()).collect()) {
            // Get the right hand side register idx
            let rhs = next(self)? as u8;
            let mut args = vec![lhs, rhs];
            if store {
                // Get the register to store the value in
                args.push(self.use_register()?);
            }
            // Emit the instruction and it's arguments
            self.emit_byte(expected[idx].clone().1, args.clone());
            // Free the registers used for the lhs and rhs for later use
            self.free_register(lhs);
            self.free_register(rhs);

            // Return the register that the value was stored in
            Ok(if store {
                args[2]
            } else {
                0
            })
        } else {
            // The value was not any of the expected operators, so act as a proxy for the higher
            // precedence operation.
            Ok(lhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use crate::{Instruction, Value};
    use super::Compiler;
    use crate::TokenKind;

    mod utils {
        pub(super) fn add_constant(constants: &mut Vec<u8>, instructions: &mut Vec<u8>, value: Value, store: u8) {
            let idx = constants.len();
            let bytes: Vec<u8> = value.into();
            constants.extend(bytes.clone());
            instructions.extend(vec![
                Instruction::Const as u8,
                idx as u8,
                bytes.len() as u8,
                Instruction::Load as u8,
                idx as u8,
                bytes.len() as u8,
                store
            ]);
        }

        pub(super) fn constant_test(value: Value, source: &str) {
            let mut compiler = Compiler {
                lexer: TokenKind::lexer(source),
                ..Default::default()
            };

            compiler.compile().unwrap();
            let mut constants = Vec::new();
            let mut instructions = Vec::new();
            add_constant(&mut constants, &mut instructions, value, 0);
            assert_eq!(compiler.instructions, instructions);
            assert_eq!(compiler.constants, constants);
        }

        pub(super) fn binexp_test(op_c: char, op_i: Instruction) {
            let source: String = format!("8 {} 12;", op_c);
            let mut compiler = Compiler {
                lexer: TokenKind::lexer(source.as_str()),
                ..Default::default()
            };

            compiler.compile().unwrap();
            let mut instructions = vec![];
            let mut constants = vec![];
            add_constant(&mut constants, &mut instructions, Value::VNumber(8.), 0);
            add_constant(&mut constants, &mut instructions, Value::VNumber(12.), 1);
            instructions.append(&mut vec![
                op_i as u8,
                0,
                1,
                2,
            ]);
            assert_eq!(compiler.instructions, instructions);
            assert_eq!(compiler.constants, constants);
        }
    }



    #[test]
    fn constant() {
        utils::constant_test(Value::VNumber(1234.), "1234;");
        utils::constant_test(Value::VNumber(1523.23), "1523.23;");
        utils::constant_test(Value::VBool(false), "false;");
    }

    #[test]
    fn factor() {
        utils::binexp_test('*', Instruction::Mul);
        utils::binexp_test('/', Instruction::Div);
    }

    #[test]
    fn term() {
        utils::binexp_test('+', Instruction::Add);
        utils::binexp_test('-', Instruction::Sub);
    }
}