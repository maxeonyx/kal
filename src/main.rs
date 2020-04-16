#[macro_use]
extern crate lalrpop_util;

pub mod ast;
lalrpop_mod!(pub kal_grammar);
pub mod interpreter;

mod tests;

fn main() {
    println!(
        "{:#?}",
        kal_grammar::BlockInnerParser::new().parse(include_str!("../examples/null_function.kal"))
    );
}
