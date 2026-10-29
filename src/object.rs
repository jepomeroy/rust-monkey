#[derive(Debug, PartialEq)]
pub(crate) enum ObjectType {
    Integer(i64),
    Boolean(bool),
    Return(Box<ObjectType>),
    Null,
}

pub(crate) trait Object {
    fn objtype(&self) -> String;
    fn inspect(&self) -> String;
}

impl TryInto<i64> for ObjectType {
    type Error = String;

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            ObjectType::Integer(i) => Ok(i),
            _ => Err("Not an Integer".to_string()),
        }
    }
}

impl Object for ObjectType {
    fn objtype(&self) -> String {
        match self {
            ObjectType::Integer(_) => "INTEGER".to_string(),
            ObjectType::Boolean(_) => "BOOLEAN".to_string(),
            ObjectType::Null => "NULL".to_string(),
            ObjectType::Return(object_type) => "RETURN".to_string(),
        }
    }

    fn inspect(&self) -> String {
        match self {
            ObjectType::Integer(i) => format!("{}", i),
            ObjectType::Boolean(b) => format!("{}", b),
            ObjectType::Null => "null".to_string(),
            ObjectType::Return(r) => r.inspect(),
        }
    }
}
