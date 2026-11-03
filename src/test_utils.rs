//! Shared helpers for unit tests. Only compiled under `cargo test`.

pub(crate) struct InputExpected<'ie, T> {
    pub(crate) input: &'ie str,
    pub(crate) expected: T,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Value<'a> {
    Int(i64),
    Message(&'a str),
    Boolean(bool),
}

// These let callers pass plain values: test_infix_expression(&e, 5, "+", "x")
impl From<i64> for Value<'_> {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<i32> for Value<'_> {
    fn from(v: i32) -> Self {
        Value::Int(v as i64)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(v: &'a str) -> Self {
        Value::Message(v)
    }
}

impl From<bool> for Value<'_> {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}
