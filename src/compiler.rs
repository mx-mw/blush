/*
	# Compiler
	The compiler consists of many functions for parsing grammar and emmitting bytecode.
	Each parser function returns a u8 referencing the register that the result of the expression was stored in. This
	allows the higher precedence or enclosing expressions to easily reference the value for later use.
 */

use crate::{Instruction, TokenKind, Value, Bag, ZippedBag, runtime::{CompilerScope, Local}, error::{bag::BagError, compiler::*}};
use logos::{Lexer, Logos};

#[derive(Clone)]
pub struct Compiler<'src> {
    pub lexer: Lexer<'src, TokenKind>,
	pub baggage: Vec<ZippedBag>,
	pub current_bag: Bag,
    pub registers: Vec<u8>,
    pub previous: Option<TokenKind>,
    pub previous_slice: String,
    pub current: Option<TokenKind>,
    pub scope: CompilerScope,
}

impl Default for Compiler<'_> {
    fn default() -> Self {
        Self {
            lexer: TokenKind::lexer(""),
			baggage: vec![],
			current_bag: Bag::new(),
            registers: (0..16).collect(),
            previous: None,
            current: None,
            previous_slice: "".into(),
            scope: CompilerScope::default(),
        }
    }
}

impl<'s> Compiler<'s> {
	pub fn new(source: &'s str) -> Self {
		Self {
			lexer: TokenKind::lexer(source),
			..Default::default()
		}
	}

