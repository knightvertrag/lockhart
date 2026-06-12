use crate::{
    gc::{Gc, GcRef},
    object::ObjFunction,
    parser::ParseError,
    vm::InterpretError,
};

pub fn compile(source: String, gc: &mut Gc) -> Result<GcRef<ObjFunction>, InterpretError> {
    let program = crate::parser::parse(&source).map_err(|e| {
        InterpretError::InterpretCompileError(e.to_string())
    })?;
    crate::codegen::compile_ast(&program, gc).map_err(|e| {
        InterpretError::InterpretCompileError(e.to_string())
    })
}

impl From<ParseError> for InterpretError {
    fn from(err: ParseError) -> Self {
        InterpretError::InterpretCompileError(err.to_string())
    }
}