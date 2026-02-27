use std::{
    mem,
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    object::{ObjFunction, ObjString, ObjectType},
    table::Table,
    value::Value,
};

#[repr(C)]
pub struct GcObject {
    marked: bool,
    next: Option<NonNull<GcObject>>,
    obj_type: ObjectType,
    size: usize,
}

impl GcObject {
    pub fn new(obj_type: ObjectType, size: usize) -> GcObject {
        GcObject {
            marked: false,
            next: None,
            obj_type,
            size,
        }
    }
}
pub struct GcRef<T> {
    pointer: NonNull<T>,
}

impl<T> GcRef<T> {
    pub fn dangling() -> GcRef<T> {
        GcRef {
            pointer: NonNull::dangling(),
        }
    }
}

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
    bytes_allocated: usize,
    next_gc: usize,
    first: Option<NonNull<GcObject>>,
    strings: Table,
    grey_stack: Vec<NonNull<GcObject>>,
}

impl Gc {
    const HEAP_GROW_FACTOR: usize = 2;
    pub fn new() -> Gc {
        Gc {
            bytes_allocated: 0,
            next_gc: 1024 * 1024,
            first: None,
            strings: Table::new(),
            grey_stack: Vec::new(),
        }
    }

    pub fn alloc<T>(&mut self, object: T) -> GcRef<T> {
        unsafe {
            self.bytes_allocated += size_of::<T>();
            let boxed_o = Box::new(object);
            let ptr = NonNull::new_unchecked(Box::into_raw(boxed_o));
            // extract *mut T, then cast to GcObject, then transmute to NonNull 
            let mut header: NonNull<GcObject> = mem::transmute(ptr.as_ptr());
            header.as_mut().next = self.first.take();
            self.first = Some(header);
            GcRef { pointer: ptr }
        }
    }

    // check if string is already interned, if not then allocate and return reference
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

    pub fn should_collect(&self) -> bool {
        self.bytes_allocated > self.next_gc
    }

    pub fn mark_object<T>(&mut self, reference: GcRef<T>) {
        unsafe {
            let mut object = NonNull::new_unchecked(reference.pointer.as_ptr() as *mut GcObject);
            if object.as_ref().marked {
                return;
            }
            object.as_mut().marked = true;
            self.grey_stack.push(object);
        }
    }

    pub fn mark_value(&mut self, value: &Value) {
        if let Value::STR(s) = value {
            self.mark_object(*s);
        } else if let Value::FUNCTION(f) = value {
            self.mark_object(*f);
        }
    }

    pub fn mark_table(&mut self, table: &Table) {
        for (key, value) in table.iter() {
            self.mark_object(key);
            self.mark_value(&value);
        }
    }

    pub fn collect_garbage(&mut self) {
        self.trace_references();
        self.remove_white_strings();
        self.sweep();
        self.next_gc = (self.bytes_allocated * Gc::HEAP_GROW_FACTOR).max(1024 * 1024);
    }

    fn trace_references(&mut self) {
        while let Some(object) = self.grey_stack.pop() {
            self.blacken_object(object);
        }
    }

    fn blacken_object(&mut self, object: NonNull<GcObject>) {
        unsafe {
            match object.as_ref().obj_type {
                ObjectType::STRING => {}
                ObjectType::FUNCTION => {
                    let function = object.cast::<ObjFunction>();
                    self.mark_object(function.as_ref().name);
                    for value in &function.as_ref().chunk.constants {
                        self.mark_value(value);
                    }
                }
                ObjectType::CLASS => {}
            }
        }
    }

    fn remove_white_strings(&mut self) {
        let mut keys_to_remove = Vec::new();
        for (key, _) in self.strings.iter() {
            if !Gc::is_marked(key) {
                keys_to_remove.push(key);
            }
        }
        for key in keys_to_remove {
            self.strings.delete_entry(key);
        }
    }

    fn is_marked<T>(reference: GcRef<T>) -> bool {
        unsafe {
            let object = NonNull::new_unchecked(reference.pointer.as_ptr() as *mut GcObject);
            object.as_ref().marked
        }
    }

    fn sweep(&mut self) {
        unsafe {
            let mut previous: Option<NonNull<GcObject>> = None;
            let mut current = self.first;

            while let Some(mut object) = current {
                if object.as_ref().marked {
                    object.as_mut().marked = false;
                    previous = Some(object);
                    current = object.as_ref().next;
                } else {
                    let unreached = object;
                    current = object.as_ref().next;

                    if let Some(mut prev) = previous {
                        prev.as_mut().next = current;
                    } else {
                        self.first = current;
                    }

                    self.bytes_allocated = self.bytes_allocated.saturating_sub(unreached.as_ref().size);
                    self.free_object(unreached);
                }
            }
        }
    }

    fn free_object(&mut self, object: NonNull<GcObject>) {
        unsafe {
            match object.as_ref().obj_type {
                ObjectType::STRING => {
                    drop(Box::from_raw(object.cast::<ObjString>().as_ptr()));
                }
                ObjectType::FUNCTION => {
                    drop(Box::from_raw(object.cast::<ObjFunction>().as_ptr()));
                }
                ObjectType::CLASS => {}
            }
        }
    }
}

impl Drop for Gc {
    fn drop(&mut self) {
        unsafe {
            let mut current = self.first;
            while let Some(object) = current {
                current = object.as_ref().next;
                self.free_object(object);
            }
        }
    }
}
