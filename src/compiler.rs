use crate::{instruction::Instruction, scanner::TokenKind, value::Value};
use logos::{Lexer, Logos};
pub type CompileResult<T> = Result<T, (CompileError, String)>;

#[derive(Debug, PartialEq)]
pub enum CompileError {
    TokenError,    // A token was not in the correct position
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

pub struct Local {
    name: String,
    depth: u8,
}

pub struct Scope {
    locals: Vec<Local>,
    num_locals: u8,
    depth: u8,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            locals: Vec::new(),
            num_locals: 0,
            depth: 0,
        }
    }
}

#[derive(Clone)]
pub struct Compiler<'src> {
    pub lexer: Lexer<'src, TokenKind>,
    pub instructions: Vec<u8>,
    pub constants: Vec<u8>,
    // TODO(mx-mw) investigate using a linked list to make this more efficient
    pub registers: Vec<u8>,
    pub previous: Option<TokenKind>,
    pub previous_slice: String,
    pub current: Option<TokenKind>,
    pub scope: Scope,
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
            previous_slice: "".into(),
            scope: Scope::default()
        }
    }
}

impl<'s> Compiler<'s> {
    /// Take the array of tokens and generate bytecode
    pub fn compile(&mut self) -> CompileResult<()> {
        while self.peek() != None{
            self.declaration()?;
        }
        self.consume(None, "Expected end of expression")?;
        Ok(())
    }

    /// Wrapper around [Logos::Lexer::next]
    /// Consume and return the next token if it exists
    fn next(&mut self) -> Option<TokenKind> {
        self.previous = self.current.clone();
        self.previous_slice = self.lexer.slice().to_string();
        self.current = self.lexer.next();

        self.current.clone()
    }

    fn previous(&self) -> Option<TokenKind> {
        self.previous.clone()
    }

    /// Consume a token and expect to equal `kind`
    /// If it did not match, throw an error with `why` as the message
    pub(crate) fn consume(
        &mut self,
        kind: Option<TokenKind>,
        why: &'static str,
    ) -> CompileResult<()> {
        match self.next() {
            // Match kind
            k if k == kind => Ok(()),
            // Give a slightly more verbose error showing the token it saw
            Some(k) => compile_error!(
                CompileError::TokenError,
                "{} (Expected {:?}; got {:?})",
                why,
                kind,
                k
            ),
            // None was not expected
            None => compile_error!(CompileError::TokenError, "{} (Expected {:?})", why, kind),
        }
    }

    /// Get the next token without advancing
    pub(crate) fn peek(&self) -> Option<TokenKind> {
        self.lexer.clone().peekable().peek().cloned()
    }

    /// Get the next available register to store a value in
    pub(crate) fn use_register(&mut self) -> CompileResult<u8> {
        if self.registers.is_empty() {
            compile_error!(CompileError::RegisterError, "No empty registers")
        } else {
            Ok(self.registers.remove(0))
        }
    }

    /// Free a register
    pub(crate) fn free_register(&mut self, register: u8) {
        self.registers.push(register)
    }

    /// Emit an [Instruction] and it's arguments
    /// Converts an [Instruction] to a u8, and pushes it along with it's arguments onto the end of the
    /// instructions vector
    pub(crate) fn emit_byte(&mut self, instruction: Instruction, arguments: Vec<u8>) {
        // Push the instruction as a byte onto the vec
        self.instructions.push(instruction as u8);
        // Extend the `instructions` vec with the `arguments` vec
        self.instructions.extend(arguments)
    }

    /// Store a constant value and append the appropriate bytes to the bytecode
    /// Specifically, encode the value as bytes and append those to the constants vector, then emit
    /// a [Instruction::Const] and the starting index of the vector
    pub(crate) fn emit_const(&mut self, value: Value) -> CompileResult<u8> {
        // Get the index of the first byte of the value
        let idx = self.constants.len();
        // Convert the value to bytes
        let value: Vec<u8> = value.into();
        // Emit the byte with the starting index and the length of the value as the arguments
        let store = self.use_register()?;
        self.emit_byte(Instruction::Const, vec![idx as u8, value.len() as u8, store]);
        // Append the byteified value onto the `constants` vec
        self.constants.extend(value.clone());
        Ok(store)
    }

    /// Check if the next token is expected
    pub(crate) fn tag(&mut self, expected: Option<&TokenKind>) -> bool {
        if self.peek() == expected.clone() {
            self.next();
            true
        } else {
            false
        }
    }

