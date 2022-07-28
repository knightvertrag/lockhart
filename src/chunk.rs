use crate::bytecode::Opcode;

#[derive(Debug)]
pub enum Constant {
    DOUBLE(f64),
    STRING(String),
}
#[derive(Debug)]
pub struct Lineno(usize);
#[derive(Debug)]
pub struct Chunk {
    code: Vec<(Opcode, Lineno)>,
    constants: Vec<Constant>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::<Constant>::new(),
        }
    }

    // pub fn add_constant_double(&mut self, value: f64) -> usize {
    //     if let Some(n) = 
    // }
    pub fn add_constant_double(&mut self, value: f64) -> usize {
        self.add_constant(Constant::DOUBLE(value));
        self.constants.len()
    }

    pub fn add_constant_string(&mut self, value: String) -> usize {
        self.add_constant(Constant::STRING(value));
        self.constants.len()
    }

    pub fn add_constant(&mut self, value: Constant) -> usize {
        self.constants.push(value);
        self.constants.len()
    }

    // pub fn find<T>(&self, key: T) -> Option<usize> {
    //     self.constants.iter().find(|x| {
    //         match x {
    //             Constant::DOUBLE(n) => 
    //         }
    //     })
    // }
}
