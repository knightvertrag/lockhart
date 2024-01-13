use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::object::ObjectType;

#[repr(C)]
pub struct GcObject {
    marked: bool,
    next: Option<NonNull<GcObject>>,
    obj_type: ObjectType,
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
        Self { pointer: self.pointer.clone() }
    }
}

impl<T> Copy for GcRef<T> {}

impl<T> PartialEq for GcRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
}

impl<T> Eq for GcRef<T> {}