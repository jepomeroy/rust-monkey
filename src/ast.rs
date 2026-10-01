//! Abstract syntax tree types built by the [`Parser`](crate::parser::Parser).

use crate::token::Token;

/// Common interface for every node in the AST.
pub(crate) trait Node {
    /// The literal text of the token this node is associated with.
    /// Used mainly for debugging and testing.
    fn token_literal(&self) -> String;
}

/// A statement: a construct that doesn't produce a value (e.g. `let x = 5;`).
#[derive(Debug)]
pub(crate) enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::Let(s) => s.token_literal(),
            Statement::Return(s) => s.token_literal(),
        }
    }
}

/// An expression: a construct that produces a value (e.g. `5`, `add(1, 2)`).
#[derive(Debug)]
pub(crate) enum Expression {
    Ident(Identifier),
}

/// The root of every AST: a program is a sequence of statements.
pub(crate) struct Program {
    pub(crate) statements: Vec<Statement>,
}

impl Program {
    pub(crate) fn new() -> Self {
        Self { statements: vec![] }
    }
}

impl Node for Program {
    /// Returns the literal of the first statement, or `""` for an empty program.
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            return self.statements[0].token_literal();
        }

        "".to_string()
    }
}

/// Expression types
#[derive(Debug)]
pub(crate) struct Identifier {
    pub(crate) token: Token,
    pub(crate) value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

/// A binding of the form `let <name> = <value>;`.
#[derive(Debug)]
pub(crate) struct LetStatement {
    pub(crate) token: Token,
    pub(crate) name: Identifier,
    // TODO: store the bound expression once expression parsing exists.
    // pub(crate) value: Box<dyn Expression>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

/// A binding of the form `return <value>;`.
#[derive(Debug)]
pub(crate) struct ReturnStatement {
    pub(crate) token: Token,
    // TODO: store the bound expression once expression parsing exists.
    // pub(crate) value: Expression,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}
