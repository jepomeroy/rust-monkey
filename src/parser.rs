//! Builds an [`ast`](crate::ast) from the tokens produced by the [`Lexer`].

use std::collections::HashMap;

use crate::{
    ast::{
        Boolean,
        Expression::{self, InfixExpr, PrefixExpr},
        Identifier, Infix, IntegerLiteral, LetStatement, Prefix, Program, ReturnStatement,
        Statement::{self, ExpressionStmt},
    },
    lexer::Lexer,
    token::{
        Token,
        TokenType::{self, ASSIGN},
    },
};

#[derive(PartialEq, PartialOrd, Debug)]
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

fn map_precedence(token: TokenType) -> Precedence {
    match token {
        TokenType::PLUS => Precedence::Sum,
        TokenType::MINUS => Precedence::Sum,
        TokenType::ASTERISK => Precedence::Product,
        TokenType::SLASH => Precedence::Product,
        TokenType::LT => Precedence::LessGreater,
        TokenType::GT => Precedence::LessGreater,
        TokenType::EQ => Precedence::Equals,
        TokenType::NEQ => Precedence::Equals,
        _ => Precedence::Lowest,
    }
}

type PrefixParseFn<'p> = fn(&mut Parser<'p>) -> Option<Expression<'p>>;
type InfixParseFn<'p> = fn(&mut Parser<'p>, Option<Expression<'p>>) -> Option<Expression<'p>>;

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
        parser.register_prefix(TokenType::TRUE, Self::parse_boolean);
        parser.register_prefix(TokenType::FALSE, Self::parse_boolean);
        parser.register_prefix(TokenType::LPAREN, Self::parse_grouped_expression);
        parser.register_prefix(TokenType::IDENT, Self::parse_identifier);
        parser.register_prefix(TokenType::INT, Self::parse_integer);
        parser.register_prefix(TokenType::BANG, Self::parse_prefix);
        parser.register_prefix(TokenType::MINUS, Self::parse_prefix);

        // populate infix parse fns
        parser.register_infix(TokenType::PLUS, Self::parse_infix);
        parser.register_infix(TokenType::MINUS, Self::parse_infix);
        parser.register_infix(TokenType::SLASH, Self::parse_infix);
        parser.register_infix(TokenType::ASTERISK, Self::parse_infix);
        parser.register_infix(TokenType::LT, Self::parse_infix);
        parser.register_infix(TokenType::GT, Self::parse_infix);
        parser.register_infix(TokenType::EQ, Self::parse_infix);
        parser.register_infix(TokenType::NEQ, Self::parse_infix);

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

    fn no_prefix_parse_fn_error(&mut self, token: TokenType) {
        self.errors
            .push(format!("no prefix parse function for {:?} found", token));
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

    fn parse_boolean(&mut self) -> Option<Expression<'p>> {
        Some(Expression::Boolean(Boolean {
            token: self.curr_token.clone(),
            value: self.curr_token_is(TokenType::TRUE),
        }))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement<'p>> {
        let expr = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(ExpressionStmt(expr?))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression<'p>> {
        if let Some(prefix) = self.prefix_parse_fns.get(&self.curr_token.token).copied() {
            let mut left = prefix(self);

            while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
                let Some(infix) = self.infix_parse_fns.get(&self.peek_token.token).copied() else {
                    return left;
                };

                self.next_token();
                left = infix(self, left);
            }

            return left;
        }

        self.no_prefix_parse_fn_error(self.curr_token.token);

        None
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression<'p>> {
        self.next_token();
        let expr = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }

        expr
    }

    fn parse_identifier(&mut self) -> Option<Expression<'p>> {
        Some(Expression::Ident(Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal,
        }))
    }

    fn parse_infix(&mut self, left: Option<Expression<'p>>) -> Option<Expression<'p>> {
        if let Some(left_expr) = left {
            let token = self.curr_token.clone();
            let precedence = self.curr_precedence();
            self.next_token();
            if let Some(right_expr) = self.parse_expression(precedence) {
                return Some(InfixExpr(Infix {
                    token: token.clone(),
                    left: Box::new(left_expr),
                    operator: token.literal,
                    right: Box::new(right_expr),
                }));
            }
        }

        None
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

    /// Parses `!5` and `-15` expression starting with `curr_token` on the prefix
    fn parse_prefix(&mut self) -> Option<Expression<'p>> {
        let token = self.curr_token.clone();

        self.next_token();

        if let Some(right) = self.parse_expression(Precedence::Prefix) {
            let expr = PrefixExpr(Prefix {
                token: token.clone(),
                operator: token.literal,
                right: Box::new(right),
            });

            return Some(expr);
        }

        None
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

    fn peek_precedence(&self) -> Precedence {
        map_precedence(self.peek_token.token)
    }

    fn curr_precedence(&self) -> Precedence {
        map_precedence(self.curr_token.token)
    }

    fn register_infix(&mut self, token: TokenType, func: InfixParseFn<'p>) {
        self.infix_parse_fns.insert(token, func);
    }

    fn register_prefix(&mut self, token: TokenType, func: PrefixParseFn<'p>) {
        self.prefix_parse_fns.insert(token, func);
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::ast::{Node, Statement};

    use super::*;

    #[derive(Debug, Clone, Copy)]
    enum Value<'a> {
        Int(i64),
        Ident(&'a str),
        Boolean(bool),
    }

    // These let callers pass plain values: test_infix_expression(&e, 5, "+", "x")
    impl From<i64> for Value<'_> {
        fn from(v: i64) -> Self {
            Value::Int(v)
        }
    }
    impl From<i32> for Value<'_> {
        fn from(v: i32) -> Self {
            Value::Int(v as i64)
        }
    }
    impl<'a> From<&'a str> for Value<'a> {
        fn from(v: &'a str) -> Self {
            Value::Ident(v)
        }
    }

    impl<'a> From<bool> for Value<'_> {
        fn from(v: bool) -> Self {
            Value::Boolean(v)
        }
    }

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

        p
    }

    fn test_boolean_literal(expr: &Expression, value: bool) {
        match expr {
            Expression::Boolean(boolean) => {
                assert_eq!(boolean.value, value, "boolean mismatch");
                assert_eq!(
                    boolean.token_literal(),
                    value.to_string(),
                    "token literal mismatch"
                );
            }
            other => panic!("exp is not Boolean. got={other:?}"),
        }
    }

    fn test_identifier(expr: &Expression, value: &str) {
        match expr {
            Expression::Ident(identifier) => {
                assert_eq!(identifier.value, value, "identifier mismatch");
                assert_eq!(identifier.token_literal(), value, "token literal mismatch");
            }
            other => panic!("exp is not Identifier. got={other:?}"),
        }
    }

    fn test_integer_literal(expr: &Expression, value: i64) {
        match expr {
            Expression::Integer(integer) => {
                assert_eq!(integer.value, value, "integer value mismatch")
            }
            other => panic!("Expected Integer, got {other:?}"),
        }
    }

    #[test]
    fn test_boolean_expression() {
        let input = r"true; false;";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 2);

        let expected = [true, false];

        for (stmt, exp) in program.statements.iter().zip(expected) {
            match stmt {
                Statement::ExpressionStmt(boolean) => {
                    assert_eq!(boolean.string(), exp.to_string());
                    assert_eq!(boolean.token_literal(), exp.to_string());
                }
                _ => panic!("Expected Boolean, got {:?}", stmt),
            }
        }
    }

    fn test_literal_expression<'a>(exp: &Expression, expected: impl Into<Value<'a>>) {
        match expected.into() {
            Value::Boolean(v) => test_boolean_literal(exp, v),
            Value::Ident(v) => test_identifier(exp, v),
            Value::Int(v) => test_integer_literal(exp, v),
        }
    }

    fn test_infix_expression<'a>(
        exp: &Expression,
        left: impl Into<Value<'a>>,
        operator: &str,
        right: impl Into<Value<'a>>,
    ) {
        let Expression::InfixExpr(infix) = exp else {
            panic!("exp is not InfixExpression. got={exp:?}");
        };

        test_literal_expression(&infix.left, left);
        assert_eq!(infix.operator, operator, "exp.operator is not '{operator}'");
        test_literal_expression(&infix.right, right);
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

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
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                Statement::ExpressionStmt(integer) => {
                    assert_eq!(integer.string(), "5");
                    assert_eq!(integer.token_literal(), "5");
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
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        let expected = vec!["x".to_string(), "y".to_string(), "foobar".to_string()];

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

    struct OpPrecedenceInputs<'opi> {
        input: &'opi str,
        expected: &'opi str,
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let inputs = vec![
            OpPrecedenceInputs {
                input: "-a * b",
                expected: "((-a) * b)",
            },
            OpPrecedenceInputs {
                input: "!-a",
                expected: "(!(-a))",
            },
            OpPrecedenceInputs {
                input: "a + b + c",
                expected: "((a + b) + c)",
            },
            OpPrecedenceInputs {
                input: "a + b - c",
                expected: "((a + b) - c)",
            },
            OpPrecedenceInputs {
                input: "a * b * c",
                expected: "((a * b) * c)",
            },
            OpPrecedenceInputs {
                input: "a * b / c",
                expected: "((a * b) / c)",
            },
            OpPrecedenceInputs {
                input: "a + b / c",
                expected: "(a + (b / c))",
            },
            OpPrecedenceInputs {
                input: "a + b * c + d / e - f",
                expected: "(((a + (b * c)) + (d / e)) - f)",
            },
            OpPrecedenceInputs {
                input: "3 + 4; -5 * 5",
                expected: "(3 + 4)((-5) * 5)",
            },
            OpPrecedenceInputs {
                input: "5 > 4 == 3 < 4",
                expected: "((5 > 4) == (3 < 4))",
            },
            OpPrecedenceInputs {
                input: "5 > 4 != 3 > 4",
                expected: "((5 > 4) != (3 > 4))",
            },
            OpPrecedenceInputs {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5",
                expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            },
            OpPrecedenceInputs {
                input: "true",
                expected: "true",
            },
            OpPrecedenceInputs {
                input: "false",
                expected: "false",
            },
            OpPrecedenceInputs {
                input: "3 > 5 == false",
                expected: "((3 > 5) == false)",
            },
            OpPrecedenceInputs {
                input: "3 < 5 == true",
                expected: "((3 < 5) == true)",
            },
            OpPrecedenceInputs {
                input: "1 + (2 + 3) + 4",
                expected: "((1 + (2 + 3)) + 4)",
            },
            OpPrecedenceInputs {
                input: "(5 + 5) * 2",
                expected: "((5 + 5) * 2)",
            },
            OpPrecedenceInputs {
                input: "2 / (5 + 5)",
                expected: "(2 / (5 + 5))",
            },
            OpPrecedenceInputs {
                input: "-(5 + 5)",
                expected: "(-(5 + 5))",
            },
            OpPrecedenceInputs {
                input: "!(true == true)",
                expected: "(!(true == true))",
            },
        ];

        for input in inputs {
            let mut parser = make_program(input.input);
            let program = parser.parse_program();
            check_for_parse_errors(&parser);

            println!("{:?}\n\n", program.string());

            let actual = program.string();
            assert_eq!(actual, input.expected);
        }
    }

    struct InfixInputs<'pi> {
        input: &'pi str,
        left_value: Value<'pi>,
        operator: &'pi str,
        right_value: Value<'pi>,
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let inputs = vec![
            InfixInputs {
                input: "5 + 5",
                left_value: Value::Int(5),
                operator: "+",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 - 5",
                left_value: Value::Int(5),
                operator: "-",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 * 5",
                left_value: Value::Int(5),
                operator: "*",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 / 5",
                left_value: Value::Int(5),
                operator: "/",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 > 5",
                left_value: Value::Int(5),
                operator: ">",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 < 5",
                left_value: Value::Int(5),
                operator: "<",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 == 5",
                left_value: Value::Int(5),
                operator: "==",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "5 != 5",
                left_value: Value::Int(5),
                operator: "!=",
                right_value: Value::Int(5),
            },
            InfixInputs {
                input: "true == true",
                left_value: Value::Boolean(true),
                operator: "==",
                right_value: Value::Boolean(true),
            },
            InfixInputs {
                input: "true != false",
                left_value: Value::Boolean(true),
                operator: "!=",
                right_value: Value::Boolean(false),
            },
            InfixInputs {
                input: "false == false",
                left_value: Value::Boolean(false),
                operator: "==",
                right_value: Value::Boolean(false),
            },
        ];

        for input in inputs {
            let mut parser = make_program(input.input);
            let program = parser.parse_program();
            check_for_parse_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            for stmt in program.statements {
                match stmt {
                    Statement::ExpressionStmt(expr) => match expr {
                        Expression::InfixExpr(infix) => {
                            test_literal_expression(&*infix.left, input.left_value);
                            assert_eq!(infix.operator, input.operator);
                            test_literal_expression(&*infix.right, input.right_value);
                        }
                        _ => panic!("Expected Expression Statement, got {:?}", expr),
                    },
                    _ => panic!("Expected Expression Statement, got {:?}", stmt),
                }
            }
        }
    }

    struct PrefixInputs<'pi> {
        input: &'pi str,
        operator: &'pi str,
        value: Value<'pi>,
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let inputs = vec![
            PrefixInputs {
                input: "!5",
                operator: "!",
                value: Value::Int(5),
            },
            PrefixInputs {
                input: "-15",
                operator: "-",
                value: Value::Int(15),
            },
            PrefixInputs {
                input: "!true",
                operator: "!",
                value: Value::Boolean(true),
            },
            PrefixInputs {
                input: "!false",
                operator: "!",
                value: Value::Boolean(false),
            },
        ];

        for input in inputs {
            let mut parser = make_program(input.input);
            let program = parser.parse_program();
            check_for_parse_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            for stmt in program.statements {
                match stmt {
                    Statement::ExpressionStmt(expr) => match expr {
                        Expression::PrefixExpr(prefix) => {
                            assert_eq!(prefix.operator, input.operator);
                            test_literal_expression(&*prefix.right, input.value);
                        }
                        _ => panic!("Expected Expression Statement, got {:?}", expr),
                    },
                    _ => panic!("Expected Expression Statement, got {:?}", stmt),
                }
            }
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
        check_for_parse_errors(&parser);

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
