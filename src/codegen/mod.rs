mod compiler;
mod emit;

pub use compiler::CompileError;
pub use emit::Codegen;

use crate::{
    ast::Program,
    gc::{Gc, GcRef},
    object::ObjFunction,
};

pub fn compile_ast(program: &Program, gc: &mut Gc) -> Result<GcRef<ObjFunction>, CompileError> {
    let mut gen = Codegen::new_script(gc);
    gen.emit_program(program)?;
    gen.finish()
}