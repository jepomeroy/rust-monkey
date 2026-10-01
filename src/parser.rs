//! Builds an [`ast`](crate::ast) from the tokens produced by the [`Lexer`].

use crate::{
    ast::{Expression, Identifier, LetStatement, Program, Statement},
    lexer::Lexer,
    token::{
        Token,
        TokenType::{self, ASSIGN},
    },
};

/// Recursive-descent parser that builds a [`Program`] from a [`Lexer`]'s tokens.
///
/// Keeps a one-token lookahead: `curr_token` is being parsed and
/// `peek_token` is the one after it.
pub(crate) struct Parser {
    lexer: Lexer,
    curr_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl Parser {
    /// Creates a parser, reading two tokens to fill `curr_token` and `peek_token`.
    pub(crate) fn new(mut lexer: Lexer) -> Self {
        let curr = lexer.next_token();
        let peek = lexer.next_token();

        Self {
            lexer,
            curr_token: curr,
            peek_token: peek,
            errors: vec![],
        }
    }

    fn curr_token_is(&self, token: TokenType) -> bool {
        self.curr_token.token == token
    }

    fn peek_token_is(&self, token: TokenType) -> bool {
        self.peek_token.token == token
    }

    /// Advances only if the next token is of type `peek`.
    /// Returns whether it advanced.
    fn expect_peek(&mut self, peek: TokenType) -> bool {
        if self.peek_token_is(peek) {
            self.next_token();
            true
        } else {
            self.peek_error(peek);
            false
        }
    }

    fn peek_error(&mut self, token: TokenType) {
        self.errors.push(format!(
            "expected next token to be {:?}, got {:?} instead",
            token, self.peek_token.token
        ));
    }

    pub(crate) fn errors(&self) -> &[String] {
        self.errors.as_slice()
    }

    /// Shifts the lookahead window forward by one token.
    fn next_token(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    /// Parses statements until EOF. Statements that fail to parse are skipped.
    pub(crate) fn parse_program(&mut self) -> Option<Program> {
        let mut program = Program::new();

        while self.curr_token.token != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }

            self.next_token();
        }

        Some(program)
    }

    /// Dispatches on the current token.
    fn parse_statement(&mut self) -> Option<Statement> {
        match self.curr_token.token {
            // TokenType::ILLEGAL => todo!(),
            // TokenType::EOF => todo!(),
            // TokenType::IDENT => todo!(),
            // TokenType::INT => todo!(),
            // TokenType::ASSIGN => todo!(),
            // TokenType::PLUS => todo!(),
            // TokenType::MINUS => todo!(),
            // TokenType::BANG => todo!(),
            // TokenType::ASTERISK => todo!(),
            // TokenType::SLASH => todo!(),
            // TokenType::LT => todo!(),
            // TokenType::GT => todo!(),
            // TokenType::EQ => todo!(),
            // TokenType::NEQ => todo!(),
            // TokenType::COMMA => todo!(),
            // TokenType::SEMICOLON => todo!(),
            // TokenType::LPAREN => todo!(),
            // TokenType::RPAREN => todo!(),
            // TokenType::LBRACE => todo!(),
            // TokenType::RBRACE => todo!(),
            // TokenType::FUNCTION => todo!(),
            TokenType::LET => self.parse_let_statement(),
            // TokenType::TRUE => todo!(),
            // TokenType::FALSE => todo!(),
            // TokenType::IF => todo!(),
            // TokenType::ELSE => todo!(),
            // TokenType::RETURN => todo!(),
            _ => None,
        }
    }

    /// Parses `let <ident> = <expr>;`, starting with `curr_token` on `let`.
    fn parse_let_statement(&mut self) -> Option<Statement> {
        let curr_token = self.curr_token.clone();

        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        let identifier = Identifier {
            token: self.curr_token.token,
            value: self.curr_token.literal.to_string(),
        };

        let stmt = LetStatement {
            token: curr_token,
            name: identifier,
            // value: Box::new({}),
        };

        if !self.expect_peek(ASSIGN) {
            return None;
        }

        while !self.curr_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Statement::Let(stmt))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Node, Statement};

    use super::*;

    fn check_for_parse_errors(p: &Parser) {
        let errors = p.errors();

        for err in errors {
            println!("parse error: {}", err);
        }

        assert_eq!(errors.len(), 0, "parser had {} errors", errors.len());
    }

    #[test]
    fn test_let_statements() {
        let input = r"
            let x = 5;
            let y = 10;
            let foobar = 838383;";

        let lexer = Lexer::new(input.to_string());
        let mut p = Parser::new(lexer);
        let program = p.parse_program();
        check_for_parse_errors(&p);

        assert!(program.is_some());

        if let Some(prog) = program {
            assert_eq!(prog.statements.len(), 3);

            let expected = vec!["x".to_string(), "y".to_string(), "foobar".to_string()];

            println!("{:?}", prog.statements);

            for (stmt, expected) in prog.statements.iter().zip(expected) {
                assert!(&stmt.token_literal() == "let");

                let Statement::Let(let_stmt) = &stmt else {
                    panic!("Expected let statement")
                };

                assert_eq!(let_stmt.name.token_literal(), expected);
            }
        }
    }
}
