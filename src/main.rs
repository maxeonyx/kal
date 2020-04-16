#[macro_use]
extern crate lalrpop_util;

pub mod ast;
lalrpop_mod!(pub kal_grammar);
pub mod interpreter;

mod tests;

fn main() {
    let ast = kal_grammar::BlockInnerParser::new()
        .parse(include_str!("../examples/big_file.kal"))
        .unwrap_or_else(|err| panic!("Failed to parse file, {:?}", err));

    // Never deallocate the AST, because it has to live longer than garbage collected objects.
    let ast = Box::leak(Box::new(ast));

    // println!("{:#?}", ast);
    let result = interpreter::eval(ast);

    println!("{:#?}", result);
}
