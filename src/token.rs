//! Token definitions produced by the [`Lexer`](crate::lexer::Lexer).

/// The kind of a lexical token.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum TokenType {
    /// A character the lexer doesn't recognize.
    ILLEGAL,
    /// End of input.
    EOF,
    // Idents and Literals
    IDENT,
    INT,
    // Operators
    ASSIGN,
    PLUS,
    MINUS,
    BANG,
    ASTERISK,
    SLASH,
    LT,
    GT,
    EQ,
    NEQ,
    // Delimiters
    COMMA,
    SEMICOLON,
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    // Keywords
    FUNCTION,
    LET,
    TRUE,
    FALSE,
    IF,
    ELSE,
    RETURN,
}

/// A single token: its kind plus the exact source text it was read from.
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Token {
    pub token: TokenType,
    pub literal: String,
}

impl Token {
    pub(crate) fn new(token: TokenType, literal: String) -> Self {
        Self { token, literal }
    }
}
