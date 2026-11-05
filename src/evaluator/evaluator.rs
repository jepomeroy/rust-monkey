use std::collections::HashMap;

use crate::{
    ast::{
        BlockStatement, Expression, HashLiteral, IfExpression, Node, NodeType, Program, Statement,
    },
    evaluator::builtins::get_builtin,
    object::{
        Env, Environment, FunctionObject, HashPair, Object,
        ObjectType::{
            self, Array, Boolean, Builtin, Error, Function, Hash, Integer, Null, Return, StringObj,
        },
    },
};

const TRUE: ObjectType = ObjectType::Boolean(true);
const FALSE: ObjectType = ObjectType::Boolean(false);

pub(crate) fn eval(node: &Node, env: &Env) -> ObjectType {
    match node {
        Node::StatementNode(statement) => eval_statement(statement, env),
        Node::ExpressionNode(expression) => eval_expression(expression, env),
        Node::BlockStatementNode(block_statement) => eval_block_statement(block_statement, env),
        Node::ProgramNode(program) => eval_program(program, env),
    }
}

fn apply_function(function: FunctionObject, args: &[ObjectType]) -> ObjectType {
    let env = extend_function_env(&function, args);
    let evaluated = eval_block_statement(&function.body, &env);

    unwrap_return_value(evaluated)
}

fn extend_function_env(func: &FunctionObject, args: &[ObjectType]) -> Env {
    let env = Environment::new_enclosed(func.env.clone());

    for (i, param) in func.parameters.iter().enumerate() {
        match param {
            Expression::Ident(identifier) => {
                env.borrow_mut()
                    .set(&identifier.value, args.get(i).unwrap().clone());
            }
            _ => (),
        }
    }

    env
}

fn unwrap_return_value(obj: ObjectType) -> ObjectType {
    match obj {
        Return(object_type) => *object_type,
        _ => obj,
    }
}

fn eval_array_expression(arr: &[ObjectType], idx: usize) -> ObjectType {
    if idx > arr.len() - 1 {
        return Null;
    }

    arr.get(idx).unwrap().clone()
}

fn eval_bang_operator_expression(right: ObjectType) -> ObjectType {
    match right {
        ObjectType::Boolean(b) => match b {
            true => FALSE,
            false => TRUE,
        },
        ObjectType::Null => TRUE,
        _ => FALSE,
    }
}

fn eval_block_statement(block: &BlockStatement, env: &Env) -> ObjectType {
    let mut result = Null;

    for s in &block.statements {
        result = eval_statement(s, env);

        if let Error(_) = result {
            return result;
        }

        if let Return(_) = result {
            return result;
        }
    }

    result
}

fn eval_expression(expr: &Expression, env: &Env) -> ObjectType {
    match expr {
        Expression::Array(array) => {
            let elements = eval_expressions(&array.elements, env);

            Array(elements)
        }
        Expression::Boolean(boolean) => get_boolean_object(boolean.value),
        Expression::Call(call_expression) => {
            let function = eval_expression(&call_expression.function, env);

            if is_error(&function) {
                return function;
            }

            let args = eval_expressions(&call_expression.arguments, env);

            if args.len() == 1
                && let Some(arg) = args.first()
                && is_error(arg)
            {
                return arg.clone();
            }

            match function {
                Function(function_obj) => apply_function(function_obj, &args),
                Builtin(func) => func(&args),
                other => new_error(format!("not a function: got={}", other.objtype())),
            }
        }
        Expression::Function(function_literal) => {
            let parameters = function_literal.parameters.clone();
            let body = function_literal.body.clone();

            Function(FunctionObject {
                parameters,
                body,
                env: env.clone(),
            })
        }
        Expression::Hash(hash) => eval_hash_literal(hash, env),
        Expression::Ident(identifier) => {
            if let Some(value) = env.borrow().get(&identifier.value) {
                return value;
            }

            if let Some(builtin) = get_builtin(&identifier.value) {
                return ObjectType::Builtin(builtin);
            }

            new_error(format!("identifier not found: {}", identifier.value))
        }
        Expression::Index(index) => {
            let left = eval_expression(&*index.left, env);
            if is_error(&left) {
                return left;
            }

            let index = eval_expression(&*index.index, env);
            if is_error(&index) {
                return index;
            }

            eval_index_expression(left, index)
        }
        Expression::IfExpr(if_expression) => eval_if_expression(if_expression, env),
        Expression::InfixExpr(infix) => {
            let left = eval_expression(&infix.left, env);

            if is_error(&left) {
                return left;
            }

            let right = eval_expression(&infix.right, env);

            if is_error(&right) {
                return right;
            }

            eval_infix_operator(&infix.operator, left, right)
        }
        Expression::Integer(integer_literal) => ObjectType::Integer(integer_literal.value),
        Expression::PrefixExpr(prefix) => {
            let right = eval_expression(&prefix.right, env);

            if is_error(&right) {
                return right;
            }

            eval_prefix_expression(&prefix.operator, right)
        }
        Expression::StringExpr(string_literal) => ObjectType::StringObj(string_literal.string()),
    }
}

