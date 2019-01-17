#[test]
pub fn let_expr_basic() {
	use crate::interpreter::{eval_block, types::*};
	use crate::kal_grammar::BlockInnerParser;

	let text = include_str!("examples/let_expr_basic.kal");

	let block = BlockInnerParser::new()
		.parse(text)
		.expect("Failed to parse example.");

	let val = eval_block(&block);

	println!("{:#?} == {:#?}", val, Value::Int(42));
	assert!(val == Value::Int(42));
}
