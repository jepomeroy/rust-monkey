use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::{BlockStatement, Expression, NodeType};

pub(crate) trait Object {
    fn objtype(&self) -> String;
    fn inspect(&self) -> String;
}

pub(crate) type BuiltinFunction = fn(args: &[ObjectType]) -> ObjectType;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FunctionObject {
    pub(crate) parameters: Vec<Expression>,
    pub(crate) body: BlockStatement,
    pub(crate) env: Env,
}

impl Object for FunctionObject {
    fn objtype(&self) -> String {
        "FUNCTION".to_string()
    }

    fn inspect(&self) -> String {
        let mut out = String::new();

        out.push_str("fn(");
        out.push_str(
            &self
                .parameters
                .iter()
                .map(|p| p.string())
                .collect::<Vec<String>>()
                .join(", "),
        );
        out.push_str(") {\n");
        out.push_str(self.body.string().as_str());
        out.push_str("\n}");

        out
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ObjectType {
    Boolean(bool),
    Builtin(BuiltinFunction),
    Error(String),
    Function(FunctionObject),
    Integer(i64),
    Null,
    Return(Box<ObjectType>),
    StringObj(String),
}

impl Object for ObjectType {
    fn objtype(&self) -> String {
        match self {
            ObjectType::Boolean(_) => "BOOLEAN".to_string(),
            ObjectType::Builtin(_) => "BUILTIN".to_string(),
            ObjectType::Error(_) => "ERROR".to_string(),
            ObjectType::Function(f) => f.objtype(),
            ObjectType::Integer(_) => "INTEGER".to_string(),
            ObjectType::Null => "NULL".to_string(),
            ObjectType::Return(_) => "RETURN".to_string(),
            ObjectType::StringObj(_) => "STRING".to_string(),
        }
    }

    fn inspect(&self) -> String {
        match self {
            ObjectType::Boolean(b) => format!("{}", b),
            ObjectType::Builtin(_) => "builtin function".to_string(),
            ObjectType::Error(e) => e.clone(),
            ObjectType::Function(f) => f.inspect(),
            ObjectType::Integer(i) => format!("{}", i),
            ObjectType::Null => "null".to_string(),
            ObjectType::Return(r) => r.inspect(),
            ObjectType::StringObj(s) => s.to_string(),
        }
    }
}

pub(crate) type Env = Rc<RefCell<Environment>>;

#[derive(Debug, PartialEq)]
pub(crate) struct Environment {
    store: HashMap<String, ObjectType>,
    outer: Option<Env>,
}

impl<'e> Environment {
    pub(crate) fn new() -> Env {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            outer: None,
        }))
    }

    pub(crate) fn new_enclosed(outer: Env) -> Env {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            outer: Some(outer),
        }))
    }

    pub(crate) fn get(&self, name: &str) -> Option<ObjectType> {
        self.store
            .get(name)
            .cloned()
            .or_else(|| self.outer.as_ref()?.borrow().get(name))
    }

    pub(crate) fn set(&mut self, name: &str, obj: ObjectType) -> ObjectType {
        self.store.insert(name.to_string(), obj.clone());
        obj
    }
}
