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
    fn compute_hash(&self) -> usize {
        let mut hash: usize = 2166136261;
        for c in self.s.as_bytes() {
            hash ^= *c as usize;
            hash *= hash.wrapping_mul(16777619);
        }
        hash
    }
}

impl core::fmt::Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.s)
    }
}