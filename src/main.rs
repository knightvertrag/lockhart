use std::{env, io, process};

mod ast;
mod bytecode;
mod chunk;
mod codegen;
mod compiler;
mod gc;
mod lexer;
mod object;
mod parser;
mod repl;
mod source;
mod table;
mod token;
mod value;
mod vm;

use ast::pretty::{dump_program, DumpFormat};
use source::{execute, open_source_file};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--dump-ast") {
        dump_ast_cli(&args);
        return Ok(());
    }

    if args.len() == 1 {
        println!("===============Lockhart initiated===============");
        repl::start();
    } else {
        let src_filename = &args[1];
        let code = open_source_file(&src_filename);
        execute(code);
    }
    Ok(())
}

fn dump_ast_cli(args: &[String]) {
    let format = parse_dump_format(args);
    let file = match find_dump_ast_file(args) {
        Some(path) => path,
        None => {
            eprintln!("Usage: lockhart --dump-ast [--format tree|json] <file.lh>");
            process::exit(1);
        }
    };

    let code = open_source_file(&file);
    match parser::parse(&code) {
        Ok(program) => {
            println!("{}", dump_program(&program, format));
        }
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}

fn parse_dump_format(args: &[String]) -> DumpFormat {
    let mut format = DumpFormat::Tree;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--format" {
            if let Some(value) = args.get(i + 1) {
                format = match value.as_str() {
                    "json" => DumpFormat::Json,
                    "tree" => DumpFormat::Tree,
                    other => {
                        eprintln!("Unknown format '{other}'. Use 'tree' or 'json'.");
                        process::exit(1);
                    }
                };
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    format
}

fn find_dump_ast_file(args: &[String]) -> Option<String> {
    let flags_with_value = ["--format"];
    let mut path = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--dump-ast" || arg == flags_with_value[0] {
            if flags_with_value.contains(&arg.as_str()) {
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if arg.starts_with('-') {
            i += 1;
            continue;
        }
        path = Some(arg.clone());
        i += 1;
    }
    path
}