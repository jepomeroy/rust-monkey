//! Builds an [`ast`](crate::ast) from the tokens produced by the [`Lexer`].

use std::collections::HashMap;

use crate::{
    ast::{
        Expression, Identifier, IntegerLiteral, LetStatement, Program, ReturnStatement,
        Statement::{self, ExpressionStmt},
    },
    lexer::Lexer,
    token::{
        Token,
        TokenType::{self, ASSIGN},
    },
};

pub(crate) enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // < or >
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunc(X)
    Index,       // array[idx]
}

type PrefixParseFn<'p> = fn(&mut Parser<'p>) -> Option<Expression<'p>>;
type InfixParseFn<'p> = fn(Expression<'p>) -> Option<Expression<'p>>;

/// Recursive-descent parser that builds a [`Program`] from a [`Lexer`]'s tokens.
///
/// Keeps a one-token lookahead: `curr_token` is being parsed and
/// `peek_token` is the one after it.
#[derive(Debug)]
pub(crate) struct Parser<'p> {
    lexer: Lexer<'p>,
    curr_token: Token<'p>,
    peek_token: Token<'p>,
    errors: Vec<String>,
    prefix_parse_fns: HashMap<TokenType, PrefixParseFn<'p>>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn<'p>>,
}

impl<'p> Parser<'p> {
    /// Creates a parser, reading two tokens to fill `curr_token` and `peek_token`.
    pub(crate) fn new(mut lexer: Lexer<'p>) -> Self {
        let curr = lexer.next_token();
        let peek = lexer.next_token();
        let prefix_parse_fns = HashMap::new();
        let infix_parse_fns = HashMap::new();

        let mut parser = Self {
            lexer,
            curr_token: curr,
            peek_token: peek,
            errors: vec![],
            prefix_parse_fns,
            infix_parse_fns,
        };

        // populate prefix parse fns
        parser.register_prefix(TokenType::IDENT, Self::parse_identifier);
        parser.register_prefix(TokenType::INT, Self::parse_integer);

        // populate infix parse fns

        parser
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
    pub(crate) fn parse_program(&mut self) -> Program<'p> {
        let mut program = Program::new();

        while self.curr_token.token != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }

            self.next_token();
        }

        program
    }

    /// Dispatches on the current token.
    fn parse_statement(&mut self) -> Option<Statement<'p>> {
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
            TokenType::RETURN => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_expression_statement(&mut self) -> Option<Statement<'p>> {
        let expr = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(ExpressionStmt(expr?))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression<'p>> {
        if let Some(prefix) = self.prefix_parse_fns.get(&self.curr_token.token) {
            return prefix(self);
        }

        None
    }

    fn parse_identifier(&mut self) -> Option<Expression<'p>> {
        Some(Expression::Ident(Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal,
        }))
    }

    fn parse_integer(&mut self) -> Option<Expression<'p>> {
        match self.curr_token.literal.parse::<i64>() {
            Ok(int) => Some(Expression::Integer(IntegerLiteral {
                token: self.curr_token.clone(),
                value: int,
            })),
            Err(_) => {
                self.errors.push(format!(
                    "could not parse {} as integer",
                    self.curr_token.literal
                ));

                None
            }
        }
    }

    /// Parses `let <ident> = <expr>;`, starting with `curr_token` on `let`.
    fn parse_let_statement(&mut self) -> Option<Statement<'p>> {
        let curr_token = self.curr_token.clone();

        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        let identifier = Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal,
        };

        let stmt = LetStatement {
            token: curr_token,
            name: identifier,
            // value: Box::new({}),
        };

        if !self.expect_peek(ASSIGN) {
            return None;
        }

        // TODO: We're skipping the expressions until we
        // encounter a semicolon
        while !self.curr_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Statement::Let(stmt))
    }

    /// Parses `return <expr>;`, starting with `curr_token` on `return`.
    fn parse_return_statement(&mut self) -> Option<Statement<'p>> {
        let curr_token = self.curr_token.clone();

        let stmt = ReturnStatement {
            token: curr_token,
            // value: Box<Expression>
        };

        // TODO: We're skipping the expressions until we
        // encounter a semicolon
        while !self.curr_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Statement::Return(stmt))
    }

    fn register_prefix(&mut self, token: TokenType, func: PrefixParseFn<'p>) {
        self.prefix_parse_fns.insert(token, func);
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

    fn make_program(input: &str) -> Parser<'_> {
        let lexer = Lexer::new(input);
        let p = Parser::new(lexer);
        println!("{:?}", p);
        check_for_parse_errors(&p);

        p
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let mut parser = make_program(input);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                Statement::ExpressionStmt(identifier) => {
                    assert_eq!(identifier.string(), "foobar");
                    assert_eq!(identifier.token_literal(), "foobar");
                }
                _ => panic!("Expected identifier, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_integer_expression() {
        let input = "5;";

        let mut parser = make_program(input);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                Statement::ExpressionStmt(identifier) => {
                    assert_eq!(identifier.string(), "5");
                    assert_eq!(identifier.token_literal(), "5");
                }
                _ => panic!("Expected Integer, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_let_statements() {
        let input = r"
            let x = 5;
            let y = 10;
            let foobar = 838383;";

        let mut parser = make_program(input);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 3);

        let expected = vec!["x".to_string(), "y".to_string(), "foobar".to_string()];

        println!("{:?}", program.statements);

        for (stmt, expected) in program.statements.iter().zip(expected) {
            assert!(stmt.token_literal() == "let");

            match stmt {
                Statement::Let(let_stmt) => {
                    assert_eq!(let_stmt.name.value, expected);
                    assert_eq!(let_stmt.name.token_literal(), expected);
                }
                _ => panic!("Expected let statement, got {:?}", stmt),
            };
        }
    }

    #[test]
    fn test_return_statements() {
        let input = r"
            return 5;
            return 10;
            return 838383;";

        let mut parser = make_program(input);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 3);

        let expected = vec!["x".to_string(), "y".to_string(), "foobar".to_string()];

        println!("{:?}", program.statements);

        for (stmt, _expected) in program.statements.iter().zip(expected) {
            assert!(stmt.token_literal() == "return");

            match stmt {
                Statement::Return(ret_stmt) => {
                    assert_eq!(ret_stmt.token_literal(), "return");
                }
                _ => panic!("Expected return statement"),
            };
        }
    }
}
