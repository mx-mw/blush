use logos::Logos;
#[derive(PartialEq, Debug, Clone, PartialOrd, Logos)]
#[allow(unused)]
pub enum TokenKind {
    // Single-character tokens.
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("/")]
    Slash,
    #[token("*")]
    Star,
    #[token(";")]
    Semicolon,
    #[token("^")]
    Caret,

    // One or two character tokens.
    #[token("!")]
    Bang,
    #[token("!=")]
    BangEqual,
    #[token("=")]
    Equal,
    #[token("==")]
    EqualEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,

    // Literals.
    #[regex("[a-zA-Z_]+[a-zA-Z_0-9]*")]
    Identifier,
    #[regex(r"-?([0-9]+([.][0-9]*)?|[.][0-9]+)", |lex| lex.slice().parse::<f32>().unwrap())]
    Number(f32),

    // Keywords.
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[regex("(true|false)", |lex| lex.slice().parse::<bool>().unwrap())]
    Bool(bool),
    #[token("class")]
    Class,
    #[token("super")]
    Super,
    #[token("fn")]
    Fn,
    #[token("for")]
    For,
    #[token("nil")]
    Nil,
    #[token("return")]
    Return,
    #[token("this")]
    This,
    #[token("let")]
    Let,
    #[token("while")]
    While,
    #[token("\n")]
    Newline,

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"//.**\n", logos::skip)]
    #[regex(r"/\*(.|\n)*\*/\n", logos::skip)]
    Error,
}
