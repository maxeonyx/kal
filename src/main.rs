use new_interpreter::Interpreter;
use std::rc::Rc;

#[macro_use]
extern crate lalrpop_util;

#[cfg(test)]
mod tests;

mod ast;
// mod interpreter;
mod kal_ref;
mod new_interpreter;

lalrpop_mod!(#[allow(clippy::all)] pub kal_grammar);

fn main() {
    let ast = kal_grammar::BlockInnerParser::new()
        .parse(include_str!("../examples/handle.kal"))
        .unwrap_or_else(|err| panic!("Failed to parse file, {:?}", err));

    let mut runtime = Interpreter::new();

    let result = runtime.eval(Rc::new(ast));

    println!("{:#?}", result);
}
