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
    Boolean(Boolean<'e>),
    Ident(Identifier<'e>),
    IfExpr(IfExpression<'e>),
    InfixExpr(Infix<'e>),
    Integer(IntegerLiteral<'e>),
    PrefixExpr(Prefix<'e>),
}

impl<'e> Node for Expression<'e> {
    fn token_literal(&self) -> &str {
        match self {
            Expression::Boolean(boolean) => boolean.token_literal(),
            Expression::Ident(identifier) => identifier.token_literal(),
            Expression::IfExpr(if_expression) => if_expression.token_literal(),
            Expression::InfixExpr(infix) => infix.token_literal(),
            Expression::Integer(int) => int.token_literal(),
            Expression::PrefixExpr(prefix) => prefix.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Expression::Boolean(boolean) => boolean.string(),
            Expression::Ident(identifier) => identifier.string(),
            Expression::IfExpr(if_expression) => if_expression.string(),
            Expression::InfixExpr(infix) => infix.string(),
            Expression::Integer(int) => int.string(),
            Expression::PrefixExpr(prefix) => prefix.string(),
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
        let mut out = String::new();

        for s in &self.statements {
            out.push_str(&s.string());
        }

        out
    }
}

/// Boolean type
#[derive(Debug)]
pub(crate) struct Boolean<'b> {
    pub(crate) token: Token<'b>,
    pub(crate) value: bool,
}

impl<'b> Node for Boolean<'b> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        match self.value {
            true => "true".to_string(),
            false => "false".to_string(),
        }
    }
}

/// BlockStatement type inf the form of `{<statement_1>; ... <statement_n>;}`
#[derive(Debug)]
pub(crate) struct BlockStatement<'bs> {
    pub(crate) token: Token<'bs>,
    pub(crate) statements: Vec<Statement<'bs>>,
}

impl<'bs> Node for BlockStatement<'bs> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        let mut out = String::new();

        for statement in &self.statements {
            out.push_str(statement.string().as_str());
        }

        out
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

/// A binding for the if/else in the form of `if (<condition>) <consequence> else <alternative>`
#[derive(Debug)]
pub(crate) struct IfExpression<'ie> {
    pub(crate) token: Token<'ie>,
    pub(crate) condition: Box<Expression<'ie>>,
    pub(crate) consequence: BlockStatement<'ie>,
    pub(crate) alternative: Option<BlockStatement<'ie>>,
}

impl<'ie> Node for IfExpression<'ie> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str("if");
        out.push_str(self.condition.string().as_str());
        out.push_str(" ");
        out.push_str(self.consequence.string().as_str());

        if let Some(alt) = &self.alternative {
            out.push_str("else ");
            out.push_str(alt.string().as_str());
        }

        out
    }
}

/// A binding of the form `!5 or -15`
#[derive(Debug)]
pub(crate) struct Infix<'i> {
    pub(crate) token: Token<'i>,
    pub(crate) left: Box<Expression<'i>>,
    pub(crate) operator: &'i str,
    pub(crate) right: Box<Expression<'i>>,
}

impl<'i> Node for Infix<'i> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str("(");
        out.push_str(self.left.string().as_str());
        out.push_str(" ");
        out.push_str(self.operator);
        out.push_str(" ");
        out.push_str(self.right.string().as_str());
        out.push_str(")");

        out
    }
}

/// A binding for Integer literal values in the form of 5 or -15
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
        let mut out = "let ".to_string();
        out.push_str(&self.name.string());
        out.push_str(" = ");

        out.push_str(";");

        out
    }
}

/// A binding of the form `!5 or -15`
#[derive(Debug)]
pub(crate) struct Prefix<'p> {
    pub(crate) token: Token<'p>,
    pub(crate) operator: &'p str,
    pub(crate) right: Box<Expression<'p>>,
}

impl<'p> Node for Prefix<'p> {
    fn token_literal(&self) -> &str {
        self.token.literal
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str("(");
        out.push_str(self.operator);
        out.push_str(self.right.string().as_str());
        out.push_str(")");

        out
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
        let mut out = "return".to_string();

        out.push_str(";");

        out
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
        assert_eq!(
            program.string(),
            "let myVar = ;".to_string(),
            "program mismatch"
        )
    }
}