    /// Wrapper around `tag` for multiple values of `expected`
    pub(crate) fn tag_any(&mut self, expected: Vec<TokenKind>) -> Option<usize> {
        for (idx, token) in expected.iter().enumerate() {
            if self.tag(Some(token)) {
                return Some(idx);
            }
        }
        None
    }

    pub(crate) fn declaration(&mut self) -> CompileResult<()> {
        if self.tag(Some(&TokenKind::Let)) {
            self.let_declaration()
        } else {
            self.statement()?;
            Ok(())
        }
    }

    pub(crate) fn let_declaration(&mut self) -> CompileResult<()> {
        let global = self.parse_variable("Expected variable name after 'let'.")?;

        self.consume(
            Some(TokenKind::Equal),
            "Variables must be initialized."
        )?;
        let v = self.expression()?;
        self.consume(
            Some(TokenKind::Semicolon),
            "Expected ';' after variable declaration"
        )?;
        self.define_variable(global, v);
        Ok(())
    }

    pub(crate) fn statement(&mut self) -> CompileResult<u8> {
        if self.tag(Some(&TokenKind::LeftBrace)) {
            self.begin_scope();
            let v = self.block()?;
            self.end_scope();
            Ok(v)
        } else {
            self.expression_stmt()
        }
    }

    pub(crate) fn expression_stmt(&mut self) -> CompileResult<u8> {
        let res = self.expression()?;
        self.consume(
            Some(TokenKind::Semicolon),
            "Expected ';' at end of expression",
        )?;
        Ok(res)
    }

    /// Parse expressions and generate bytecode
    /// Root method for parsing expressions
    pub(crate) fn expression(&mut self) -> CompileResult<u8> {
        self.equality()
    }

    /// Parse an equality assertion expression.
    /// i.e. parse `x == y` or `x != y`
    pub(crate) fn equality(&mut self) -> CompileResult<u8> {
        self.binop(
            Self::comparison,
            false,
            vec![
                (TokenKind::EqualEqual, Instruction::Eq, false),
                (TokenKind::BangEqual, Instruction::Ne, false),
            ]
        )
    }

    /// Parse a comparison expression.
    /// i.e. parse `x < y`, `x > y`, `x <= y` or `x >= y`
    pub(crate) fn comparison(&mut self) -> CompileResult<u8> {
        self.binop(
            Self::term,
            false,
            vec![
                (TokenKind::Less, Instruction::Lt, false),
                (TokenKind::Greater, Instruction::Lt, true),
                (TokenKind::LessEqual, Instruction::Le, false),
                (TokenKind::GreaterEqual, Instruction::Le, true),
            ],
        )
    }

    /// Parse a term expression
    /// i.e. parse `x + y` or `x - y`
    pub(crate) fn term(&mut self) -> CompileResult<u8> {
        self.binop(
            Self::factor,
            true,
            vec![
                (TokenKind::Plus, Instruction::Add, false),
                (TokenKind::Minus, Instruction::Sub, false),
            ],
        )
    }

    /// Parse a factor expression
    /// i.e. parse `x * y` or `x / y`
    pub(crate) fn factor(&mut self) -> CompileResult<u8> {
        self.binop(
            Self::unary,
            true,
            vec![
                (TokenKind::Star, Instruction::Mul, false),
                (TokenKind::Slash, Instruction::Div, false),
            ],
        )
    }

    /// Parse a unary expression
    /// i.e. parse `!x` or `-x`
    pub(crate) fn unary(&mut self) -> CompileResult<u8> {
        let unary_ops = vec![
            (TokenKind::Minus, Instruction::Neg),
            (TokenKind::Bang, Instruction::Not),
        ];
        Ok(
            if let Some(idx) = self.tag_any(unary_ops.iter().map(|i| i.0.clone()).collect()) {
                let rhs = self.primitive()?;
                let store = self.use_register()?;
                self.emit_byte(unary_ops[idx].1, vec![rhs, store]);
                self.free_register(rhs);
                store
            } else {
                self.primitive()?
            },
        )
    }

    /// Parse a grouping (stuff in parentheses) expression
    pub(crate) fn grouping(&mut self) -> CompileResult<u8> {
        let idx = self.expression()?;
        self.consume(
            Some(TokenKind::RightParen),
            "Expected ')' following expression.",
        )?;
        Ok(idx)
    }

