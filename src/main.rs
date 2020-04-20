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
        .parse(include_str!("../examples/boolean_precedence.kal"))
        .unwrap_or_else(|err| panic!("Failed to parse file, {:?}", err));

    // // Never deallocate the AST, because it has to live longer than garbage collected objects.
    // let ast = Box::leak(Box::new(ast));

    let mut runtime = Interpreter::new();

    let result = runtime.eval(Rc::new(ast));

    println!("{:#?}", result);
}