fn eval_expressions(exps: &[Expression], env: &Env) -> Vec<ObjectType> {
    let mut result = vec![];

    for exp in exps {
        let evaluated = eval_expression(exp, env);

        if is_error(&evaluated) {
            return vec![evaluated];
        }

        result.push(evaluated);
    }

    result
}

fn eval_hash_index_expression(hash: &ObjectType, index: &ObjectType) -> ObjectType {
    match hash {
        Hash(hash_map) => {
            if let Some(val) = hash_map.get(&index.hash_key()) {
                return *val.value.clone();
            }

            Null
        }
        other => new_error(format!("Expected HASH object. got {other:?}")),
    }
}

fn eval_hash_literal(hash: &HashLiteral, env: &Env) -> ObjectType {
    let mut pairs_hash_map = HashMap::new();

    for (key_expr, value_expr) in &hash.pairs {
        let key = eval_expression(key_expr, env);
        if is_error(&key) {
            return key;
        };

        let value = eval_expression(value_expr, env);
        if is_error(&value) {
            return value;
        };

        let hash_key = key.hash_key();
        pairs_hash_map.insert(
            hash_key,
            HashPair {
                key: Box::new(key),
                value: Box::new(value),
            },
        );
    }

    Hash(pairs_hash_map)
}

fn eval_if_expression(expr: &IfExpression, env: &Env) -> ObjectType {
    let condition = eval_expression(&expr.condition, env);

    if is_error(&condition) {
        return condition;
    }

    if is_truthy(condition) {
        eval_statements(&expr.consequence.statements, env)
    } else if let Some(alternative) = &expr.alternative {
        eval_statements(&alternative.statements, env)
    } else {
        Null
    }
}

fn eval_index_expression(left: ObjectType, index: ObjectType) -> ObjectType {
    match (&left, &index) {
        (Array(arr), Integer(index)) => {
            if *index < 0 {
                return Null;
            }

            eval_array_expression(&arr, *index as usize)
        }
        (Hash(_), _) => eval_hash_index_expression(&left, &index),
        (other, _) => new_error(format!("index operator not supported: {}", other.objtype())),
    }
}

fn eval_infix_operator(operator: &str, left: ObjectType, right: ObjectType) -> ObjectType {
    match (&left, &right) {
        (Integer(left_val), Integer(right_val)) => {
            if let Some(ot) = eval_integer_infix_operator(operator, *left_val, *right_val) {
                return ot;
            }

            new_error(format!(
                "unknown operator: {} {} {}",
                left.objtype(),
                operator,
                right.objtype()
            ))
        }
        (StringObj(left_val), StringObj(right_val)) => {
            if let Some(st) = eval_string_infix_operator(operator, &*left_val, &*right_val) {
                return st;
            }

            new_error(format!(
                "unknown operator: {} {} {}",
                left.objtype(),
                operator,
                right.objtype()
            ))
        }
        (Boolean(left_val), Boolean(right_val)) => match operator {
            "==" => get_boolean_object(left_val == right_val),
            "!=" => get_boolean_object(left_val != right_val),
            _ => new_error(format!(
                "unknown operator: {} {} {}",
                left.objtype(),
                operator,
                right.objtype()
            )),
        },
        _ => new_error(format!(
            "type mismatch: {} {} {}",
            left.objtype(),
            operator,
            right.objtype()
        )),
    }
}

