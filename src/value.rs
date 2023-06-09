#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    NUMBER(f64),
    BOOL(bool),
    STR(String),
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
    pub fn get_string(&self) -> Option<&str> {
        if let Value::STR(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn falsify(value: &Value) -> bool {
        match value {
            Value::BOOL(x) => !*x,
            Value::NUMBER(x) => {
                if *x == 0f64 {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn values_equal(v1: &Value, v2: &Value) -> bool {
        if std::mem::discriminant(v1) == std::mem::discriminant(v2) {
            match v1 {
                Value::BOOL(x) => *x == v2.get_bool().unwrap(),
                Value::NUMBER(x) => *x == v2.get_number().unwrap(),
                Value::NIL => true,
                Value::STR(s) => s == v2.get_string().unwrap(),
                _ => false,
            }
        } else {
            false
        }
    }
}
