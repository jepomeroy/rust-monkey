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
        return b'a' <= ch && ch <= b'z' || b'A' <= ch && ch <= b'Z';
    }

    fn is_digit(ch: u8) -> bool {
        return b'0' <= ch && ch <= b'9';
    }

    /// Returns the next token, advancing past it.
    ///
    /// Once the input is exhausted this keeps returning [`TokenType::EOF`].
    pub(crate) fn next_token(&mut self) -> Token<'src> {
        self.skip_whitespace();

        let token = match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    Token::new(TokenType::EQ, "==")
                } else {
                    Token::new(TokenType::ASSIGN, "=")
                }
            }
            b',' => Token::new(TokenType::COMMA, ","),
            b'{' => Token::new(TokenType::LBRACE, "{"),
            b'}' => Token::new(TokenType::RBRACE, "}"),
            b'(' => Token::new(TokenType::LPAREN, "("),
            b')' => Token::new(TokenType::RPAREN, ")"),
            b'+' => Token::new(TokenType::PLUS, "+"),
            b'-' => Token::new(TokenType::MINUS, "-"),
            b'!' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    Token::new(TokenType::NEQ, "!=")
                } else {
                    Token::new(TokenType::BANG, "!")
                }
            }
            b'*' => Token::new(TokenType::ASTERISK, "*"),
            b'/' => Token::new(TokenType::SLASH, "/"),
            b'<' => Token::new(TokenType::LT, "<"),
            b'>' => Token::new(TokenType::GT, ">"),
            b';' => Token::new(TokenType::SEMICOLON, ";"),
            0 => Token::new(TokenType::EOF, ""),
            _ => {
                // Identifiers and numbers consume their own characters, so
                // return early to skip the trailing `read_char` below.
                if Lexer::is_letter(self.ch) {
                    return self.read_identifier();
                }

                if Lexer::is_digit(self.ch) {
                    return self.read_number();
                }

                Token::new(TokenType::ILLEGAL, &self.input[self.position..self.read_position])
            }
        };

        self.read_char();

        token
    }

    /// Returns the next byte without consuming it, or `0` at end of input.
    fn peek_char(&self) -> u8 {
        if self.read_position >= self.input.len() {
            return 0;
        } else {
            return self.input.as_bytes()[self.read_position];
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
    fn read_identifier(&mut self) -> Token<'src> {
        let position = self.position;

        while Lexer::is_letter(self.ch) {
            self.read_char();
        }

        let literal = &self.input[position..self.position];

        match literal {
            "let" => Token::new(TokenType::LET, literal),
            "fn" => Token::new(TokenType::FUNCTION, literal),
            "true" => Token::new(TokenType::TRUE, literal),
            "false" => Token::new(TokenType::FALSE, literal),
            "if" => Token::new(TokenType::IF, literal),
            "else" => Token::new(TokenType::ELSE, literal),
            "return" => Token::new(TokenType::RETURN, literal),
            _ => Token::new(TokenType::IDENT, literal),
        }
    }

    /// Reads a run of decimal digits as an [`TokenType::INT`].
    fn read_number(&mut self) -> Token<'src> {
        let position = self.position;

        while Lexer::is_digit(self.ch) {
            self.read_char();
        }

        let literal = &self.input[position..self.position];

        Token::new(TokenType::INT, literal)
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
        let input = r"let five = 5;
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
            ";

        let tests = [
            Token::new(TokenType::LET, "let"),
            Token::new(TokenType::IDENT, "five"),
            Token::new(TokenType::ASSIGN, "="),
            Token::new(TokenType::INT, "5"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::LET, "let"),
            Token::new(TokenType::IDENT, "ten"),
            Token::new(TokenType::ASSIGN, "="),
            Token::new(TokenType::INT, "10"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::LET, "let"),
            Token::new(TokenType::IDENT, "add"),
            Token::new(TokenType::ASSIGN, "="),
            Token::new(TokenType::FUNCTION, "fn"),
            Token::new(TokenType::LPAREN, "("),
            Token::new(TokenType::IDENT, "x"),
            Token::new(TokenType::COMMA, ","),
            Token::new(TokenType::IDENT, "y"),
            Token::new(TokenType::RPAREN, ")"),
            Token::new(TokenType::LBRACE, "{"),
            Token::new(TokenType::IDENT, "x"),
            Token::new(TokenType::PLUS, "+"),
            Token::new(TokenType::IDENT, "y"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::RBRACE, "}"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::LET, "let"),
            Token::new(TokenType::IDENT, "result"),
            Token::new(TokenType::ASSIGN, "="),
            Token::new(TokenType::IDENT, "add"),
            Token::new(TokenType::LPAREN, "("),
            Token::new(TokenType::IDENT, "five"),
            Token::new(TokenType::COMMA, ","),
            Token::new(TokenType::IDENT, "ten"),
            Token::new(TokenType::RPAREN, ")"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::BANG, "!"),
            Token::new(TokenType::MINUS, "-"),
            Token::new(TokenType::SLASH, "/"),
            Token::new(TokenType::ASTERISK, "*"),
            Token::new(TokenType::INT, "5"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::INT, "5"),
            Token::new(TokenType::LT, "<"),
            Token::new(TokenType::INT, "10"),
            Token::new(TokenType::GT, ">"),
            Token::new(TokenType::INT, "5"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::IF, "if"),
            Token::new(TokenType::LPAREN, "("),
            Token::new(TokenType::INT, "5"),
            Token::new(TokenType::LT, "<"),
            Token::new(TokenType::INT, "10"),
            Token::new(TokenType::RPAREN, ")"),
            Token::new(TokenType::LBRACE, "{"),
            Token::new(TokenType::RETURN, "return"),
            Token::new(TokenType::TRUE, "true"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::RBRACE, "}"),
            Token::new(TokenType::ELSE, "else"),
            Token::new(TokenType::LBRACE, "{"),
            Token::new(TokenType::RETURN, "return"),
            Token::new(TokenType::FALSE, "false"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::RBRACE, "}"),
            Token::new(TokenType::INT, "10"),
            Token::new(TokenType::NEQ, "!="),
            Token::new(TokenType::INT, "9"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::INT, "10"),
            Token::new(TokenType::EQ, "=="),
            Token::new(TokenType::INT, "10"),
            Token::new(TokenType::SEMICOLON, ";"),
            Token::new(TokenType::EOF, ""),
        ];

        let mut lexer = Lexer::new(input);

        for t in tests.iter() {
            let token = lexer.next_token();

            assert_eq!(*t, token);
        }
    }
}
