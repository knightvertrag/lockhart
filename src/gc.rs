use std::{
    fmt::{Display, Write},
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    object::{self, ObjString, ObjectType},
    table::Table,
    value::Value,
};

#[repr(C)]
pub struct GcObject {
    marked: bool,
    next: Option<NonNull<GcObject>>,
    obj_type: ObjectType,
}

impl GcObject {
    pub fn new(obj_type: ObjectType) -> GcObject {
        GcObject {
            marked: false,
            next: None,
            obj_type,
        }
    }
}
pub struct GcRef<T> {
    pointer: NonNull<T>,
}

impl<T> GcRef<T> {}

impl<T> Deref for GcRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T> DerefMut for GcRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.pointer.as_mut() }
    }
}

impl<T> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
        }
    }
}

impl<T> Copy for GcRef<T> {}

impl<T> PartialEq for GcRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
}

impl<T> Eq for GcRef<T> {}

pub struct Gc {
    next_gc: usize,
    first: Option<NonNull<GcObject>>,
    strings: Table,
    grey_stack: Vec<NonNull<GcObject>>,
}

impl Gc {
    const HEAP_GROW_FACTOR: usize = 2;
    pub fn new() -> Gc {
        Gc {
            next_gc: 1024 * 1024,
            first: None,
            strings: Table::new(),
            grey_stack: Vec::new(),
        }
    }

    fn alloc<T>(&mut self, object: T) -> GcRef<T> {
        unsafe {
            let boxed_o = Box::new(object);
            let ptr = NonNull::new_unchecked(Box::into_raw(boxed_o));
            let mut header: NonNull<GcObject> = mem::transmute(ptr.as_ref());
            header.as_mut().next = self.first.take();
            self.first = Some(header);
            GcRef { pointer: ptr }
        }
    }

    pub fn intern(&mut self, s: String) -> GcRef<ObjString> {
        let o_string = ObjString::from_string(s);
        if let Some(value) = self.strings.find_string(&o_string.s, o_string.hash) {
            value
        } else {
            let reference = self.alloc(o_string);
            self.strings.set(reference, Value::NIL);
            reference
        }
    }
}
