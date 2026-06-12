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

    pub fn get_function(&self) -> Option<GcRef<ObjFunction>> {
        if let Value::FUNCTION(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn is_falsey(value: &Value) -> bool {
        match value {
            Value::NUMBER(x) => *x == 0f64,
            Value::BOOL(bool) => !bool,
            Value::STR(_) => false,
            Value::FUNCTION(_) => false,
            Value::NIL => true,
        }
    }

    pub fn falsify(value: &Value) -> bool {
        Value::is_falsey(value)
    }

    pub fn values_equal(v1: &Value, v2: &Value) -> bool {
        if std::mem::discriminant(v1) == std::mem::discriminant(v2) {
            match v1 {
                Value::BOOL(x) => *x == v2.get_bool().unwrap(),
                Value::NUMBER(x) => *x == v2.get_number().unwrap(),
                Value::NIL => true,
                Value::STR(s) => *s == v2.get_string().unwrap(),
                Value::FUNCTION(f) => *f == v2.get_function().unwrap(),
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

#[cfg(test)]
mod tests {
    use crate::{
        gc::Gc,
        object::ObjFunction,
        value::Value,
    };

    #[test]
    fn getters_return_typed_values() {
        let mut gc = Gc::new();
        let s = gc.intern("abc".to_string());

        assert_eq!(Value::NUMBER(3.0).get_number(), Some(3.0));
        assert_eq!(Value::BOOL(true).get_bool(), Some(true));
        assert!(Value::STR(s).get_string() == Some(s));
        assert_eq!(Value::NIL.get_number(), None);
    }

    #[test]
    fn falsey_and_falsify_behavior() {
        let mut gc = Gc::new();
        let s = gc.intern("x".to_string());

        assert!(Value::is_falsey(&Value::NIL));
        assert!(Value::is_falsey(&Value::NUMBER(0.0)));
        assert!(!Value::is_falsey(&Value::NUMBER(1.0)));
        assert!(Value::falsify(&Value::NIL));

        assert!(Value::falsify(&Value::BOOL(false)));
        assert!(!Value::falsify(&Value::BOOL(true)));
        assert!(Value::falsify(&Value::NUMBER(0.0)));
        assert!(!Value::falsify(&Value::STR(s)));
    }

    #[test]
    fn functions_are_truthy() {
        let mut gc = Gc::new();
        let name = gc.intern("f".to_string());
        let function = gc.alloc(ObjFunction::new(name));
        let value = Value::FUNCTION(function);

        assert!(!Value::is_falsey(&value));
        assert!(!Value::falsify(&value));
    }

    #[test]
    fn values_equal_checks_type_and_contents() {
        let mut gc = Gc::new();
        let a1 = gc.intern("a".to_string());
        let a2 = gc.intern("a".to_string());
        let f_name = gc.intern("f".to_string());
        let f1 = gc.alloc(ObjFunction::new(f_name));
        let f2 = gc.alloc(ObjFunction::new(f_name));

        assert!(Value::values_equal(&Value::NUMBER(1.0), &Value::NUMBER(1.0)));
        assert!(Value::values_equal(&Value::BOOL(true), &Value::BOOL(true)));
        assert!(Value::values_equal(&Value::NIL, &Value::NIL));
        assert!(Value::values_equal(&Value::STR(a1), &Value::STR(a2)));
        assert!(Value::values_equal(&Value::FUNCTION(f1), &Value::FUNCTION(f1)));
        assert!(!Value::values_equal(&Value::FUNCTION(f1), &Value::FUNCTION(f2)));
        assert!(!Value::values_equal(&Value::NUMBER(1.0), &Value::BOOL(true)));
    }
}
