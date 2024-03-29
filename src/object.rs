use crate::gc::GcObject;

pub enum ObjectType {
    STRING,
    FUNCTION,
    CLASS,
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
            header: GcObject::new(ObjectType::STRING),
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