#[derive(Debug)]
pub enum Opcode {
    OPCONSTANT(usize),
    OPRETURN,
    OPNEGATE,
    OPADD,
    OPSUBSTRACT,
    OPMULTIPLY,
    OPDIVIDE,
    OPMOD
}

