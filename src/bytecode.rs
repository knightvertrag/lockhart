#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    OPCONSTANT(usize),
    OPRETURN,
    OPNEGATE,
    OPADD,
    OPSUBSTRACT,
    OPMULTIPLY,
    OPDIVIDE,
    OPMOD,
    OPTRUE,
    OPFALSE,
    OPNOT,
    // comaparators
    OPEQ,
    OPGT,
    OPLT,
    OPPRINT,
    OPNIL,
}
