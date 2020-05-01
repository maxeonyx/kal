#[cfg(test)]
mod tests;

mod ast;
mod eval;
mod eval_impls;
mod interpreter;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(#[allow(clippy::all)] pub kal_grammar);

use interpreter::Interpreter;

fn main() {
    let ast = kal_grammar::BlockInnerParser::new()
        .parse(include_str!("../examples/handle_implicit.kal"))
        .unwrap_or_else(|err| panic!("Failed to parse file, {:?}", err));

    let mut interpreter = Interpreter::new();

    let result = interpreter.eval(ast);

    println!("{:#?}", result);
}
