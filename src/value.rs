#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    STRING(String),
    NUMBER(f64),
    BOOL(bool),
    NIL
}