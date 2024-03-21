use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::null_mut,
};

use crate::{gc::GcRef, object::ObjString, value::Value};

pub struct Entry {
    key: Option<GcRef<ObjString>>,
    value: Value,
}

pub struct Table {
    count: usize,
    capacity: usize,
    entries: *mut Entry,
}

impl Table {
    const MAX_LOAD: f32 = 0.75;
    pub fn new() -> Table {
        Table {
            count: 0,
            capacity: 0,
            entries: null_mut(),
        }
    }

    pub fn find_string(&mut self, s: &str, hash: usize) -> Option<GcRef<ObjString>> {
        unsafe {
            if self.count == 0 {
                return None;
            }
            let mut index = hash % (self.capacity - 1);
            loop {
                let entry = self.entries.offset(index as isize);
                match (*entry).key {
                    Some(key) => {
                        if s.len() == key.s.len() && hash == key.hash && s == key.s {
                            return Some(key);
                        }
                    }
                    None => {
                        if let Value::NIL = (*entry).value {
                            return None;
                        }
                    }
                }
                index = (index + 1) % (self.capacity - 1);
            }
        }
    }

    pub fn find_entry(entries: *mut Entry, key: GcRef<ObjString>, capacity: usize) -> *mut Entry {
        unsafe {
            let mut index = key.hash % (capacity - 1);
            let mut tombstone: *mut Entry = null_mut();
            loop {
                let entry = entries.add(index);
                match (*entry).key {
                    Some(k) => {
                        if k == key {
                            return entry;
                        }
                    }
                    None => {
                        if let Value::NIL = (*entry).value {
                            return if !tombstone.is_null() {
                                tombstone
                            } else {
                                entry
                            };
                        } else if tombstone.is_null() {
                            tombstone = entry;
                        }
                    }
                }
                index = (index + 1) % (capacity - 1);
            }
        }
    }

    pub fn delete_entry(&mut self, key: GcRef<ObjString>) -> bool {
        unsafe {
            if self.count == 0 {
                return false;
            }
            let entry = Table::find_entry(self.entries, key, self.capacity);
            if (*entry).key.is_none() {
                return false;
            }
            (*entry).key = None;
            (*entry).value = Value::BOOL(true);
            true
        }
    }

    pub fn iter(&self) -> IterTable {
        IterTable {
            ptr: self.entries,
            end: unsafe { self.entries.add(self.capacity) },
        }
    }

    // set and return true if new key
    pub fn set(&mut self, key: GcRef<ObjString>, value: Value) -> bool {
        unsafe {
            let entry = Table::find_entry(self.entries, key, self.capacity);
            let is_new_key = (*entry).key.is_none();
            if is_new_key {
                self.count += 1;
            }

            (*entry).key = Some(key);
            (*entry).value = value;
            is_new_key
        }
    }

    pub fn get(&self, key: GcRef<ObjString>) -> Option<Value> {
        if self.count == 0 {
            return None;
        }

        let entry = Table::find_entry(self.entries, key, self.capacity);
        unsafe {
            if (*entry).key.is_none() {
                return None;
            } else {
                return Some((*entry).value.clone());
            }
        }
    }
    pub fn add_all(&mut self, from: &Table) {
        unsafe {
            for i in 0..(from.capacity as isize) {
                let entry = from.entries.offset(i);
                if let Some(key) = (*entry).key {
                    self.set(key, (*entry).value.clone());
                }
            }
        }
    }

    unsafe fn adjust_capacity(&mut self, capacity: usize) {
        // allocate and initialize the table
        let entries = alloc(Layout::array::<Entry>(capacity).unwrap()) as *mut Entry;
        for i in 0..(capacity as isize) {
            let entry = entries.offset(i);
            (*entry).key = None;
            (*entry).value = Value::NIL;
        }

        // insert all entries back into reallocated table
        // reset count because tombstones eliminated during reallocation
        self.count = 0;
        for i in 0..capacity {
            let entry = self.entries.add(i);
            if let Some(k) = (*entry).key {
                let dest = Table::find_entry(entries, k, self.capacity);
                (*dest).key = (*entry).key;
                (*dest).value = (*entry).value.clone();
                self.count += 1;
            } else {
                continue;
            }
        }

        dealloc(
            self.entries.cast(),
            Layout::array::<Entry>(self.capacity).unwrap(),
        );

        // point to reallocated entries
        self.entries = entries;
        self.capacity = capacity;
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        unsafe {
            if !self.entries.is_null() {
                dealloc(
                    self.entries.cast(),
                    Layout::array::<Entry>(self.capacity).unwrap(),
                );
            }
        }
    }
}

pub struct IterTable {
    ptr: *mut Entry,
    end: *const Entry,
}
impl Iterator for IterTable {
    type Item = (GcRef<ObjString>, Value);

    fn next(&mut self) -> Option<Self::Item> {
        while self.ptr as *const Entry != self.end {
            unsafe {
                let entry = self.ptr;
                self.ptr = self.ptr.offset(1);
                if let Some(key) = (*entry).key {
                    return Some((key, (*entry).value.clone()));
                }
            }
        }
        None
    }
}
