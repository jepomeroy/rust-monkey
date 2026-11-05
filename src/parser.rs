//! Builds an [`ast`](crate::ast) from the tokens produced by the [`Lexer`].

use std::collections::HashMap;

use crate::{
    ast::{
        Array, BlockStatement, Boolean, CallExpression,
        Expression::{self, StringExpr},
        FunctionLiteral, HashLiteral, Identifier, IfExpression, IndexExpression, Infix,
        IntegerLiteral, LetStatement, Prefix, Program, ReturnStatement,
        Statement::{self, ExpressionStmt},
        StringExpression,
    },
    lexer::Lexer,
    parser::Precedence::Lowest,
    token::{
        Token,
        TokenType::{self},
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
        TokenType::Plus => Precedence::Sum,
        TokenType::Minus => Precedence::Sum,
        TokenType::Asterisk => Precedence::Product,
        TokenType::Slash => Precedence::Product,
        TokenType::Lt => Precedence::LessGreater,
        TokenType::Gt => Precedence::LessGreater,
        TokenType::Eq => Precedence::Equals,
        TokenType::Neq => Precedence::Equals,
        TokenType::Lparen => Precedence::Call,
        TokenType::Lbracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}

type PrefixParseFn<'p> = fn(&mut Parser<'p>) -> Option<Expression>;
type InfixParseFn<'p> = fn(&mut Parser<'p>, Option<Expression>) -> Option<Expression>;

