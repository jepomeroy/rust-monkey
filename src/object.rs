#[derive(Debug, PartialEq)]
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
            ObjectType::Return(object_type) => "RETURN".to_string(),
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
