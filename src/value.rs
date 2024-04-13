use std::fmt::Display;

use crate::{gc::GcRef, object::{ObjFunction, ObjString}};

#[derive(Clone, PartialEq)]
pub enum Value {
    NUMBER(f64),
    BOOL(bool),
    STR(GcRef<ObjString>),
    FUNCTION(GcRef<ObjFunction>),
    NIL,
}

impl Value {
    pub fn get_bool(&self) -> Option<bool> {
        if let Value::BOOL(x) = self {
            Some(*x)
        } else {
            None
        }
    }

    pub fn get_number(&self) -> Option<f64> {
        if let Value::NUMBER(x) = self {
            Some(*x)
        } else {
            None
        }
    }
    
    pub fn get_string(&self) -> Option<GcRef<ObjString>> {
        if let Value::STR(s) = self {
            Some(*s)
        } else {
            None
        }
    }

    pub fn is_falsey(value: &Value) -> bool {
        match value {
            Value::NUMBER(x) => *x == 0f64,
            Value::BOOL(bool) => !bool,
            Value::STR(_) => false,
            Value::NIL => true,
            _ => true,
        }
    }

    pub fn falsify(value: &Value) -> bool {
        match value {
            Value::BOOL(x) => !*x,
            Value::NUMBER(x) => *x == 0f64,
            _ => false,
        }
    }

    pub fn values_equal(v1: &Value, v2: &Value) -> bool {
        if std::mem::discriminant(v1) == std::mem::discriminant(v2) {
            match v1 {
                Value::BOOL(x) => *x == v2.get_bool().unwrap(),
                Value::NUMBER(x) => *x == v2.get_number().unwrap(),
                Value::NIL => true,
                Value::STR(s) => *s == v2.get_string().unwrap(),
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NUMBER(x) => write!(f, "{}", x),
            Value::BOOL(x) => write!(f, "{}", x),
            Value::STR(s) => write!(f, "{}", **s),
            Value::FUNCTION(x) => write!(f, "{}", **x),
            Value::NIL => write!(f, "nil"),
        }
    }
}