/// Recursive-descent parser that builds a [`Program`] from a [`Lexer`]'s tokens.
///
/// Keeps a one-token lookahead: `curr_token` is being parsed and
/// `peek_token` is the one after it.
#[derive(Debug)]
pub(crate) struct Parser<'p> {
    lexer: Lexer<'p>,
    curr_token: Token,
    peek_token: Token,
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
        parser.register_prefix(TokenType::True, Self::parse_boolean);
        parser.register_prefix(TokenType::False, Self::parse_boolean);
        parser.register_prefix(TokenType::Function, Self::parse_function_literal);
        parser.register_prefix(TokenType::Lparen, Self::parse_grouped_expression);
        parser.register_prefix(TokenType::Ident, Self::parse_identifier);
        parser.register_prefix(TokenType::If, Self::parse_if_expression);
        parser.register_prefix(TokenType::Int, Self::parse_integer);
        parser.register_prefix(TokenType::Bang, Self::parse_prefix);
        parser.register_prefix(TokenType::Minus, Self::parse_prefix);
        parser.register_prefix(TokenType::String, Self::parse_string_literal);
        parser.register_prefix(TokenType::Lbracket, Self::parse_array_literal);
        parser.register_prefix(TokenType::Lbrace, Self::parse_hash_literal);

        // populate infix parse fns
        parser.register_infix(TokenType::Plus, Self::parse_infix);
        parser.register_infix(TokenType::Minus, Self::parse_infix);
        parser.register_infix(TokenType::Slash, Self::parse_infix);
        parser.register_infix(TokenType::Asterisk, Self::parse_infix);
        parser.register_infix(TokenType::Lt, Self::parse_infix);
        parser.register_infix(TokenType::Gt, Self::parse_infix);
        parser.register_infix(TokenType::Eq, Self::parse_infix);
        parser.register_infix(TokenType::Neq, Self::parse_infix);
        parser.register_infix(TokenType::Lparen, Self::parse_call_expression);
        parser.register_infix(TokenType::Lbracket, Self::parse_index_expression);

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
            "expected next token to be {}, got {} instead",
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
    pub(crate) fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while self.curr_token.token != TokenType::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }

            self.next_token();
        }

        program
    }

    /// Dispatches on the current token.
    fn parse_statement(&mut self) -> Option<Statement> {
        match self.curr_token.token {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let curr = self.curr_token.clone();
        let mut statements = Vec::<Statement>::new();

        self.next_token();

        while !self.curr_token_is(TokenType::Rbrace) && !self.curr_token_is(TokenType::Eof) {
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            }

            self.next_token();
        }

        BlockStatement {
            token: curr,
            statements,
        }
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let curr = self.curr_token.clone();

        let elements = self
            .parse_expression_list(TokenType::Rbracket)
            .unwrap_or(vec![]);

        Some(Expression::Array(Array {
            token: curr,
            elements,
        }))
    }

    fn parse_boolean(&mut self) -> Option<Expression> {
        Some(Expression::Boolean(Boolean {
            token: self.curr_token.clone(),
            value: self.curr_token_is(TokenType::True),
        }))
    }

    fn parse_call_expression(&mut self, function: Option<Expression>) -> Option<Expression> {
        let curr = self.curr_token.clone();
        let arguments = self
            .parse_expression_list(TokenType::Rparen)
            .unwrap_or(vec![]);

        if let Some(func) = function {
            return Some(Expression::Call(CallExpression {
                token: curr,
                function: Box::new(func),
                arguments,
            }));
        }

        None
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Option<Vec<Expression>> {
        let mut list = vec![];

        if self.peek_token_is(end) {
            self.next_token();
            return Some(list);
        }

        self.next_token();

        if let Some(expr) = self.parse_expression(Lowest) {
            list.push(expr);
        }

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            if let Some(expr) = self.parse_expression(Lowest) {
                list.push(expr);
            }
        }

        if !self.expect_peek(end) {
            return None;
        }

        Some(list)
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Some(ExpressionStmt(expr?))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        if let Some(prefix) = self.prefix_parse_fns.get(&self.curr_token.token).copied() {
            let mut left = prefix(self);

            while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
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

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let curr = self.curr_token.clone();

        if !self.expect_peek(TokenType::Lparen) {
            return None;
        }

        let parameters = self.parse_function_parameters();

        if !self.expect_peek(TokenType::Lbrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expression::Function(FunctionLiteral {
            token: curr,
            parameters,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Vec<Expression> {
        let mut parameters = Vec::new();

        if self.peek_token_is(TokenType::Rparen) {
            self.next_token();
            return parameters;
        }

        self.next_token();

        let ident = Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        };

        parameters.push(Expression::Ident(ident));

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();

            let ident = Identifier {
                token: self.curr_token.clone(),
                value: self.curr_token.literal.clone(),
            };

            parameters.push(Expression::Ident(ident));
        }

        if !self.expect_peek(TokenType::Rparen) {
            return Vec::new();
        }

        parameters
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let expr = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::Rparen) {
            return None;
        }

        expr
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let curr = self.curr_token.clone();
        let mut pairs = vec![];

        while !self.peek_token_is(TokenType::Rbrace) {
            self.next_token();
            let key = self.parse_expression(Lowest)?;

            if !self.expect_peek(TokenType::Colon) {
                return None;
            }

            self.next_token();
            let value = self.parse_expression(Lowest)?;

            pairs.push((key, value));

            if !self.peek_token_is(TokenType::Rbrace) && !self.expect_peek(TokenType::Comma) {
                return None;
            }
        }

        if !self.expect_peek(TokenType::Rbrace) {
            return None;
        }

        Some(Expression::Hash(HashLiteral { token: curr, pairs }))
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        Some(Expression::Ident(Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        }))
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let curr = self.curr_token.clone();

        if !self.expect_peek(TokenType::Lparen) {
            return None;
        }

        self.next_token();

        if let Some(condition) = self.parse_expression(Precedence::Lowest) {
            if !&self.expect_peek(TokenType::Rparen) {
                return None;
            }

            if !&self.expect_peek(TokenType::Lbrace) {
                return None;
            }

            let consequence = self.parse_block_statement();

            let mut alternative: Option<BlockStatement> = None;

            if self.peek_token_is(TokenType::Else) {
                self.next_token();

                if !self.expect_peek(TokenType::Lbrace) {
                    return None;
                }

                alternative = Some(self.parse_block_statement());
            };

            return Some(Expression::IfExpr(IfExpression {
                token: curr,
                condition: Box::new(condition),
                consequence,
                alternative,
            }));
        }

        None
    }

    fn parse_index_expression(&mut self, left: Option<Expression>) -> Option<Expression> {
        if let Some(left) = left {
            let curr = self.curr_token.clone();

            self.next_token();
            if let Some(index) = self.parse_expression(Lowest) {
                if !self.expect_peek(TokenType::Rbracket) {
                    return None;
                }

                return Some(Expression::Index(IndexExpression {
                    token: curr,
                    left: Box::new(left),
                    index: Box::new(index),
                }));
            }
        }

        None
    }

    fn parse_infix(&mut self, left: Option<Expression>) -> Option<Expression> {
        if let Some(left_expr) = left {
            let token = self.curr_token.clone();
            let precedence = self.curr_precedence();
            self.next_token();
            if let Some(right_expr) = self.parse_expression(precedence) {
                return Some(Expression::InfixExpr(Infix {
                    token: token.clone(),
                    left: Box::new(left_expr),
                    operator: token.literal,
                    right: Box::new(right_expr),
                }));
            }
        }

        None
    }

    fn parse_integer(&mut self) -> Option<Expression> {
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
    fn parse_let_statement(&mut self) -> Option<Statement> {
        let curr_token = self.curr_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        let identifier = Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        self.next_token();

        if let Some(value) = self.parse_expression(Precedence::Lowest) {
            let stmt = LetStatement {
                token: curr_token,
                name: identifier,
                value: Box::new(value),
            };

            if self.peek_token_is(TokenType::Semicolon) {
                self.next_token();
            }

            return Some(Statement::Let(stmt));
        }

        None
    }

    /// Parses `!5` and `-15` expression starting with `curr_token` on the prefix
    fn parse_prefix(&mut self) -> Option<Expression> {
        let token = self.curr_token.clone();

        self.next_token();

        if let Some(right) = self.parse_expression(Precedence::Prefix) {
            return Some(Expression::PrefixExpr(Prefix {
                token: token.clone(),
                operator: token.literal,
                right: Box::new(right),
            }));
        }

        None
    }

    /// Parses `return <expr>;`, starting with `curr_token` on `return`.
    fn parse_return_statement(&mut self) -> Option<Statement> {
        let curr = self.curr_token.clone();

        self.next_token();

        if let Some(value) = self.parse_expression(Precedence::Lowest) {
            let stmt = ReturnStatement {
                token: curr,
                value: Box::new(value),
            };

            if self.peek_token_is(TokenType::Semicolon) {
                self.next_token();
            }

            return Some(Statement::Return(stmt));
        }

        None
    }

    fn parse_string_literal(&mut self) -> Option<Expression> {
        Some(StringExpr(StringExpression {
            token: self.curr_token.clone(),
            value: self.curr_token.literal.clone(),
        }))
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
    use std::vec;

    use crate::ast::{NodeType, Statement};
    use crate::test_utils::{InputExpected, Value};

    use super::*;

    struct LetTestData<'ltd, T> {
        input: &'ltd str,
        expected_ident: &'ltd str,
        expected: T,
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
        Parser::new(lexer)
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

    fn test_literal_expression<'a>(exp: &Expression, expected: impl Into<Value<'a>>) {
        match expected.into() {
            Value::Boolean(v) => test_boolean_literal(exp, v),
            Value::Message(v) => test_identifier(exp, v),
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

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                Statement::ExpressionStmt(call) => match call {
                    Expression::Call(call_expr) => {
                        test_identifier(&call_expr.function, "add");
                        assert_eq!(call_expr.arguments.len(), 3);
                        test_literal_expression(call_expr.arguments.first().unwrap(), 1);
                        test_infix_expression(call_expr.arguments.get(1).unwrap(), 2, "*", 3);
                        test_infix_expression(call_expr.arguments.get(2).unwrap(), 4, "+", 5);
                    }
                    other => panic!("Expected Call Expression, got {other:?}"),
                },
                _ => panic!("Expected Call Statement, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_function_literal() {
        let input = "fn(x, y) { x + y; }";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            assert!(stmt.token_literal() == "fn");

            match stmt {
                Statement::ExpressionStmt(func) => match func {
                    Expression::Function(func_literal) => {
                        // test parameters
                        assert_eq!(func_literal.parameters.len(), 2, "Expected 2 parameters");
                        test_literal_expression(func_literal.parameters.first().unwrap(), "x");
                        test_literal_expression(func_literal.parameters.get(1).unwrap(), "y");

                        // test body statements
                        assert_eq!(
                            func_literal.body.statements.len(),
                            1,
                            "Expected 1 block statement"
                        );

                        let expr = match func_literal.body.statements.first().unwrap() {
                            ExpressionStmt(expression) => expression,
                            other => panic!("Expected body statement, got {other:?}"),
                        };

                        test_infix_expression(expr, "x", "+", "y");
                    }
                    other => panic!("Expected if expression, got {other:?}"),
                },
                _ => panic!("Expected Function Expression, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_function_parameter_parsing() {
        let tests = vec![
            InputExpected::<Vec<&str>> {
                input: "fn() {};",
                expected: vec![],
            },
            InputExpected::<Vec<&str>> {
                input: "fn(x) {};",
                expected: vec!["x"],
            },
            InputExpected::<Vec<&str>> {
                input: "fn(x, y, z) {};",
                expected: vec!["x", "y", "z"],
            },
        ];

        for ie in tests {
            let mut parser = make_program(ie.input);
            let program = parser.parse_program();
            check_for_parse_errors(&parser);

            for stmt in program.statements {
                assert!(stmt.token_literal() == "fn");

                match stmt {
                    Statement::ExpressionStmt(func) => match func {
                        Expression::Function(func_literal) => {
                            // test parameters
                            assert_eq!(
                                func_literal.parameters.len(),
                                ie.expected.len(),
                                "Expected {} parameters",
                                ie.expected.len()
                            );

                            for (p, e) in func_literal.parameters.iter().zip(&ie.expected) {
                                test_literal_expression(p, *e);
                            }
                        }
                        other => panic!("Expected if expression, got {other:?}"),
                    },
                    _ => panic!("Expected Function Expression, got {:?}", stmt),
                }
            }
        }
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
    fn test_if_expression() {
        let input = "if (x < y) { x }";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            assert!(stmt.token_literal() == "if");

            match stmt {
                Statement::ExpressionStmt(if_stmt) => match if_stmt {
                    Expression::IfExpr(if_expr) => {
                        // check condition
                        test_infix_expression(&if_expr.condition, "x", "<", "y");

                        // test consequence
                        assert_eq!(
                            if_expr.consequence.statements.len(),
                            1,
                            "consequence is not 1 statement"
                        );

                        let conseqence = match if_expr.consequence.statements.first().unwrap() {
                            ExpressionStmt(expression) => expression,
                            other => panic!("Expected consequence statement, got {other:?}"),
                        };

                        test_identifier(conseqence, "x");
                    }
                    other => panic!("Expected if expression, got {other:?}"),
                },
                _ => panic!("Expected if statement, got {:?}", stmt),
            };
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            assert!(stmt.token_literal() == "if");

            match stmt {
                Statement::ExpressionStmt(if_stmt) => match if_stmt {
                    Expression::IfExpr(if_expr) => {
                        // check condition
                        test_infix_expression(&if_expr.condition, "x", "<", "y");

                        // test consequence
                        assert_eq!(
                            if_expr.consequence.statements.len(),
                            1,
                            "consequence is not 1 statement"
                        );

                        let conseqence = match if_expr.consequence.statements.first().unwrap() {
                            ExpressionStmt(expression) => expression,
                            other => panic!("Expected consequence statement, got {other:?}"),
                        };

                        test_identifier(conseqence, "x");

                        // test alternative
                        if let Some(alternative) = if_expr.alternative {
                            assert_eq!(
                                alternative.statements.len(),
                                1,
                                "alternative is not 1 statement"
                            );

                            let alt = match alternative.statements.first().unwrap() {
                                ExpressionStmt(expression) => expression,
                                other => panic!("Expected consequence statement, got {other:?}"),
                            };

                            test_identifier(alt, "y");
                        } else {
                            panic!("Expected alternative block statement, got None");
                        }
                    }
                    other => panic!("Expected if expression, got {other:?}"),
                },
                _ => panic!("Expected if statement, got {:?}", stmt),
            };
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
        let tests = vec![
            LetTestData {
                input: "let x = 5;",
                expected_ident: "x",
                expected: Value::Int(5),
            },
            LetTestData {
                input: "let y = 10;",
                expected_ident: "y",
                expected: Value::Int(10),
            },
            LetTestData {
                input: "let foobar = 838388;",
                expected_ident: "foobar",
                expected: Value::Int(838388),
            },
            LetTestData {
                input: "let y = true;",
                expected_ident: "y",
                expected: Value::Boolean(true),
            },
            LetTestData {
                input: "let foobar = y;",
                expected_ident: "foobar",
                expected: Value::Message("y"),
            },
        ];

        for test in tests {
            let mut parser = make_program(test.input);
            let program = parser.parse_program();
            check_for_parse_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            for stmt in program.statements {
                assert!(stmt.token_literal() == "let");

                match stmt {
                    Statement::Let(let_stmt) => {
                        test_let_statement(&let_stmt, test.expected_ident);
                        test_literal_expression(&let_stmt.value, test.expected);
                    }
                    _ => panic!("Expected let statement, got {:?}", stmt),
                };
            }
        }
    }

    fn test_let_statement(stmt: &LetStatement, name: &str) {
        assert_eq!(stmt.name.value, name);
        assert_eq!(stmt.name.token_literal(), name)
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            InputExpected {
                input: "-a * b",
                expected: "((-a) * b)",
            },
            InputExpected {
                input: "!-a",
                expected: "(!(-a))",
            },
            InputExpected {
                input: "a + b + c",
                expected: "((a + b) + c)",
            },
            InputExpected {
                input: "a + b - c",
                expected: "((a + b) - c)",
            },
            InputExpected {
                input: "a * b * c",
                expected: "((a * b) * c)",
            },
            InputExpected {
                input: "a * b / c",
                expected: "((a * b) / c)",
            },
            InputExpected {
                input: "a + b / c",
                expected: "(a + (b / c))",
            },
            InputExpected {
                input: "a + b * c + d / e - f",
                expected: "(((a + (b * c)) + (d / e)) - f)",
            },
            InputExpected {
                input: "3 + 4; -5 * 5",
                expected: "(3 + 4)((-5) * 5)",
            },
            InputExpected {
                input: "5 > 4 == 3 < 4",
                expected: "((5 > 4) == (3 < 4))",
            },
            InputExpected {
                input: "5 > 4 != 3 > 4",
                expected: "((5 > 4) != (3 > 4))",
            },
            InputExpected {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5",
                expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            },
            InputExpected {
                input: "true",
                expected: "true",
            },
            InputExpected {
                input: "false",
                expected: "false",
            },
            InputExpected {
                input: "3 > 5 == false",
                expected: "((3 > 5) == false)",
            },
            InputExpected {
                input: "3 < 5 == true",
                expected: "((3 < 5) == true)",
            },
            InputExpected {
                input: "1 + (2 + 3) + 4",
                expected: "((1 + (2 + 3)) + 4)",
            },
            InputExpected {
                input: "(5 + 5) * 2",
                expected: "((5 + 5) * 2)",
            },
            InputExpected {
                input: "2 / (5 + 5)",
                expected: "(2 / (5 + 5))",
            },
            InputExpected {
                input: "-(5 + 5)",
                expected: "(-(5 + 5))",
            },
            InputExpected {
                input: "!(true == true)",
                expected: "(!(true == true))",
            },
            InputExpected {
                input: "a + add(b * c) + d",
                expected: "((a + add((b * c))) + d)",
            },
            InputExpected {
                input: "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                expected: "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            },
            InputExpected {
                input: "add(a + b + c * d / f + g)",
                expected: "add((((a + b) + ((c * d) / f)) + g))",
            },
            InputExpected {
                input: "a * [1, 2, 3, 4][b * c] * d",
                expected: "((a * ([1, 2, 3, 4][(b * c)])) * d)",
            },
            InputExpected {
                input: "add(a * b[2], b[1], 2 * [1, 2][1])",
                expected: "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
            },
        ];

        for input in tests {
            let mut parser = make_program(input.input);
            let program = parser.parse_program();
            check_for_parse_errors(&parser);

            let actual = program.string();
            assert_eq!(actual, input.expected);
        }
    }

    #[test]
    fn test_parsing_array_literal() {
        let input = "[1, 2 * 2, 3 + 3]";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                Statement::ExpressionStmt(expr) => match expr {
                    Expression::Array(array) => {
                        assert_eq!(array.elements.len(), 3);
                        test_integer_literal(array.elements.first().unwrap(), 1);
                        test_infix_expression(array.elements.get(1).unwrap(), 2, "*", 2);
                        test_infix_expression(array.elements.get(2).unwrap(), 3, "+", 3);
                    }
                    _ => panic!("Expected Expression Statement, got {:?}", expr),
                },
                _ => panic!("Expected Expression Statement, got {:?}", stmt),
            }
        }
    }
    #[test]
    fn test_parsing_empty_hash_literal() {
        let input = "{}";

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                ExpressionStmt(expr) => match expr {
                    Expression::Hash(hash_literal) => {
                        assert_eq!(hash_literal.pairs.len(), 0);
                    }
                    other => panic!("Expression is not HashLiteral, got {other:?}"),
                },
                _ => panic!("Expected Expression Statement, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_parsing_hash_literals_string_keys() {
        let input = r#"{"one": 1, "two": 2, "three": 3}"#;

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        for stmt in program.statements {
            match stmt {
                ExpressionStmt(expr) => match expr {
                    Expression::Hash(hash_literal) => {
                        assert_eq!(hash_literal.pairs.len(), 3);

                        let mut expected = HashMap::new();
                        expected.insert("one".to_string(), 1_i64);
                        expected.insert("two".to_string(), 2_i64);
                        expected.insert("three".to_string(), 3_i64);

                        for (key, value) in hash_literal.pairs {
                            match (key, value.clone()) {
                                (StringExpr(string_expr), Expression::Integer(_)) => {
                                    let expected_value =
                                        expected.get(&string_expr.string()).unwrap();

                                    test_integer_literal(&value, *expected_value);
                                }
                                other => panic!("Expected String, Integer Pair. got {other:?}"),
                            }
                        }
                    }
                    other => panic!("Expression is not HashLiteral, got {other:?}"),
                },
                _ => panic!("Expected Expression Statement, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_parsing_hash_literal_with_expressions() {
        let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        let Statement::ExpressionStmt(expr) = &program.statements[0] else {
            panic!(
                "Expected Expression Statement, got {:?}",
                program.statements[0]
            );
        };

        let Expression::Hash(hash) = expr else {
            panic!("exp is not HashLiteral. got={expr:?}");
        };

        assert_eq!(hash.pairs.len(), 3, "hash.pairs has wrong length");

        let tests: HashMap<&str, (i64, &str, i64)> = HashMap::from([
            ("one", (0, "+", 1)),
            ("two", (10, "-", 8)),
            ("three", (15, "/", 5)),
        ]);

        for (key, value) in &hash.pairs {
            let StringExpr(literal) = key else {
                panic!("key is not StringLiteral. got={key:?}");
            };

            let key = literal.string();
            let Some(&(left, operator, right)) = tests.get(key.as_str()) else {
                panic!("No test function for key {key:?} found");
            };

            test_infix_expression(value, left, operator, right);
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
                            test_literal_expression(&infix.left, input.left_value);
                            assert_eq!(infix.operator, input.operator);
                            test_literal_expression(&infix.right, input.right_value);
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
                            test_literal_expression(&prefix.right, input.value);
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

    #[test]
    fn test_string_literal_expression() {
        let input = r#""hello world""#;

        let mut parser = make_program(input);
        let program = parser.parse_program();
        check_for_parse_errors(&parser);

        if let Some(stmt) = program.statements.first() {
            match stmt {
                ExpressionStmt(expression) => match expression {
                    Expression::StringExpr(string_expression) => {
                        assert_eq!(string_expression.string(), "hello world")
                    }
                    other => panic!("exp is not a StringLiteral. got={other:?}"),
                },
                other => panic!("Expected an expression statement. got={other:?}"),
            }
        }
    }
}
