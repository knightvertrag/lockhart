use crate::{
    gc::GcRef,
    object::{ObjFunction, ObjString},
};

pub const STACK_SIZE: usize = 50000;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Local {
    pub name: String,
    pub depth: i8,
}

pub enum FunctionType {
    Function,
    Script,
}

pub struct CompilerState {
    pub enclosing: Option<Box<CompilerState>>,
    pub function: ObjFunction,
    pub f_type: FunctionType,
    pub locals: Vec<Local>,
    pub scope_depth: i8,
    pub total: usize,
}

impl CompilerState {
    pub fn new(function_name: GcRef<ObjString>, f_type: FunctionType) -> Box<CompilerState> {
        let function = ObjFunction::new(function_name);
        let empty_local = Local {
            name: String::new(),
            depth: -1,
        };
        Box::new(CompilerState {
            enclosing: None,
            function,
            f_type,
            locals: vec![empty_local; STACK_SIZE],
            scope_depth: 0,
            total: 1,
        })
    }
}

pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CompileError {
    fn from(value: String) -> Self {
        CompileError(value)
    }
}