fn eval_integer_infix_operator(
    operator: &str,
    left_val: i64,
    right_val: i64,
) -> Option<ObjectType> {
    match operator {
        "+" => Some(Integer(left_val + right_val)),
        "-" => Some(Integer(left_val - right_val)),
        "*" => Some(Integer(left_val * right_val)),
        "/" => Some(Integer(left_val / right_val)),
        "<" => Some(get_boolean_object(left_val < right_val)),
        ">" => Some(get_boolean_object(left_val > right_val)),
        "==" => Some(get_boolean_object(left_val == right_val)),
        "!=" => Some(get_boolean_object(left_val != right_val)),
        _ => None,
    }
}

fn eval_minus_prefix_operator_expression(right: ObjectType) -> ObjectType {
    match right {
        ObjectType::Integer(i) => ObjectType::Integer(-i),
        _ => new_error(format!("unknown operator: -{}", right.objtype())),
    }
}

fn eval_prefix_expression(prefix: &str, right: ObjectType) -> ObjectType {
    match prefix {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => new_error(format!("unknown operator: {}{}", prefix, right.objtype())),
    }
}

fn eval_program(program: &Program, env: &Env) -> ObjectType {
    let mut result = Null;

    for s in &program.statements {
        result = eval_statement(s, env);

        match result {
            Return(object_type) => return *object_type,
            Error(_) => return result,
            _ => (),
        }
    }

    result
}

fn eval_statement(stmt: &Statement, env: &Env) -> ObjectType {
    match stmt {
        Statement::Let(let_statement) => {
            let value = eval_expression(&*let_statement.value, env);

            if is_error(&value) {
                return value;
            }

            env.borrow_mut().set(&let_statement.name.value, value)
        }
        Statement::Return(return_statement) => {
            let value = eval_expression(&return_statement.value, env);

            if is_error(&value) {
                return value;
            }

            Return(Box::new(value))
        }
        Statement::ExpressionStmt(expression) => eval_expression(expression, env),
    }
}

fn eval_statements(statements: &[Statement], env: &Env) -> ObjectType {
    let mut result = Null;

    for s in statements {
        result = eval_statement(s, env);

        if let Error(_) = result {
            return result;
        }

        if let Return(_) = result {
            return result;
        }
    }

    result
}

fn eval_string_infix_operator(operator: &str, left: &str, right: &str) -> Option<ObjectType> {
    match operator {
        "+" => Some(StringObj(format!("{}{}", left, right))),
        _ => None,
    }
}

fn get_boolean_object(input: bool) -> ObjectType {
    if input { TRUE } else { FALSE }
}

fn is_error(obj: &ObjectType) -> bool {
    match obj {
        Error(_) => true,
        _ => false,
    }
}

fn is_truthy(obj: ObjectType) -> bool {
    match obj {
        Boolean(b) => b,
        Null => false,
        _ => true,
    }
}

