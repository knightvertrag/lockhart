#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    STRING(String),
    NUMBER(f64)
}