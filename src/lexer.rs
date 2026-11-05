//! Converts Monkey source text into a stream of [`Token`]s.

use crate::token::Token;
use crate::token::TokenType;

/// Byte-oriented lexer. Only ASCII input is supported.
#[derive(Default, Debug)]
pub(crate) struct Lexer<'src> {
    input: &'src str,
    /// Index of `ch` in `input`.
    position: usize,
    /// Index of the next byte to read (always `position + 1`).
    read_position: usize,
    /// The byte currently under examination; `0` means end of input.
    ch: u8,
}

impl<'src> Lexer<'src> {
    /// Creates a lexer positioned on the first byte of `input`.
    pub(crate) fn new(input: &'src str) -> Self {
        let mut lexer = Self {
            input,
            ..Default::default()
        };
        lexer.read_char();
        lexer
    }

    fn is_letter(ch: u8) -> bool {
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase()
    }

    fn is_digit(ch: u8) -> bool {
        ch.is_ascii_digit()
    }

    /// Returns the next token, advancing past it.
    ///
    /// Once the input is exhausted this keeps returning [`TokenType::EOF`].
    pub(crate) fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    Token::new(TokenType::Eq, "==")
                } else {
                    Token::new(TokenType::Assign, "=")
                }
            }
            b',' => Token::new(TokenType::Comma, ","),
            b'{' => Token::new(TokenType::Lbrace, "{"),
            b'}' => Token::new(TokenType::Rbrace, "}"),
            b'[' => Token::new(TokenType::Lbracket, "["),
            b']' => Token::new(TokenType::Rbracket, "]"),
            b'(' => Token::new(TokenType::Lparen, "("),
            b')' => Token::new(TokenType::Rparen, ")"),
            b'+' => Token::new(TokenType::Plus, "+"),
            b'-' => Token::new(TokenType::Minus, "-"),
            b'!' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    Token::new(TokenType::Neq, "!=")
                } else {
                    Token::new(TokenType::Bang, "!")
                }
            }
            b'*' => Token::new(TokenType::Asterisk, "*"),
            b'/' => Token::new(TokenType::Slash, "/"),
            b'<' => Token::new(TokenType::Lt, "<"),
            b'>' => Token::new(TokenType::Gt, ">"),
            b';' => Token::new(TokenType::Semicolon, ";"),
            b':' => Token::new(TokenType::Colon, ":"),
            b'"' => Token::new(TokenType::String, self.read_string()),
            0 => Token::new(TokenType::Eof, ""),
            _ => {
                // Identifiers and numbers consume their own characters, so
                // return early to skip the trailing `read_char` below.
                if Lexer::is_letter(self.ch) {
                    return self.read_identifier();
                }

                if Lexer::is_digit(self.ch) {
                    return self.read_number();
                }

                Token::new(
                    TokenType::Illegal,
                    &self.input[self.position..self.read_position],
                )
            }
        };

        self.read_char();

        token
    }

    /// Returns the next byte without consuming it, or `0` at end of input.
    fn peek_char(&self) -> u8 {
        if self.read_position >= self.input.len() {
            0
        } else {
            self.input.as_bytes()[self.read_position]
        }
    }

    /// Advances one byte, setting `ch` to `0` once past the end of input.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input.as_bytes()[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    /// Reads a run of letters, returning a keyword token if it matches one
    /// and an [`TokenType::IDENT`] otherwise.
    fn read_identifier(&mut self) -> Token {
        let position = self.position;

        while Lexer::is_letter(self.ch) {
            self.read_char();
        }

        let literal = &self.input[position..self.position];

        match literal {
            "let" => Token::new(TokenType::Let, literal),
            "fn" => Token::new(TokenType::Function, literal),
            "true" => Token::new(TokenType::True, literal),
            "false" => Token::new(TokenType::False, literal),
            "if" => Token::new(TokenType::If, literal),
            "else" => Token::new(TokenType::Else, literal),
            "return" => Token::new(TokenType::Return, literal),
            _ => Token::new(TokenType::Ident, literal),
        }
    }

    /// Reads a run of decimal digits as an [`TokenType::INT`].
    fn read_number(&mut self) -> Token {
        let position = self.position;

        while Lexer::is_digit(self.ch) {
            self.read_char();
        }

        let literal = &self.input[position..self.position];

        Token::new(TokenType::Int, literal)
    }

    /// Reads the run of characters unitl the closing quote is found.
    fn read_string(&mut self) -> String {
        self.read_char();
        let position = self.position;

        while self.ch != b'"' && self.ch != 0 {
            self.read_char();
        }

        self.input[position..self.position].to_string()
    }

    fn skip_whitespace(&mut self) {
        while self.ch == b' ' || self.ch == b'\t' || self.ch == b'\n' || self.ch == b'\r' {
            self.read_char();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = r#"let five = 5;
            let ten = 10;

            let add = fn(x, y) {
                x + y;
            };

            let result = add(five, ten);
            
            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }
            

            10 != 9;
            10 == 10;
            "foobar"
            "foo bar"
            [1, 2];
            {"foo": "bar"}
            "#;

        let tests = [
            Token::new(TokenType::Let, "let"),
            Token::new(TokenType::Ident, "five"),
            Token::new(TokenType::Assign, "="),
            Token::new(TokenType::Int, "5"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Let, "let"),
            Token::new(TokenType::Ident, "ten"),
            Token::new(TokenType::Assign, "="),
            Token::new(TokenType::Int, "10"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Let, "let"),
            Token::new(TokenType::Ident, "add"),
            Token::new(TokenType::Assign, "="),
            Token::new(TokenType::Function, "fn"),
            Token::new(TokenType::Lparen, "("),
            Token::new(TokenType::Ident, "x"),
            Token::new(TokenType::Comma, ","),
            Token::new(TokenType::Ident, "y"),
            Token::new(TokenType::Rparen, ")"),
            Token::new(TokenType::Lbrace, "{"),
            Token::new(TokenType::Ident, "x"),
            Token::new(TokenType::Plus, "+"),
            Token::new(TokenType::Ident, "y"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Rbrace, "}"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Let, "let"),
            Token::new(TokenType::Ident, "result"),
            Token::new(TokenType::Assign, "="),
            Token::new(TokenType::Ident, "add"),
            Token::new(TokenType::Lparen, "("),
            Token::new(TokenType::Ident, "five"),
            Token::new(TokenType::Comma, ","),
            Token::new(TokenType::Ident, "ten"),
            Token::new(TokenType::Rparen, ")"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Bang, "!"),
            Token::new(TokenType::Minus, "-"),
            Token::new(TokenType::Slash, "/"),
            Token::new(TokenType::Asterisk, "*"),
            Token::new(TokenType::Int, "5"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Int, "5"),
            Token::new(TokenType::Lt, "<"),
            Token::new(TokenType::Int, "10"),
            Token::new(TokenType::Gt, ">"),
            Token::new(TokenType::Int, "5"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::If, "if"),
            Token::new(TokenType::Lparen, "("),
            Token::new(TokenType::Int, "5"),
            Token::new(TokenType::Lt, "<"),
            Token::new(TokenType::Int, "10"),
            Token::new(TokenType::Rparen, ")"),
            Token::new(TokenType::Lbrace, "{"),
            Token::new(TokenType::Return, "return"),
            Token::new(TokenType::True, "true"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Rbrace, "}"),
            Token::new(TokenType::Else, "else"),
            Token::new(TokenType::Lbrace, "{"),
            Token::new(TokenType::Return, "return"),
            Token::new(TokenType::False, "false"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Rbrace, "}"),
            Token::new(TokenType::Int, "10"),
            Token::new(TokenType::Neq, "!="),
            Token::new(TokenType::Int, "9"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Int, "10"),
            Token::new(TokenType::Eq, "=="),
            Token::new(TokenType::Int, "10"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::String, "foobar"),
            Token::new(TokenType::String, "foo bar"),
            Token::new(TokenType::Lbracket, "["),
            Token::new(TokenType::Int, "1"),
            Token::new(TokenType::Comma, ","),
            Token::new(TokenType::Int, "2"),
            Token::new(TokenType::Rbracket, "]"),
            Token::new(TokenType::Semicolon, ";"),
            Token::new(TokenType::Lbrace, "{"),
            Token::new(TokenType::String, "foo"),
            Token::new(TokenType::Colon, ":"),
            Token::new(TokenType::String, "bar"),
            Token::new(TokenType::Rbrace, "}"),
            Token::new(TokenType::Eof, ""),
        ];

        let mut lexer = Lexer::new(input);

        for t in tests.iter() {
            let token = lexer.next_token();

            assert_eq!(*t, token);
        }
    }
}
