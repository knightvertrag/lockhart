use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::vm::Vm;

pub fn open_source_file(file_name: &str) -> String {
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::open(path) {
        Err(err) => panic!("Could not open file {}: {}", display, err),
        Ok(file) => file,
    };

    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => return buf,
        Err(err) => panic!("Could not read data from file: {}", err),
    }
}

pub fn execute(code: String) {
    let mut interpreter = Vm::init_vm();
    match interpreter.interpret(code) {
        Ok(_) => (),
        Err(err) => println!("Error: {:?}", err),
    }
}