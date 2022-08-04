use crate::bytecode::Opcode;

#[derive(Debug, Clone)]
pub enum Constant {
    DOUBLE(f64),
    STRING(String),
}
#[derive(Debug)]
pub struct Lineno(pub usize);
#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<(Opcode, Lineno)>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::<Constant>::new(),
        }
    }

    pub fn add_constant_double(&mut self, value: f64) -> usize {
        self.add_constant(Constant::DOUBLE(value));
        self.constants.len()
    }

    pub fn add_constant_string(&mut self, value: String) -> usize {
        self.add_constant(Constant::STRING(value));
        self.constants.len()
    }

    fn add_constant(&mut self, value: Constant) -> usize {
        self.constants.push(value);
        self.constants.len()
    }
}
