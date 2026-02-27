use std::mem::size_of;

use crate::{chunk::Chunk, gc::{GcObject, GcRef}};

#[derive(Clone)]
pub enum ObjectType {
    STRING,
    FUNCTION,
    CLASS,
}

#[repr(C)]
pub struct ObjFunction {
    header: GcObject,
    pub arity: u8,
    pub chunk: Chunk,
    pub name: GcRef<ObjString>,
}

impl ObjFunction {
    pub fn new(name: GcRef<ObjString>) -> ObjFunction {
        ObjFunction {
            header: GcObject::new(ObjectType::FUNCTION, size_of::<ObjFunction>()),
            arity: 0,
            chunk: Chunk::new(),
            name, 
        }
    }
}

impl core::fmt::Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.s == "" {
            f.write_fmt(format_args!("<fn {}>", *(*self).name))
        } else {
            f.write_fmt(format_args!("<script>"))
        }
    }
}
#[repr(C)]
pub struct ObjString {
    header: GcObject,
    pub s: String,
    pub hash: usize,
}

impl ObjString {
    pub fn from_string(s: String) -> ObjString {
        let hash = ObjString::compute_hash(&s);
        ObjString {
            header: GcObject::new(ObjectType::STRING, size_of::<ObjString>()),
            s,
            hash,
        }

    }
    fn compute_hash(s: &str) -> usize {
        let mut hash: usize = 2166136261;
        for c in s.as_bytes() {
            hash ^= *c as usize;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}

impl core::fmt::Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.s)
    }
}
