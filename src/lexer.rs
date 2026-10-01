//! Converts Monkey source text into a stream of [`Token`]s.

use crate::token::Token;
use crate::token::TokenType;

/// Byte-oriented lexer. Only ASCII input is supported.
#[derive(Default)]
pub(crate) struct Lexer {
    input: Vec<u8>,
    /// Index of `ch` in `input`.
    position: usize,
    /// Index of the next byte to read (always `position + 1`).
    read_position: usize,
    /// The byte currently under examination; `0` means end of input.
    ch: u8,
}

impl Lexer {
    /// Creates a lexer positioned on the first byte of `input`.
    pub(crate) fn new(input: String) -> Self {
        let mut lexer = Self {
            input: input.as_bytes().to_vec(),
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
    pub(crate) fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    Token::new(TokenType::EQ, "==".to_string())
                } else {
                    Token::new(TokenType::ASSIGN, "=".to_string())
                }
            }
            b',' => Token::new(TokenType::COMMA, ",".to_string()),
            b'{' => Token::new(TokenType::LBRACE, "{".to_string()),
            b'}' => Token::new(TokenType::RBRACE, "}".to_string()),
            b'(' => Token::new(TokenType::LPAREN, "(".to_string()),
            b')' => Token::new(TokenType::RPAREN, ")".to_string()),
            b'+' => Token::new(TokenType::PLUS, "+".to_string()),
            b'-' => Token::new(TokenType::MINUS, "-".to_string()),
            b'!' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    Token::new(TokenType::NEQ, "!=".to_string())
                } else {
                    Token::new(TokenType::BANG, "!".to_string())
                }
            }
            b'*' => Token::new(TokenType::ASTERISK, "*".to_string()),
            b'/' => Token::new(TokenType::SLASH, "/".to_string()),
            b'<' => Token::new(TokenType::LT, "<".to_string()),
            b'>' => Token::new(TokenType::GT, ">".to_string()),
            b';' => Token::new(TokenType::SEMICOLON, ";".to_string()),
            0 => Token::new(TokenType::EOF, "".to_string()),
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
                    TokenType::ILLEGAL,
                    String::from_utf8(vec![self.ch]).unwrap(),
                )
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
            return self.input[self.read_position];
        }
    }

    /// Advances one byte, setting `ch` to `0` once past the end of input.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
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

        let literal = String::from_utf8(self.input[position..self.position].to_vec()).unwrap();

        match literal.as_str() {
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
    fn read_number(&mut self) -> Token {
        let position = self.position;

        while Lexer::is_digit(self.ch) {
            self.read_char();
        }

        let literal = String::from_utf8(self.input[position..self.position].to_vec()).unwrap();

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
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "five".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "ten".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "add".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::FUNCTION, "fn".to_string()),
            Token::new(TokenType::LPAREN, "(".to_string()),
            Token::new(TokenType::IDENT, "x".to_string()),
            Token::new(TokenType::COMMA, ",".to_string()),
            Token::new(TokenType::IDENT, "y".to_string()),
            Token::new(TokenType::RPAREN, ")".to_string()),
            Token::new(TokenType::LBRACE, "{".to_string()),
            Token::new(TokenType::IDENT, "x".to_string()),
            Token::new(TokenType::PLUS, "+".to_string()),
            Token::new(TokenType::IDENT, "y".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::RBRACE, "}".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "result".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::IDENT, "add".to_string()),
            Token::new(TokenType::LPAREN, "(".to_string()),
            Token::new(TokenType::IDENT, "five".to_string()),
            Token::new(TokenType::COMMA, ",".to_string()),
            Token::new(TokenType::IDENT, "ten".to_string()),
            Token::new(TokenType::RPAREN, ")".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::BANG, "!".to_string()),
            Token::new(TokenType::MINUS, "-".to_string()),
            Token::new(TokenType::SLASH, "/".to_string()),
            Token::new(TokenType::ASTERISK, "*".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::LT, "<".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::GT, ">".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::IF, "if".to_string()),
            Token::new(TokenType::LPAREN, "(".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::LT, "<".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::RPAREN, ")".to_string()),
            Token::new(TokenType::LBRACE, "{".to_string()),
            Token::new(TokenType::RETURN, "return".to_string()),
            Token::new(TokenType::TRUE, "true".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::RBRACE, "}".to_string()),
            Token::new(TokenType::ELSE, "else".to_string()),
            Token::new(TokenType::LBRACE, "{".to_string()),
            Token::new(TokenType::RETURN, "return".to_string()),
            Token::new(TokenType::FALSE, "false".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::RBRACE, "}".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::NEQ, "!=".to_string()),
            Token::new(TokenType::INT, "9".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::EQ, "==".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::EOF, "".to_string()),
        ];

        let mut lexer = Lexer::new(input.to_string());

        for t in tests.iter() {
            let token = lexer.next_token();

            assert_eq!(*t, token);
        }
    }
}
