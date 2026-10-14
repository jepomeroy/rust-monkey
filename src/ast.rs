//! Abstract syntax tree types built by the [`Parser`](crate::parser::Parser).

use crate::token::Token;

/// Common interface for every node in the AST.
pub(crate) trait Node {
    /// The literal text of the token this node is associated with.
    /// Used mainly for debugging and testing.
    fn token_literal(&self) -> &str;
    fn string(&self) -> String;
}

/// A statement: a construct that doesn't produce a value (e.g. `let x = 5;`).
#[derive(Debug)]
pub(crate) enum Statement<'s> {
    Let(LetStatement<'s>),
    Return(ReturnStatement<'s>),
    ExpressionStmt(Expression<'s>),
}

impl<'s> Node for Statement<'s> {
    fn token_literal(&self) -> &str {
        match self {
            Statement::Let(s) => s.token_literal(),
            Statement::Return(s) => s.token_literal(),
            Statement::ExpressionStmt(expression) => expression.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Statement::Let(s) => s.string(),
            Statement::Return(s) => s.string(),
            Statement::ExpressionStmt(expression) => expression.string(),
        }
    }
}

/// An expression: a construct that produces a value (e.g. `5`, `add(1, 2)`).
#[derive(Debug)]
pub(crate) enum Expression<'e> {
    Ident(Identifier<'e>),
    Integer(IntegerLiteral<'e>),
}

impl<'e> Node for Expression<'e> {
    fn token_literal(&self) -> &str {
        match self {
            Expression::Ident(identifier) => identifier.token_literal(),
            Expression::Integer(int) => int.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Expression::Ident(identifier) => identifier.string(),
            Expression::Integer(int) => int.string(),
        }
    }
}

/// The root of every AST: a program is a sequence of statements.
#[derive(Debug)]
pub(crate) struct Program<'p> {
    pub(crate) statements: Vec<Statement<'p>>,
}

impl<'p> Program<'p> {
    pub(crate) fn new() -> Self {
        Self { statements: vec![] }
    }
}

impl<'p> Node for Program<'p> {
    /// Returns the literal of the first statement, or `""` for an empty program.
    fn token_literal(&self) -> &str {
        if !self.statements.is_empty() {
            return self.statements[0].token_literal();
        }

        ""
    }

    fn string(&self) -> String {
        let mut str = String::new();

        for s in &self.statements {
            str.push_str(&s.string());
        }

        str
    }
}

/// Expression types
#[derive(Debug)]
pub(crate) struct Identifier<'i> {
    pub(crate) token: Token<'i>,
    pub(crate) value: &'i str,
}

impl<'i> Node for Identifier<'i> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Debug)]
pub(crate) struct IntegerLiteral<'i> {
    pub(crate) token: Token<'i>,
    pub(crate) value: i64,
}

impl<'i> Node for IntegerLiteral<'i> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

/// A binding of the form `let <name> = <value>;`.
#[derive(Debug)]
pub(crate) struct LetStatement<'l> {
    pub(crate) token: Token<'l>,
    pub(crate) name: Identifier<'l>,
    // TODO: store the bound expression once expression parsing exists.
    // pub(crate) value: Box<dyn Expression>,
}

impl<'l> Node for LetStatement<'l> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        let mut str = "let ".to_string();
        str.push_str(&self.name.string());
        str.push_str(" = ");

        str.push_str(";");

        str
    }
}

/// A binding of the form `return <value>;`.
#[derive(Debug)]
pub(crate) struct ReturnStatement<'r> {
    pub(crate) token: Token<'r>,
    // TODO: store the bound expression once expression parsing exists.
    // pub(crate) value: Expression,
}

impl<'r> Node for ReturnStatement<'r> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        let mut str = "return".to_string();

        str.push_str(";");

        str
    }
}

#[cfg(test)]
mod tests {

    use crate::token::TokenType;

    use super::*;

    #[test]
    fn test_string() {
        let program = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    token: TokenType::LET,
                    literal: "let",
                },
                name: Identifier {
                    token: Token {
                        token: TokenType::IDENT,
                        literal: "myVar",
                    },
                    value: "myVar",
                },
            })],
        };

        // TODO: Add the value for the right side of the statement
        assert_eq!(program.string(), "let myVar = ;".to_string())
    }
}
