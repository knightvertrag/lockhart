use crate::gc::GcObject;

pub enum ObjectType {
    STRING,
    FUNCTION,
    CLASS,
}

#[repr(C)]
pub struct ObjString {
    header: GcObject,
    s: String,
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