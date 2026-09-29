use crate::token::Token;
use crate::token::TokenType;

#[derive(Default)]
pub(crate) struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub(crate) fn new(input: String) -> Self {
        let mut lexer = Self {
            input: input.as_bytes().to_vec(),
            ..Default::default()
        };
        lexer.read_char();
        lexer
    }

    pub(crate) fn next_token(&mut self) -> Token {
        let token = match self.ch {
            b'=' => Token::new(TokenType::ASSIGN, "=".to_string()),
            b',' => Token::new(TokenType::COMMA, ",".to_string()),
            b'{' => Token::new(TokenType::LBRACE, "{".to_string()),
            b'}' => Token::new(TokenType::RBRACE, "}".to_string()),
            b'(' => Token::new(TokenType::LPAREN, "(".to_string()),
            b')' => Token::new(TokenType::RPAREN, ")".to_string()),
            b'+' => Token::new(TokenType::PLUS, "+".to_string()),
            b';' => Token::new(TokenType::SEMICOLON, ";".to_string()),
            0 => Token::new(TokenType::EOF, "".to_string()),
            _ => Token::new(
                TokenType::ILLEGAL,
                String::from_utf8(vec![self.ch]).unwrap(),
            ),
        };

        self.read_char();

        token
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_next_token() {
        let input = "=+(){},;";

        let tests = [
            Token {
                token: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token: TokenType::PLUS,
                literal: "+".to_string(),
            },
            Token {
                token: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Token {
                token: TokenType::RPAREN,
                literal: ")".to_string(),
            },
            Token {
                token: TokenType::LBRACE,
                literal: "{".to_string(),
            },
            Token {
                token: TokenType::RBRACE,
                literal: "}".to_string(),
            },
            Token {
                token: TokenType::COMMA,
                literal: ",".to_string(),
            },
            Token {
                token: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token: TokenType::EOF,
                literal: "".to_string(),
            },
        ];

        let mut lexer = Lexer::new(input.to_string());

        for t in tests.iter() {
            let token = lexer.next_token();

            assert_eq!(t.token, token.token);
            assert_eq!(t.literal, token.literal)
        }
    }

    #[test]
    #[ignore = "not ready to implement"]
    fn test_next_token() {
        let input = r"let five = 5;
let ten = 10;

let add = fn(x, y) {
  x + y;
};

let result = add(five, ten);
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
            Token::new(TokenType::EOF, "".to_string()),
        ];

        let mut lexer = Lexer::new(input.to_string());

        for t in tests.iter() {
            let token = lexer.next_token();

            assert_eq!(t.token, token.token);
            assert_eq!(t.literal, token.literal)
        }
    }
}
