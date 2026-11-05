//! Token definitions produced by the [`Lexer`](crate::lexer::Lexer).

use std::fmt::Display;

/// The kind of a lexical token.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) enum TokenType {
    /// A character the lexer doesn't recognize.
    Illegal,
    /// End of input.
    Eof,
    // Idents and Literals
    Ident,
    Int,
    String,
    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Eq,
    Neq,
    // Delimiters
    Comma,
    Semicolon,
    Colon,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Lbracket,
    Rbracket,
    // Keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

/// A single token: its kind plus the exact source text it was read from.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct Token {
    pub token: TokenType,
    pub literal: String,
}

impl Token {
    pub(crate) fn new(token: TokenType, literal: impl Into<String>) -> Self {
        Self {
            token,
            literal: literal.into(),
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Assign => write!(f, "="),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Bang => write!(f, "!"),
            TokenType::Asterisk => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Lt => write!(f, "<"),
            TokenType::Gt => write!(f, ">"),
            TokenType::Eq => write!(f, "=="),
            TokenType::Neq => write!(f, "!="),
            TokenType::Comma => write!(f, ","),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Lparen => write!(f, "("),
            TokenType::Rparen => write!(f, ")"),
            TokenType::Lbrace => write!(f, "{{"),
            TokenType::Rbrace => write!(f, "}}"),
            TokenType::Lbracket => write!(f, "["),
            TokenType::Rbracket => write!(f, "]"),
            TokenType::Function => write!(f, "fn"),
            token => write!(f, "{token:?}"),
        }
    }
}
