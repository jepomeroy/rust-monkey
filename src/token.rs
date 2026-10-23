//! Token definitions produced by the [`Lexer`](crate::lexer::Lexer).

use std::fmt::Display;

/// The kind of a lexical token.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
pub(crate) struct Token<'src> {
    pub token: TokenType,
    pub literal: &'src str,
}

impl<'src> Token<'src> {
    pub(crate) fn new(token: TokenType, literal: &'src str) -> Self {
        Self { token, literal }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::ASSIGN => write!(f, "="),
            TokenType::PLUS => write!(f, "+"),
            TokenType::MINUS => write!(f, "-"),
            TokenType::BANG => write!(f, "!"),
            TokenType::ASTERISK => write!(f, "*"),
            TokenType::SLASH => write!(f, "/"),
            TokenType::LT => write!(f, "<"),
            TokenType::GT => write!(f, ">"),
            TokenType::EQ => write!(f, "=="),
            TokenType::NEQ => write!(f, "!="),
            TokenType::COMMA => write!(f, ","),
            TokenType::SEMICOLON => write!(f, ";"),
            TokenType::LPAREN => write!(f, "("),
            TokenType::RPAREN => write!(f, ")"),
            TokenType::LBRACE => write!(f, "{{"),
            TokenType::RBRACE => write!(f, "}}"),
            TokenType::FUNCTION => write!(f, "fn"),
            token => write!(f, "{token:?}"),
        }
    }
}
