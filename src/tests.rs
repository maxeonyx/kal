use crate::interpreter::{
    eval,
    types::{Object, Value},
};
use std::rc::Rc;

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
pub fn fn_add_one() {
    test_file("examples/fn_add_one.kal", |val| val == Value::Int(6));
}

#[test]
pub fn fn_nameless() {
    test_file("examples/fn_nameless.kal", |val| val == Value::Int(452));
}

#[test]
pub fn fn_nested() {
    test_file("examples/fn_nested.kal", |val| val == Value::Int(4))
}

#[test]
pub fn fn_chained() {
    test_file("examples/fn_chained.kal", |val| val == Value::Int(23))
}

#[test]
pub fn fn_null() {
    test_file("examples/fn_null.kal", |val| val == Value::Null)
}

#[test]
pub fn fn_recursive_factorial() {
    test_file("examples/fn_recursive_factorial.kal", |val| {
        val == Value::Int(120)
    })
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

#[test]
pub fn object_empty() {
    test_file("examples/object_empty.kal", |val| {
        val == Value::Object(Rc::new(Object::new()))
    })
}

#[test]
pub fn object_simple() {
    let mut obj = Object::new();
    obj.add_binding("cat".to_owned(), Value::Int(1));
    let obj = Value::Object(Rc::new(obj));

    test_file("examples/object_simple.kal", |val| val == obj)
}

#[test]
pub fn object_access() {
    test_file("examples/object_access.kal", |val| val == Value::Int(2))
}

#[test]
pub fn object_nested() {
    test_file("examples/object_nested.kal", |val| val == Value::Int(22))
}

#[test]
pub fn boolean_and() {
    test_file("examples/boolean_and.kal", |val| val == Value::Bool(false))
}

#[test]
pub fn boolean_or() {
    test_file("examples/boolean_or.kal", |val| val == Value::Bool(true))
}

#[test]
pub fn boolean_xor() {
    test_file("examples/boolean_xor.kal", |val| val == Value::Bool(true))
}

#[test]
pub fn boolean_precedence() {
    test_file("examples/boolean_precedence.kal", |val| {
        val == Value::Bool(true)
    })
}

#[test]
pub fn symbol() {
    test_file("examples/symbol.kal", |val| {
        val == Value::Symbol(0) // first symbol is always 0
    })
}

#[test]
pub fn symbol_equality() {
    test_file("examples/symbol_equality.kal", |val| {
        val == Value::Bool(false)
    })
}

#[test]
pub fn symbol_as_value() {
    test_file("examples/symbol_as_value.kal", |val| {
        val == Value::Symbol(1) // second symbol is always 1
    })
}
