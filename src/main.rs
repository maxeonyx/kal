#[cfg(test)]
mod tests;

mod ast;
mod eval;
mod eval_impls;
mod interpreter;
mod intrinsics;

use std::path::PathBuf;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(#[allow(clippy::all)] pub kal_grammar);

use interpreter::Interpreter;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    filename: PathBuf,
}

fn main() {

    let args = Args::parse();
    let borrowed_path = args.filename.as_path();
    let file = std::fs::read_to_string(borrowed_path).unwrap_or_else(|err| panic!("Failed to read file {:}. Error: {:?}", borrowed_path.to_string_lossy(), err));

    let ast = kal_grammar::BlockInnerParser::new()
        .parse(file.as_str())
        .unwrap_or_else(|err| panic!("Failed to parse file, {:?}", err));

    let mut interpreter = Interpreter::new();

    let result = interpreter.eval(ast);

    println!("{:#?}", result);
}
