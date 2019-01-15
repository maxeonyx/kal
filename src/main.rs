#[macro_use]
extern crate lalrpop_util;

pub mod ast;
lalrpop_mod!(pub kal_grammar);

fn main() {
    println!(
        "{:#?}",
        kal_grammar::BlockInnerParser::new().parse(include_str!("./fake_example.kal"))
    );
}
