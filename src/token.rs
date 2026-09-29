#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenType {
    ILLEGAL,
    EOF,
    // Idents and Literals
    IDENT,
    INT,
    // Operators
    ASSIGN,
    PLUS,
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
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub token: TokenType,
    pub literal: String,
}

impl Token {
    pub(crate) fn new(token: TokenType, literal: String) -> Self {
        Self { token, literal }
    }
}
