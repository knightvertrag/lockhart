#[allow(non_camel_case_types)]

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    OP_CONSTANT(usize),
    OP_RETURN,
    //operators
    OP_NEGATE,
    OP_ADD,
    OP_SUBSTRACT,
    OP_MULTIPLY,
    OP_DIVIDE,
    OP_MOD,
    // literals
    OP_TRUE,
    OP_FALSE,
    OP_NOT,
    OP_NIL,
    // comaparators
    OP_EQ,
    OP_GT,
    OP_LT,
    // declarations
    OP_DEFINE_GLOBAL(usize),
    OP_GET_GLOBAL(usize),
    OP_SET_GLOBAL(usize),
    OP_PRINT,
    OP_POP,
}
