use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ObjectType {
    Boolean(bool),
    Error(String),
    Integer(i64),
    Null,
    Return(Box<ObjectType>),
}

pub(crate) trait Object {
    fn objtype(&self) -> String;
    fn inspect(&self) -> String;
}

impl Object for ObjectType {
    fn objtype(&self) -> String {
        match self {
            ObjectType::Boolean(_) => "BOOLEAN".to_string(),
            ObjectType::Error(_) => "ERROR".to_string(),
            ObjectType::Integer(_) => "INTEGER".to_string(),
            ObjectType::Null => "NULL".to_string(),
            ObjectType::Return(_) => "RETURN".to_string(),
        }
    }

    fn inspect(&self) -> String {
        match self {
            ObjectType::Boolean(b) => format!("{}", b),
            ObjectType::Error(e) => e.clone(),
            ObjectType::Integer(i) => format!("{}", i),
            ObjectType::Null => "null".to_string(),
            ObjectType::Return(r) => r.inspect(),
        }
    }
}

pub(crate) type Env = Rc<RefCell<Environment>>;

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