    /// Compile primitive expressions
    /// i.e. take a value in blush code such as a number or string, and produce Const instructions
    /// according to the TokenKind and slice
    pub(crate) fn primitive(&'_ mut self) -> CompileResult<u8> {
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
            Number(n) => self.emit_const(Value::VNumber(n)),
            Bool(b) => self.emit_const(Value::VBool(b)),
            Identifier => self.load_variable(),
            LeftParen => self.grouping(),
            _ => {
                compile_error!(
                    CompileError::TokenError,
                    "Expected expression, got {:?}.",
                    n
                )
            }
        };
        res
    }

    pub(crate) fn load_variable(&mut self) -> CompileResult<u8> {
        let idx = self.ident_const()?;
        if self.tag(Some(&TokenKind::Equal)) {
            let value = self.expression()?;
            self.emit_byte(Instruction::Set, vec![idx, value])
        }
        let store = self.use_register()?;
        self.emit_byte(Instruction::Read, vec![idx, store]);
        Ok(store)
    }

    pub(crate) fn block(&mut self) -> CompileResult<u8> {
        while !self.tag(Some(&TokenKind::RightBrace)) && !self.tag(None) {
            self.declaration();
        }
        self.consume(Some(TokenKind::RightBrace), "Expect '}' after block.");
        Ok(0) // TODO(mx-mw) implement returning values
    }

    pub(crate) fn begin_scope(&mut self) {
        self.scope.depth += 1;
    }

    pub(crate) fn end_scope(&mut self) {
        self.scope.depth -= 1;
    }

    /// Parse a variable and produce a global index
    pub(crate) fn parse_variable(&mut self, why: &str) -> CompileResult<u8> {
        self.consume(Some(TokenKind::Identifier), why)?;

        self.declare_variable();

        self.ident_const()
    }

    pub(crate) fn declare_variable(&mut self) {
        self.scope.num_locals += 1;
        self.scope.locals.push(Local {
            name: self.previous_slice.clone(),
            depth: self.scope.depth
        });
    }

    pub(crate) fn ident_const(&mut self) -> CompileResult<u8> {
        self.emit_const(Value::VString(self.lexer.slice().to_string()))
    }

    pub(crate) fn define_variable(&mut self, ident_idx: u8, value_idx: u8) -> CompileResult<()> {
        Ok(self.emit_byte(Instruction::Let, vec![ident_idx, value_idx]))
    }

    /// Parse a binary expression.
    /// i.e. parse any expression which consists of a prefix, infix and suffix.
    pub(crate) fn binop(
        &mut self,
        next: fn(&mut Self) -> CompileResult<u8>,
        store: bool,
        expected: Vec<(TokenKind, Instruction, bool)>,
    ) -> CompileResult<u8> {
        // Get the left hand side register idx
        let lhs = next(self)? as u8;
        // Check if the next token is any of the expected operators
        if let Some(idx) = self.tag_any(expected.iter().map(|i| i.0.clone()).collect()) {
            // Get the right hand side register idx
            let rhs = next(self)? as u8;
            let mut args = if expected[idx].2 {
                vec![lhs, rhs]
            } else {
                vec![rhs, lhs]
            };
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
            Ok(if store { args[2] } else { 0 })
        } else {
            // The value was not any of the expected operators, so act as a proxy for the higher
            // precedence operation.
            Ok(lhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Compiler, Instruction, TokenKind, Value};
    use logos::Logos;

    pub mod utils {
        use crate::{Compiler, Instruction, TokenKind, Value};
        use logos::Logos;

        /// Init a compiler instance
        #[inline]
        pub(super) fn compiler(source: &str) -> Compiler {
            // Create an instance, use default values as they are not necessary for testing (yet)
            let mut compiler = Compiler {
                lexer: TokenKind::lexer(source),
                ..Default::default()
            };

            // TODO(mx-mw) add a parameter to make compilation optional
            compiler.compile().unwrap();
            compiler
        }

        /// Emit the bytecode to add a constant to the constants and instructions vectors
        pub(super) fn add_constant(
            constants: &mut Vec<u8>,
            instructions: &mut Vec<u8>,
            value: Value,
            store: u8,
        ) {
            // Get the index at which the first byte of the value will be stored
            let idx = constants.len();
            // Get the raw bytes of the constant
            let bytes: Vec<u8> = value.into();
            // Push the value bytes onto the constants array
            constants.extend(bytes.clone());
            // Push the appropriate instructions to store a constant and load it into a register
            instructions.extend(vec![
                Instruction::Const as u8,
                idx as u8,
                bytes.len() as u8,
                Instruction::Load as u8,
                idx as u8,
                bytes.len() as u8,
                store,
            ]);
        }

        /// Test a constant value
        pub(super) fn constant_test(value: Value, source: &str) {
            let compiler = compiler(source);

            let mut constants = Vec::new();
            let mut instructions = Vec::new();
            // Add the expected value onto the arrays
            add_constant(&mut constants, &mut instructions, value, 0);
            // Assert that the correct instructions were emitted
            assert_eq!(compiler.instructions, instructions);
            // Assert that the correct values were stored
            assert_eq!(compiler.constants, constants);
        }

        /// Test a binary expression
        pub(super) fn binexp_test(op_c: &'static str, op_i: Instruction, rev: bool, store: bool) {
            let source: String = format!("8 {} 12;", op_c);
            let compiler = compiler(source.as_str());

            let mut instructions = vec![];
            let mut constants = vec![];
            // Add the default testing values as constants to the arrays
            add_constant(&mut constants, &mut instructions, Value::VNumber(8.), 0);
            add_constant(&mut constants, &mut instructions, Value::VNumber(12.), 1);
            if rev {
                instructions.append(&mut vec![op_i as u8, 0, 1]);
            } else {
                instructions.append(&mut vec![op_i as u8, 1, 0]);
            }

            if store {
                instructions.push(2)
            }

            // Assert that the correct instructions were emitted
            assert_eq!(compiler.instructions, instructions);
            // Assert that the correct values were stored
            assert_eq!(compiler.constants, constants);
        }
    }

    use utils::compiler;

    #[test]
    fn emit_bytecode() {
        let mut compiler = Compiler::default();
        compiler.emit_byte(Instruction::Add, vec![12, 13, 234]);
        assert_eq!(compiler.instructions, vec![2, 12, 13, 234]);
        compiler.instructions = vec![];
        compiler.emit_byte(Instruction::Mul, vec![34, 2, 12]);
        assert_eq!(compiler.instructions, vec![4, 34, 2, 12]);
        compiler.instructions = vec![];
        compiler.emit_byte(Instruction::Load, vec![5, 2, 32]);
        assert_eq!(compiler.instructions, vec![1, 5, 2, 32]);
        compiler.instructions = vec![];
        compiler.emit_byte(Instruction::Div, vec![1, 2, 3]);
        assert_eq!(compiler.instructions, vec![5, 1, 2, 3]);
        compiler.instructions = vec![];
        compiler.emit_byte(Instruction::Not, vec![5, 6, 4]);
        assert_eq!(compiler.instructions, vec![10, 5, 6, 4]);
        compiler.instructions = vec![];
    }

    #[test]
    fn consume() {
        let mut compiler = Compiler::default();
        compiler.lexer = TokenKind::lexer(";;;;;;");
        assert!(compiler.consume(Some(TokenKind::Semicolon), "").is_ok());
        // TODO(mx-mw) do this but without the console stuff... maybe add a flag for logging
        // assert_eq!(
        //     compiler.consume(Some(TokenKind::Bang), "..."),
        //     Err((
        //         CompileError::TokenError,
        //         "... (Expected Some(Bang); got Semicolon)".to_string()
        //     ))
        // );
    }

    #[test]
    fn comparison() {
        utils::binexp_test("<", Instruction::Lt, false, false);
        utils::binexp_test(">", Instruction::Lt, true, false);
        utils::binexp_test("<=", Instruction::Le, false, false);
        utils::binexp_test(">=", Instruction::Le, true, false);
    }

    #[test]
    fn constant() {
        utils::constant_test(Value::VNumber(1234.), "1234;");
        utils::constant_test(Value::VNumber(1523.23), "1523.23;");
        utils::constant_test(Value::VBool(false), "false;");
    }

    #[test]
    fn factor() {
        utils::binexp_test("*", Instruction::Mul, false, true);
        utils::binexp_test("/", Instruction::Div, false, true);
    }

    #[test]
    fn term() {
        utils::binexp_test("+", Instruction::Add, false, true);
        utils::binexp_test("-", Instruction::Sub, false, true);
    }

    #[test]
    fn unary() {
        // No need to test negative numbers - regex parses negatives as well as positives
        // TODO(mx-mw) add [Instruction::Neg] implementation once variables are implemented
        let compiler = compiler("!false;");

        let mut instructions = Vec::new();
        let mut constants = Vec::new();
        utils::add_constant(&mut constants, &mut instructions, Value::VBool(false), 0);
        instructions.extend(vec![Instruction::Not as u8, 0, 1]);
        assert_eq!(compiler.instructions, instructions);
        assert_eq!(compiler.constants, constants);
    }
}
