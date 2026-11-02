//! Abstract syntax tree types built by the [`Parser`](crate::parser::Parser).

use crate::token::Token;

pub(crate) enum Node {
    StatementNode(Statement),
    ExpressionNode(Expression),
    BlockStatementNode(BlockStatement),
    ProgramNode(Program),
}

// Common interface for every node in the AST.
pub(crate) trait NodeType {
    /// The literal text of the token this node is associated with.
    /// Used mainly for debugging and testing.
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

/// A statement: a construct that doesn't produce a value (e.g. `let x = 5;`).
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    ExpressionStmt(Expression),
}

impl NodeType for Statement {
    fn token_literal(&self) -> String {
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
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Expression {
    Boolean(Boolean),
    Call(CallExpression),
    Function(FunctionLiteral),
    Ident(Identifier),
    IfExpr(IfExpression),
    InfixExpr(Infix),
    Integer(IntegerLiteral),
    PrefixExpr(Prefix),
}

impl NodeType for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Boolean(boolean) => boolean.token_literal(),
            Expression::Call(call_expression) => call_expression.token_literal(),
            Expression::Function(function_literal) => function_literal.token_literal(),
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
            Expression::Call(call_expression) => call_expression.string(),
            Expression::Function(function_literal) => function_literal.string(),
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
pub(crate) struct Program {
    pub(crate) statements: Vec<Statement>,
}

impl Program {
    pub(crate) fn new() -> Self {
        Self { statements: vec![] }
    }
}

impl NodeType for Program {
    /// Returns the literal of the first statement, or `""` for an empty program.
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            return self.statements[0].token_literal();
        }

        "".to_string()
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
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Boolean {
    pub(crate) token: Token,
    pub(crate) value: bool,
}

impl NodeType for Boolean {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        match self.value {
            true => "true".to_string(),
            false => "false".to_string(),
        }
    }
}

/// BlockStatement type in the form of `{<statement_1>; ... <statement_n>;}`
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct BlockStatement {
    pub(crate) token: Token,
    pub(crate) statements: Vec<Statement>,
}

impl NodeType for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        for statement in &self.statements {
            out.push_str(statement.string().as_str());
        }

        out
    }
}

/// Call Expression in the form of `<expression>(<comma separated expressions>);`
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct CallExpression {
    pub(crate) token: Token,
    pub(crate) function: Box<Expression>,
    pub(crate) arguments: Vec<Expression>,
}

impl NodeType for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str(self.function.string().as_str());
        out.push('(');
        out.push_str(
            &self
                .arguments
                .iter()
                .map(|a| a.string())
                .collect::<Vec<String>>()
                .join(", "),
        );
        out.push(')');

        out
    }
}

/// Function Literal in the form of `fn(<param>) { <blockstatements> }`
/// Function are a first class type so you can do something like
/// 'let myFunc = fn(...) { ... }` or `fn() { return fn(...) { ... } }`
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FunctionLiteral {
    pub(crate) token: Token,
    pub(crate) parameters: Vec<Expression>, // Expression::Ident
    pub(crate) body: BlockStatement,
}

impl NodeType for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str(self.token_literal().as_str());
        out.push('(');
        out.push_str(
            &self
                .parameters
                .iter()
                .map(|p| p.string())
                .collect::<Vec<String>>()
                .join(", "),
        );
        out.push(')');
        out.push_str(self.body.string().as_str());

        out
    }
}

/// Expression types
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Identifier {
    pub(crate) token: Token,
    pub(crate) value: String,
}

impl NodeType for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

/// A binding for the if/else in the form of `if (<condition>) <consequence> else <alternative>`
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IfExpression {
    pub(crate) token: Token,
    pub(crate) condition: Box<Expression>,
    pub(crate) consequence: BlockStatement,
    pub(crate) alternative: Option<BlockStatement>,
}

impl NodeType for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str("if");
        out.push_str(self.condition.string().as_str());
        out.push(' ');
        out.push_str(self.consequence.string().as_str());

        if let Some(alt) = &self.alternative {
            out.push_str("else ");
            out.push_str(alt.string().as_str());
        }

        out
    }
}

/// A binding of the form `!5 or -15`
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Infix {
    pub(crate) token: Token,
    pub(crate) left: Box<Expression>,
    pub(crate) operator: String,
    pub(crate) right: Box<Expression>,
}

impl NodeType for Infix {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push('(');
        out.push_str(self.left.string().as_str());
        out.push(' ');
        out.push_str(self.operator.as_str());
        out.push(' ');
        out.push_str(self.right.string().as_str());
        out.push(')');

        out
    }
}

/// A binding for Integer literal values in the form of 5 or -15
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IntegerLiteral {
    pub(crate) token: Token,
    pub(crate) value: i64,
}

impl NodeType for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

/// A binding of the form `let <name> = <value>;`.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct LetStatement {
    pub(crate) token: Token,
    pub(crate) name: Identifier,
    pub(crate) value: Box<Expression>,
}

impl NodeType for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = "let ".to_string();
        out.push_str(&self.name.string());
        out.push_str(" = ");
        out.push_str(&self.value.string());

        out.push(';');

        out
    }
}

/// A binding of the form `!5 or -15`
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Prefix {
    pub(crate) token: Token,
    pub(crate) operator: String,
    pub(crate) right: Box<Expression>,
}

impl NodeType for Prefix {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push('(');
        out.push_str(self.operator.as_str());
        out.push_str(self.right.string().as_str());
        out.push(')');

        out
    }
}

/// A binding of the form `return <value>;`.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ReturnStatement {
    pub(crate) token: Token,
    pub(crate) value: Box<Expression>,
}

impl NodeType for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = "return ".to_string();

        out.push_str(&self.value.string());
        out.push(';');

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
                    literal: "let".to_string(),
                },
                name: Identifier {
                    token: Token {
                        token: TokenType::IDENT,
                        literal: "myVar".to_string(),
                    },
                    value: "myVar".to_string(),
                },
                value: Box::new(Expression::Integer(IntegerLiteral {
                    token: Token {
                        token: TokenType::INT,
                        literal: "5".to_string(),
                    },
                    value: 5,
                })),
            })],
        };

        assert_eq!(
            program.string(),
            "let myVar = 5;".to_string(),
            "program mismatch"
        )
    }
}
