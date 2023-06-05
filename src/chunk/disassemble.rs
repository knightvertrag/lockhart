use std::{cell::RefCell, rc::Rc};

use crate::{bytecode::Opcode, chunk::Chunk};

use super::Lineno;

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    for (offset, _) in chunk.code.iter().enumerate() {
        disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    let (opcode, lineno) = chunk.code[offset];
    print!("{:04?} ", offset);
    print!("{:?} ", lineno);
    match opcode {
        Opcode::OPRETURN => {
            simple_instruction("OP_RETURN", offset);
        }
        Opcode::OPCONSTANT(idx) => {
            constant_instruction("OP_CONSTANT", &chunk, idx, offset);
        }
        Opcode::OPNEGATE => {
            simple_instruction("OP_NEGATE", offset);
        }
        Opcode::OPADD => {
            simple_instruction("OP_ADD", offset);
        }
        Opcode::OPSUBSTRACT => {
            simple_instruction("OP_SUBTRACT", offset);
        }
        Opcode::OPMULTIPLY => {
            simple_instruction("OP_MULTIPLY", offset);
        }
        Opcode::OPDIVIDE => {
            simple_instruction("OP_DIVIDE", offset);
        }
        Opcode::OPMOD => {
            simple_instruction("OP_MOD", offset);
        }
        Opcode::OPTRUE => {
            simple_instruction("OP_TRUE", offset);
        }
        Opcode::OPFALSE => {
            simple_instruction("OP_FALSE", offset);
        }
        Opcode::OPNIL => {
            simple_instruction("OP_NIL", offset);
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, idx: usize, offset: usize) -> usize {
    let constant = &chunk.constants[idx];
    println!("{name} {idx}");
    println!("{:?}", constant);
    offset + 2
}