pub(crate) fn new_error(err: String) -> ObjectType {
    Error(err)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::NodeType,
        lexer::Lexer,
        object::{
            Environment, HashKey,
            ObjectType::{self, StringObj},
        },
        parser::Parser,
        test_utils::{InputExpected, Value},
    };

    use super::*;

    fn test_eval(input: &str) -> ObjectType {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        let env = Environment::new();

        eval(&Node::ProgramNode(program), &env)
    }

    fn test_boolean_object(obj: ObjectType, expected: bool) {
        match obj {
            ObjectType::Boolean(b) => assert_eq!(b, expected),
            other => panic!("object is not an Boolean. got={other:?}"),
        }
    }

    fn test_integer_object(obj: ObjectType, expected: i64) {
        match obj {
            ObjectType::Integer(i) => assert_eq!(i, expected),
            other => panic!("object is not an Integer. got={other:?}"),
        }
    }

    fn test_null_object(obj: ObjectType) {
        assert_eq!(obj, Null)
    }

    #[test]
    fn test_array_index_expressions() {
        let tests = vec![
            InputExpected {
                input: "[1, 2, 3][0]",
                expected: Some(1),
            },
            InputExpected {
                input: "[1, 2, 3][1]",
                expected: Some(2),
            },
            InputExpected {
                input: "[1, 2, 3][2]",
                expected: Some(3),
            },
            InputExpected {
                input: "let i = 0; [1][i];",
                expected: Some(1),
            },
            InputExpected {
                input: "[1, 2, 3][1 + 1];",
                expected: Some(3),
            },
            InputExpected {
                input: "let myArray = [1, 2, 3]; myArray[2];",
                expected: Some(3),
            },
            InputExpected {
                input: "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
                expected: Some(6),
            },
            InputExpected {
                input: "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
                expected: Some(2),
            },
            InputExpected {
                input: "[1, 2, 3][3]",
                expected: None,
            },
            InputExpected {
                input: "[1, 2, 3][-1]",
                expected: None,
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);

            match test.expected {
                Some(i) => test_integer_object(evaluated, i),
                None => test_null_object(evaluated),
            }
        }
    }

    #[test]
    fn test_array_literal() {
        let input = "[1, 2 * 2, 3 + 3]";

        let evaluated = test_eval(input);

        match evaluated {
            ObjectType::Array(array) => {
                assert_eq!(array.len(), 3);

                test_integer_object(array.first().unwrap().clone(), 1);
                test_integer_object(array.get(1).unwrap().clone(), 4);
                test_integer_object(array.get(2).unwrap().clone(), 6);
            }
            other => panic!("object is not an Array. got={other:?}"),
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            InputExpected {
                input: "!true",
                expected: false,
            },
            InputExpected {
                input: "!false",
                expected: true,
            },
            InputExpected {
                input: "!5",
                expected: false,
            },
            InputExpected {
                input: "!!true",
                expected: true,
            },
            InputExpected {
                input: "!!false",
                expected: false,
            },
            InputExpected {
                input: "!!5",
                expected: true,
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);

            test_boolean_object(evaluated, test.expected);
        }
    }

    #[test]
    fn test_builtin_function() {
        let tests = vec![
            InputExpected::<Value> {
                input: r#"len("")"#,
                expected: 0.into(),
            },
            InputExpected {
                input: r#"len("four")"#,
                expected: 4.into(),
            },
            InputExpected {
                input: r#"len("hello world")"#,
                expected: 11.into(),
            },
            InputExpected {
                input: r#"len(1)"#,
                expected: "argument to `len` not supported. got INTEGER".into(),
            },
            InputExpected {
                input: r#"len("one", "two")"#,
                expected: "wrong number of arguments. got=2 want=1".into(),
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);

            match test.expected {
                Value::Int(i) => test_integer_object(evaluated, i),
                Value::Message(s) => match evaluated {
                    Error(m) => assert_eq!(m, s),
                    other => panic!("Expect an error message. got={other:?}"),
                },
                _ => (),
            }
        }
    }

    #[test]
    fn test_closures() {
        let input = r"
            let newAdder = fn(x) {
                fn(y) { x + y };
            };

            let addTwo = newAdder(2);
            addTwo(2);";

        test_integer_object(test_eval(input), 4);
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            InputExpected {
                input: "5 + true;",
                expected: "type mismatch: INTEGER + BOOLEAN",
            },
            InputExpected {
                input: "5 + true; 5;",
                expected: "type mismatch: INTEGER + BOOLEAN",
            },
            InputExpected {
                input: "-true",
                expected: "unknown operator: -BOOLEAN",
            },
            InputExpected {
                input: "true + false;",
                expected: "unknown operator: BOOLEAN + BOOLEAN",
            },
            InputExpected {
                input: "5; true + false; 5",
                expected: "unknown operator: BOOLEAN + BOOLEAN",
            },
            InputExpected {
                input: "if (10 > 1) { true + false; }",
                expected: "unknown operator: BOOLEAN + BOOLEAN",
            },
            InputExpected {
                input: r#""Hello" - "World""#,
                expected: "unknown operator: STRING - STRING",
            },
            InputExpected {
                input: r"
                if (10 > 1) {
                  if (10 > 1) {
                    return true + false;
                  }

                  return 1;
                }",
                expected: "unknown operator: BOOLEAN + BOOLEAN",
            },
            InputExpected {
                input: "foobar",
                expected: "identifier not found: foobar",
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);
            match evaluated {
                ObjectType::Error(e) => assert_eq!(e.as_str(), test.expected),
                other => panic!(
                    "no error object returned. got={other:?}, {} {}",
                    test.input, test.expected
                ),
            }
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            InputExpected {
                input: "true",
                expected: true,
            },
            InputExpected {
                input: "false",
                expected: false,
            },
            InputExpected {
                input: "1 < 2",
                expected: true,
            },
            InputExpected {
                input: "1 > 2",
                expected: false,
            },
            InputExpected {
                input: "1 < 1",
                expected: false,
            },
            InputExpected {
                input: "1 > 1",
                expected: false,
            },
            InputExpected {
                input: "1 == 1",
                expected: true,
            },
            InputExpected {
                input: "1 != 1",
                expected: false,
            },
            InputExpected {
                input: "1 == 2",
                expected: false,
            },
            InputExpected {
                input: "1 != 2",
                expected: true,
            },
            InputExpected {
                input: "true == true",
                expected: true,
            },
            InputExpected {
                input: "false == false",
                expected: true,
            },
            InputExpected {
                input: "true == false",
                expected: false,
            },
            InputExpected {
                input: "true != false",
                expected: true,
            },
            InputExpected {
                input: "false != true",
                expected: true,
            },
            InputExpected {
                input: "(1 < 2) == true",
                expected: true,
            },
            InputExpected {
                input: "(1 < 2) == false",
                expected: false,
            },
            InputExpected {
                input: "(1 > 2) == true",
                expected: false,
            },
            InputExpected {
                input: "(1 > 2) == false",
                expected: true,
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);

            test_boolean_object(evaluated, test.expected);
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            InputExpected {
                input: "let identity = fn(x) { x; }; identity(5);",
                expected: 5,
            },
            InputExpected {
                input: "let identity = fn(x) { return x; }; identity(5);",
                expected: 5,
            },
            InputExpected {
                input: "let double = fn(x) { x * 2; }; double(5);",
                expected: 10,
            },
            InputExpected {
                input: "let add = fn(x, y) { x + y; }; add(5, 5);",
                expected: 10,
            },
            InputExpected {
                input: "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                expected: 20,
            },
            InputExpected {
                input: "fn(x) { x; }(5)",
                expected: 5,
            },
        ];

        for test in tests {
            test_integer_object(test_eval(test.input), test.expected)
        }
    }

    #[test]
    fn test_function_object() {
        let input = "fn(x) { x + 2; };";

        let evaluated = test_eval(input);

        match evaluated {
            ObjectType::Function(func) => {
                assert_eq!(func.parameters.len(), 1);
                assert_eq!(func.parameters.get(0).unwrap().string(), "x");
                assert_eq!(func.body.string(), "(x + 2)");
            }
            other => panic!("object is not a Function. got={other:?}"),
        }
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            InputExpected {
                input: "5",
                expected: 5,
            },
            InputExpected {
                input: "10",
                expected: 10,
            },
            InputExpected {
                input: "-10",
                expected: -10,
            },
            InputExpected {
                input: "-10",
                expected: -10,
            },
            InputExpected {
                input: "5 + 5 + 5 + 5 - 10",
                expected: 10,
            },
            InputExpected {
                input: "2 * 2 * 2 * 2 * 2",
                expected: 32,
            },
            InputExpected {
                input: "-50 + 100 + -50",
                expected: 0,
            },
            InputExpected {
                input: "5 * 2 + 10",
                expected: 20,
            },
            InputExpected {
                input: "5 + 2 * 10",
                expected: 25,
            },
            InputExpected {
                input: "20 + 2 * -10",
                expected: 0,
            },
            InputExpected {
                input: "50 / 2 * 2 + 10",
                expected: 60,
            },
            InputExpected {
                input: "2 * (5 + 10)",
                expected: 30,
            },
            InputExpected {
                input: "3 * 3 * 3 + 10",
                expected: 37,
            },
            InputExpected {
                input: "3 * (3 * 3) + 10",
                expected: 37,
            },
            InputExpected {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
                expected: 50,
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);

            test_integer_object(evaluated, test.expected);
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            InputExpected {
                input: "let a = 5; a;",
                expected: 5,
            },
            InputExpected {
                input: "let a = 5 * 5; a;",
                expected: 25,
            },
            InputExpected {
                input: "let a = 5; let b = a; b;",
                expected: 5,
            },
            InputExpected {
                input: "let a = 5; let b = a; let c = a + b + 5; c;",
                expected: 15,
            },
        ];

        for test in tests {
            test_integer_object(test_eval(test.input), test.expected)
        }
    }

    #[test]
    fn test_hash_index_expression() {
        let tests = vec![
            InputExpected {
                input: r#"{"foo": 5}["foo"]"#,
                expected: Some(5),
            },
            InputExpected {
                input: r#"{"foo": 5}["bar"]"#,
                expected: None,
            },
            InputExpected {
                input: r#"let key = "foo"; {"foo": 5}[key]"#,
                expected: Some(5),
            },
            InputExpected {
                input: r#"{}["foo"]"#,
                expected: None,
            },
            InputExpected {
                input: r#"{5: 5}[5]"#,
                expected: Some(5),
            },
            InputExpected {
                input: r#"   `{true: 5}[true]"#,
                expected: Some(5),
            },
            InputExpected {
                input: r#"{false: 5}[false]"#,
                expected: Some(5),
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);

            match test.expected {
                Some(i) => test_integer_object(evaluated, i),
                None => test_null_object(evaluated),
            }
        }
    }

    #[test]
    fn test_hash_literals() {
        let input = r#"
            let two = "two";
            {
                "one": 10 - 9,
                two: 1 + 1,
                "thr" + "ee": 6 / 2,
                4: 4,
                true: 5,
                false: 6
            }"#;

        let evaluated = test_eval(input);

        let pairs = match evaluated {
            ObjectType::Hash(hash_map) => hash_map,
            other => panic!("Eval did not return a Hash. got {other:?}"),
        };

        let expected: HashMap<HashKey, i64> = HashMap::from([
            (StringObj("one".to_string()).hash_key(), 1),
            (StringObj("two".to_string()).hash_key(), 2),
            (StringObj("three".to_string()).hash_key(), 3),
            (Integer(4).hash_key(), 4),
            (TRUE.hash_key(), 5),
            (FALSE.hash_key(), 6),
        ]);

        assert_eq!(
            pairs.len(),
            expected.len(),
            "Hash has wrong num of pairs. got={}",
            pairs.len()
        );

        for (expected_key, expected_value) in expected {
            let pair = pairs
                .get(&expected_key)
                .unwrap_or_else(|| panic!("no pair for given key {expected_key:?} in Pairs"));

            test_integer_object(*pair.value.clone(), expected_value);
        }
    }

    #[test]
    fn test_if_else_expression() {
        let tests = vec![
            InputExpected {
                input: "if (true) { 10 }",
                expected: 10,
            },
            InputExpected {
                input: "if (false) { 10 }",
                expected: 0,
            },
            InputExpected {
                input: "if (1) { 10 }",
                expected: 10,
            },
            InputExpected {
                input: "if (1 < 2) { 10 }",
                expected: 10,
            },
            InputExpected {
                input: "if (1 > 2) { 10 }",
                expected: 0,
            },
            InputExpected {
                input: "if (1 > 2) { 10 } else { 20 }",
                expected: 20,
            },
            InputExpected {
                input: "if (1 < 2) { 10 } else { 20 }",
                expected: 10,
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);
            match evaluated {
                Integer(_) => test_integer_object(evaluated, test.expected),
                Null => test_null_object(evaluated),
                other => panic!("expected Integer or Null, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_return_statement() {
        let tests = vec![
            InputExpected {
                input: "return 10;",
                expected: 10,
            },
            InputExpected {
                input: "return 10; 9;",
                expected: 10,
            },
            InputExpected {
                input: "return 2 * 5; 9;",
                expected: 10,
            },
            InputExpected {
                input: "9; return 2 * 5; 9;",
                expected: 10,
            },
            InputExpected {
                input: r"if (10 > 1) {
                  if (10 > 1) {
                    return 10;
                  }

                  return 1;
                }",
                expected: 10,
            },
        ];

        for test in tests {
            let evaluated = test_eval(test.input);
            match evaluated {
                Integer(_) => test_integer_object(evaluated, test.expected),
                other => panic!("expected Integer,  got {other:?}"),
            }
        }
    }

    #[test]
    fn test_string_literal() {
        let input = r#""Hello World!""#;

        let evaluated = test_eval(input);

        match evaluated {
            StringObj(value) => assert_eq!(value, "Hello World!"),
            other => panic!("object is not String. got={other:?}"),
        }
    }

    #[test]
    fn test_string_concatenation() {
        let input = r#""Hello" + " " + "World!""#;

        let evaluated = test_eval(input);

        match evaluated {
            StringObj(value) => assert_eq!(value, "Hello World!"),
            other => panic!("object is not String. got={other:?}"),
        }
    }
}
