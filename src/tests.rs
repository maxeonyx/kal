use crate::interpreter::{eval, types::Value};

#[allow(dead_code)]
fn test_file(path: &str, closure: impl Fn(Value) -> bool) {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Could not read file {:?}", path));
    let ast = crate::kal_grammar::BlockInnerParser::new()
        .parse(&text)
        .unwrap_or_else(|_| panic!("Failed to parse file {:?}.", path));
    let got = eval(&ast);
    assert!(closure(got));
}

#[test]
pub fn let_expr_basic() {
    test_file("examples/let_expr_basic.kal", |val| val == Value::Int(42));
}

#[test]
pub fn add_one() {
    test_file("examples/add_one.kal", |val| val == Value::Int(6));
}

#[test]
pub fn nameless() {
    test_file("examples/nameless.kal", |val| val == Value::Int(452));
}

#[test]
pub fn nested() {
    test_file("examples/nested.kal", |val| val == Value::Int(4))
}

#[test]
pub fn string_1() {
    test_file("examples/string_1.kal", |val| {
        val == Value::Str("Hello".to_owned())
    })
}

#[test]
pub fn string_2() {
    test_file("examples/string_2.kal", |val| {
        val == Value::Str("World\n".to_owned())
    })
}