    /// Take the array of tokens and generate bytecode
    pub fn compile(&mut self) -> CompilerResult {
        while self.peek() != None {
            self.declaration()?;
        }
        self.consume(None, "Expected end of expression")?;
		self.new_bag();
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

    /// Consume a token and expect to equal `kind`
    /// If it did not match, throw an error with `why` as the message
    pub(crate) fn consume(
        &mut self,
        kind: Option<TokenKind>,
        why: &'static str,
    ) -> CompilerResult {
        match self.next() {
            // Match kind
            k if k == kind => Ok(()),
            // Give a slightly more verbose error showing the token it saw
            Some(k) => Err(CompilerError::TokenError(
				TokenError::ExpectedToken { reason: why, expected: kind, recieved: Some(k) }
			)),
            // None was not expected
            None => Err(CompilerError::TokenError(
				TokenError::ExpectedToken { reason: why, expected: kind, recieved: None }
			)),
        }
    }

    /// Get the next token without advancing
    pub(crate) fn peek(&self) -> Option<TokenKind> {
        self.lexer.clone().peekable().peek().cloned()
    }

    /// Get the next available register to store a value in
    pub(crate) fn use_register(&mut self) -> CompilerResult<u8> {
        if self.registers.is_empty() {
            Err(CompilerError::RegisterError(RegisterError::NoEmptyRegisters))
        } else {
            Ok(self.registers.remove(0))
        }
    }

    /// Free a register
    pub(crate) fn free_register(&mut self, register: u8) {
        self.registers.push(register)
    }

	pub fn new_bag(&mut self) {
		let zipped = self.current_bag.clone().zip_up();
		self.baggage.push(zipped);
		self.current_bag = Bag::new();
	}

    /// Emit an [Instruction] and it's arguments
    /// Converts an [Instruction] to a u8, and pushes it along with it's arguments onto the end of the
    /// instructions vector
    pub(crate) fn emit_byte(&mut self, instruction: Instruction, arguments: Vec<u8>) -> CompilerResult<()> {
        match self.current_bag.emit_byte(instruction, &arguments) {
			Ok(()) => Ok(()),
			Err(_e) if matches!(BagError::Full, _e)  => {
				
				self.new_bag();
				self.emit_byte(instruction, arguments)?;
				Ok(())
			}
			Err(e) => {
				Err(CompilerError::ExternalError("BagError".into(), e.to_string()))
			}
		}
    }

    /// Store a constant value and append the appropriate bytes to the bytecode
    /// Specifically, encode the value as bytes and append those to the constants vector, then emit
    /// a [Instruction::Const] and the starting index of the vector
    pub(crate) fn emit_const(&mut self, value: Value) -> CompilerResult<u8> {
		let store = self.use_register()?;
        match self.current_bag.emit_const(&value, store) {
			Ok(()) => {},
			Err(e) => {
				match e {
					BagError::Full(_) => {
						self.new_bag();
						self.free_register(store);
						self.emit_const(value)?;
					}
					_ => {
						Err(CompilerError::ExternalError("BagError".into(), e.to_string()))?;
					}
				}
				
			}
		}

		Ok(store)
    }

    /// Check if the next token is expected
    pub(crate) fn tag(&mut self, expected: Option<TokenKind>) -> bool {
        if self.peek() == expected {
            self.next();
            true
        } else {
            false
        }
    }

    /// Wrapper around `tag` for multiple values of `expected`
    pub(crate) fn tag_any(&mut self, expected: Vec<TokenKind>) -> Option<usize> {
        for (idx, token) in expected.iter().enumerate() {
            if self.tag(Some(token.clone())) {
                return Some(idx);
            }
        }
        None
    }

    pub(crate) fn declaration(&mut self) -> CompilerResult {
        if self.tag(Some(TokenKind::Let)) {
            self.let_declaration()
        } else {
            self.statement()?;
            Ok(())
        }
    }

    pub(crate) fn let_declaration(&mut self) -> CompilerResult {
        let global = self.parse_variable("Expected variable name after 'let'.")?;

        self.consume(Some(TokenKind::Equal), "Variables must be initialized.")?;
        let v = self.expression()?;

        self.consume(
            Some(TokenKind::Semicolon),
            "Expected ';' after variable declaration",
        )?;
        self.define_variable(global, v)?;
        Ok(())
    }

    pub(crate) fn statement(&mut self) -> CompilerResult<u8> {
        if self.tag(Some(TokenKind::LeftBrace)) {
            self.begin_scope();
            let v = self.block()?;
            self.end_scope();
            Ok(v)
        } else {
            self.expression_stmt()
        }
    }

    pub(crate) fn expression_stmt(&mut self) -> CompilerResult<u8> {
        let res = self.expression()?;
        self.consume(
            Some(TokenKind::Semicolon),
            "Expected ';' at end of expression",
        )?;
        Ok(res)
    }

    /// Parse expressions and generate bytecode
    /// Root method for parsing expressions
    pub(crate) fn expression(&mut self) -> CompilerResult<u8> {
        self.equality()
    }

    /// Parse an equality assertion expression.
    /// i.e. parse `x == y` or `x != y`
    pub(crate) fn equality(&mut self) -> CompilerResult<u8> {
        self.binop(
            Self::comparison,
            false,
            vec![
                (TokenKind::EqualEqual, Instruction::Eq, false),
                (TokenKind::BangEqual, Instruction::Ne, false),
            ],
        )
    }

    /// Parse a comparison expression.
    /// i.e. parse `x < y`, `x > y`, `x <= y` or `x >= y`
    pub(crate) fn comparison(&mut self) -> CompilerResult<u8> {
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
    pub(crate) fn term(&mut self) -> CompilerResult<u8> {
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
    pub(crate) fn factor(&mut self) -> CompilerResult<u8> {
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
    pub(crate) fn unary(&mut self) -> CompilerResult<u8> {
        let unary_ops = vec![
            (TokenKind::Minus, Instruction::Neg),
            (TokenKind::Bang, Instruction::Not),
        ];
        Ok(
            if let Some(idx) = self.tag_any(unary_ops.iter().map(|i| i.0.clone()).collect()) {
                let rhs = self.primitive()?;
				let store = self.use_register()?;
                self.emit_byte(unary_ops[idx].1, vec![rhs, store])?;
                self.free_register(rhs);
                store
            } else {
                self.primitive()?
            },
        )
    }

    /// Parse a grouping (stuff in parentheses) expression
    pub(crate) fn grouping(&mut self) -> CompilerResult<u8> {
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
    pub(crate) fn primitive(&'_ mut self) -> CompilerResult<u8> {
        // Peek the next byte
        let next = self.next();

        // If we reached EOF
        if next.is_none() {
            Err(CompilerError::TokenError(TokenError::EarlyEof))?
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
            _ => Err(CompilerError::TokenError(TokenError::EarlyEof))

        };
        res
    }

    pub(crate) fn load_variable(&mut self) -> CompilerResult<u8> {
        let idx = self.ident_const()?;
        if self.tag(Some(TokenKind::Equal)) {
            let value = self.expression()?;
            self.emit_byte(Instruction::Set, vec![idx, value])?;
        }
        let store = self.use_register()?;
        self.emit_byte(Instruction::Read, vec![idx, store])?;
        Ok(store)
    }

    pub(crate) fn block(&mut self) -> CompilerResult<u8> {
		self.scope.depth+=1;
        while self.peek() != Some(TokenKind::RightBrace) && !self.tag(None) {
            self.declaration()?;
        }
        self.consume(Some(TokenKind::RightBrace), "Expect '}' after block.")?;
        Ok(0) // TODO(mx-mw) implement returning values
    }

    pub(crate) fn begin_scope(&mut self) {
        self.scope.depth += 1;
    }

    pub(crate) fn end_scope(&mut self) {
        self.scope.depth -= 1;
    }

    /// Parse a variable and produce a global index
    pub(crate) fn parse_variable(&mut self, why: &'static str) -> CompilerResult<u8> {
        self.consume(Some(TokenKind::Identifier), why)?;

        self.declare_variable();

        self.ident_const()
    }

    pub(crate) fn declare_variable(&mut self) {
        self.scope.num_vars += 1;
        self.scope.vars.push(Local {
            name: self.lexer.slice().to_string(),
            depth: self.scope.depth,
        });
    }

    pub(crate) fn ident_const(&mut self) -> CompilerResult<u8> {
        self.emit_const(Value::VString(self.lexer.slice().to_string()))
    }

    pub(crate) fn define_variable(&mut self, ident_idx: u8, value_idx: u8) -> CompilerResult {
        self.emit_byte(Instruction::Let, vec![ident_idx, value_idx])?;
        Ok(())
    }

    /// Parse a binary expression.
    /// i.e. parse any expression which consists of a prefix, infix and suffix.
    pub(crate) fn binop(
        &mut self,
        next: fn(&mut Self) -> CompilerResult<u8>,
        store: bool,
        expected: Vec<(TokenKind, Instruction, bool)>,
    ) -> CompilerResult<u8> {
        // Get the left hand side register idx
        let lhs = next(self)? as u8;
        // Check if the next token is any of the expected operators
        if let Some(idx) = self.tag_any(expected.iter().map(|i| i.0.clone()).collect()) {
            // Get the right hand side register idx
            let rhs = next(self)? as u8;
            let mut args = if expected[idx].2 {
                vec![rhs, lhs]
            } else {
                vec![lhs, rhs]
            };
            if store {
                // Get the register to store the value in
                args.push(self.use_register()?);
            }
            // Emit the instruction and it's arguments
            self.emit_byte(expected[idx].clone().1, args.clone())?;
            // Free the registers used for the lhs and rhs for later use
            self.free_register(lhs);
            self.free_register(rhs);

            // Return the register that the value was stored in
            Ok(if store { args[2] } else { 0 })
        } else {
            // The value was not any of the expected operators, so hand on to the higher
            // precedence operation.
            Ok(lhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Compiler, Instruction, TokenKind, Value, Bag};
    use logos::Logos;

    pub mod utils {
        use crate::{Compiler, Instruction, TokenKind, Value, Bag};
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

        /// Test a constant value
        pub(super) fn constant_test(value: Value, source: &str) {
            let compiler = compiler(source);

            let mut bag = Bag::new();
            // Add the expected value onto the arrays
            assert!(bag.emit_const(&value, 0).is_ok());

			
            // Assert that the correct constants and instructions were emitted
            assert_eq!(compiler.baggage, vec![bag.zip_up()]);
        }

        /// Test a binary expression
        pub(super) fn binexp_test(op_c: &'static str, op_i: Instruction, rev: bool, store: bool) {
            let source: String = format!("8 {} 12;", op_c);
            let compiler = compiler(source.as_str());

            let mut bag = Bag::new();
            // Add the default testing values as constants to the arrays
			assert!(bag.emit_const(&Value::VNumber(8.), 0).is_ok());
			assert!(bag.emit_const(&Value::VNumber(12.), 1).is_ok());
            if !rev {
				bag.emit_byte(op_i, &vec![0, 1]).unwrap()
            } else {
                bag.emit_byte(op_i, &vec![1, 0]).unwrap()
            }

            if store {
                bag.emit_byte(Instruction::Sub /* 2 */, &vec![]).unwrap()
            }

            // Assert that the correct instructions and constants were stored
			assert_eq!(compiler.baggage, vec![bag.zip_up()])
        }
    }

    use utils::compiler;

    #[test]
    fn consume() {
        let mut compiler = Compiler::default();
        compiler.lexer = TokenKind::lexer(";;;;;;");
        assert!(compiler.consume(Some(TokenKind::Semicolon), "").is_ok());
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

        let mut bag = Bag::new();
		assert!(bag.emit_const(&Value::VBool(false), 0).is_ok());
		assert!(bag.emit_byte(Instruction::Not, &vec![0, 1]).is_ok());
        assert_eq!(compiler.baggage, vec![bag.zip_up()]);
    }

    #[test]
    fn let_declaration() {
        let compiler = compiler("let asdf = true;");
        let mut bag = Bag::new();
		assert!(bag.emit_const(&Value::VString("asdf".into()), 0).is_ok());
		assert!(bag.emit_const(&Value::VBool(true), 1).is_ok());

        let scope = super::CompilerScope {
            vars: vec![super::Local {
                name: "asdf".to_string(),
                depth: 0,
            }],
            num_vars: 1,
            depth: 0,
        };

		assert!(bag.emit_byte(Instruction::Let, &vec![0, 1]).is_ok());
		assert_eq!(compiler.baggage, vec![bag.zip_up()]);
        assert_eq!(compiler.scope, scope);
    }
}
