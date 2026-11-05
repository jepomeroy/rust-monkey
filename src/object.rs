use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
    rc::Rc,
};

use crate::ast::{BlockStatement, Expression, NodeType};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct HashKey {
    objtype: String,
    value: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct HashPair {
    pub(crate) key: Box<ObjectType>,
    pub(crate) value: Box<ObjectType>,
}

pub(crate) trait Object {
    fn objtype(&self) -> String;
    fn inspect(&self) -> String;
    fn hash_key(&self) -> HashKey;
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

    fn hash_key(&self) -> HashKey {
        HashKey {
            objtype: ObjectType::Error("".to_string()).objtype(),
            value: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ObjectType {
    Array(Vec<ObjectType>),
    Boolean(bool),
    Builtin(BuiltinFunction),
    Error(String),
    Function(FunctionObject),
    Hash(HashMap<HashKey, HashPair>),
    Integer(i64),
    Null,
    Return(Box<ObjectType>),
    StringObj(String),
}

impl Object for ObjectType {
    fn objtype(&self) -> String {
        match self {
            ObjectType::Array(_) => "ARRAY".to_string(),
            ObjectType::Boolean(_) => "BOOLEAN".to_string(),
            ObjectType::Builtin(_) => "BUILTIN".to_string(),
            ObjectType::Error(_) => "ERROR".to_string(),
            ObjectType::Function(f) => f.objtype(),
            ObjectType::Hash(_) => "HASH".to_string(),
            ObjectType::Integer(_) => "INTEGER".to_string(),
            ObjectType::Null => "NULL".to_string(),
            ObjectType::Return(_) => "RETURN".to_string(),
            ObjectType::StringObj(_) => "STRING".to_string(),
        }
    }

    fn inspect(&self) -> String {
        match self {
            ObjectType::Array(a) => {
                let mut out = String::new();

                out.push('[');
                out.push_str(
                    a.iter()
                        .map(|e| e.inspect())
                        .collect::<Vec<String>>()
                        .join(", ")
                        .as_str(),
                );
                out.push(']');

                out
            }
            ObjectType::Boolean(b) => format!("{}", b),
            ObjectType::Builtin(_) => "builtin function".to_string(),
            ObjectType::Error(e) => e.clone(),
            ObjectType::Function(f) => f.inspect(),
            ObjectType::Hash(h) => {
                let mut out = String::new();

                out.push('{');
                out.push_str(
                    h.iter()
                        .map(|e| format!("{}: {}", e.1.key.inspect(), e.1.value.inspect()))
                        .collect::<Vec<String>>()
                        .join(", ")
                        .as_str(),
                );
                out.push('}');

                out
            }
            ObjectType::Integer(i) => format!("{}", i),
            ObjectType::Null => "null".to_string(),
            ObjectType::Return(r) => r.inspect(),
            ObjectType::StringObj(s) => s.to_string(),
        }
    }

    fn hash_key(&self) -> HashKey {
        let mut hasher = DefaultHasher::default();

        match self {
            ObjectType::Boolean(b) => HashKey {
                objtype: ObjectType::Boolean(*b).objtype(),
                value: match b {
                    true => 1,
                    false => 0,
                },
            },
            ObjectType::Integer(i) => {
                hasher.write_i64(*i);

                HashKey {
                    objtype: ObjectType::Integer(*i).objtype(),
                    value: hasher.finish(),
                }
            }

            ObjectType::StringObj(s) => {
                let bytes = s.as_bytes();
                hasher.write(bytes);

                HashKey {
                    objtype: ObjectType::StringObj(s.clone()).objtype(),
                    value: hasher.finish(),
                }
            }
            _ => HashKey {
                objtype: ObjectType::Error("".to_string()).objtype(),
                value: 0,
            },
        }
    }
}

pub(crate) type Env = Rc<RefCell<Environment>>;

#[derive(Debug, PartialEq)]
pub(crate) struct Environment {
    store: HashMap<String, ObjectType>,
    outer: Option<Env>,
}

impl Environment {
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

#[cfg(test)]
mod tests {
    use crate::object::ObjectType::{Integer, StringObj};

    use super::*;

    #[test]
    fn test_string_hash_key() {
        let hello1 = StringObj("Hello World".to_string());
        let hello2 = StringObj("Hello World".to_string());
        let diff1 = StringObj("My name is johnny".to_string());
        let diff2 = StringObj("My name is johnny".to_string());
        let num1 = Integer(290348230);
        let num2 = Integer(290348230);

        assert_eq!(hello1.hash_key(), hello2.hash_key());
        assert_eq!(diff1.hash_key(), diff2.hash_key());
        assert_eq!(num1.hash_key(), num2.hash_key());
        assert_ne!(hello1.hash_key(), diff1.hash_key());
        assert_ne!(hello1.hash_key(), num1.hash_key());
    }
}
