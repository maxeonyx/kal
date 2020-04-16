use crate::interpreter::{eval, types::Value};

#[allow(dead_code)]
fn test_file(path: &str, closure: impl Fn(Value) -> bool) {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Could not read file {:?}", path));
    let ast = Box::new(
        crate::kal_grammar::BlockInnerParser::new()
            .parse(&text)
            .unwrap_or_else(|_| panic!("Failed to parse file {:?}.", path)),
    );
    // We ensure that the AST lives longer than any garbage collected objects by giving it a 'static
    // lifetime by leaking it.
    let ast = Box::leak(ast);
    let got = eval(ast);
    assert!(closure(got));
}

#[test]
pub fn empty_file() {
    test_file("examples/empty_file.kal", |val| val == Value::Null);
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
pub fn chained() {
    test_file("examples/chained.kal", |val| val == Value::Int(23))
}

#[test]
pub fn null_function() {
    test_file("examples/null_function.kal", |val| val == Value::Null)
}

#[test]
pub fn if_expression_true() {
    test_file("examples/if_expression_true.kal", |val| {
        val == Value::Int(71)
    })
}

#[test]
pub fn if_expression_false() {
    test_file("examples/if_expression_false.kal", |val| {
        val == Value::Int(72)
    })
}

#[test]
pub fn comparison_true() {
    test_file("examples/comparison_true.kal", |val| {
        val == Value::Bool(true)
    })
}

#[test]
pub fn comparison_false() {
    test_file("examples/comparison_false.kal", |val| {
        val == Value::Bool(false)
    })
}

#[test]
pub fn if_expression_comparison() {
    test_file("examples/if_expression_comparison.kal", |val| {
        val == Value::Int(0)
    })
}

#[test]
pub fn recursive_factorial() {
    test_file("examples/recursive_factorial.kal", |val| {
        val == Value::Int(120)
    })
}

#[cfg(not(debug_assertions))]
#[test]
pub fn big_file() {
    test_file("examples/big_file.kal", |val| val == Value::Int(109621))
}

#[cfg(not(debug_assertions))]
#[test]
pub fn big_recursive() {
    test_file("examples/big_recursive.kal", |val| val == Value::Int(1133))
}
