use std::{
    alloc::{alloc, Layout},
    ptr::{null_mut, NonNull, null}, ops::Deref,
};

use crate::{gc::GcRef, object::ObjString, value::Value};

pub struct Entry {
    key: Option<GcRef<ObjString>>,
    value: Value,
}

impl Deref for Entry {
    type Target = Entry;

    fn deref(&self) -> &Self::Target {
        self
    }
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

    pub fn find_entry(entries: *mut Entry, key: GcRef<ObjString>, capacity: usize) -> *mut Entry {
        unsafe {
            let mut index = key.hash % (capacity - 1);
            loop {
                let entry = entries.add(index);
                match (*entry).key {
                    Some(k) => {
                        if k == key {
                            return entry;
                        }
                    }
                    None => {
                        
                    }
                }
                index = (index + 1) % capacity; 
            }
        }
    }

    pub unsafe fn table_set(&mut self, key: GcRef<ObjString>, value: Value) -> bool {
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

    unsafe fn adjust_capacity(&mut self, capacity: usize) {
        let entries = alloc(Layout::array::<Entry>(capacity).unwrap()) as *mut Entry;
        for i in 0..(capacity as isize) {
            let entry = entries.offset(i);
            (*entry).key = None;
            (*entry).value = Value::NIL;
        }

        self.entries = entries;
        self.capacity = capacity;

        for i in 0..capacity {
            let entry = self.entries.add(i);
            if (*entry).key.is_none() {
                continue;
            }
        }
    }
}
