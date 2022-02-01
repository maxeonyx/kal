use crate::interpreter::{Interpreter, Key, Value::{self, *}};
use std::{collections::HashMap, rc::Rc};

#[allow(dead_code)]
fn eval_file(path: &str) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Could not read file {:?}", path));
    let ast = crate::kal_grammar::BlockInnerParser::new()
        .parse(&text)
        .unwrap_or_else(|_| panic!("Failed to parse file {:?}.", path));
    let mut runtime = Interpreter::new();

    runtime.eval(ast)
}

macro_rules! test {
    {release_mode_only, $test_name:ident, $expected_val:expr } => {
        #[cfg(not(debug_assertions))]
        #[test]
        pub fn $test_name() {
            let val = eval_file(&format!("tests/{}.kal", stringify!($test_name)));
            let expected = $expected_val;
            assert!(val == expected, "Assertion failed: got {:?}, expected {:?}.", val, expected);
        }
    };
    {$test_name:ident, $expected_val:expr} => {
        #[test]
        pub fn $test_name() {
            let val = eval_file(&format!("tests/{}.kal", stringify!($test_name)));
            let expected = $expected_val;
            assert!(val == expected, "Assertion failed: got {:?}, expected {:?}.", val, expected);
        }
    };
}

macro_rules! test_error {
    {$test_name:ident} => {
        #[test]
        #[should_panic]
        pub fn $test_name() {
            let val = eval_file(&format!("tests/{}.error.kal", stringify!($test_name)));
        }
    };
}

#[cfg(not(debug_assertions))]
#[test]
fn big_file() {
    let size = 4_000_000_i64;
    let let_statements = "let num=num+1;".repeat(size as usize);
    let text = format!("let num = 0; {} num", let_statements);
    let ast = crate::kal_grammar::BlockInnerParser::new()
        .parse(&text)
        .unwrap();
    let mut runtime = Interpreter::new();
    let val = runtime.eval(ast);
    assert!(val == Int(size));
}

test! { empty_file, Null }

test! { let_expr_basic, Int(42) }

test! { fn_add_one, Int(6) }

test! { fn_nameless, Int(452) }

test! { fn_nested, Int(4) }

test! { fn_chained, Int(23) }

test! { fn_object_empty, Object(Rc::new(HashMap::new())) }

test! { fn_object, 
    {
        let mut obj = HashMap::new();
        obj.insert(Key::Str("cat".to_owned()), Int(1));
        Object(Rc::new(obj))
    } }

test! { fn_null, Null }
test! { fn_named, Bool(true) }

test! { fn_multiple_statements, Int(100) }

test! { fn_recursive_factorial, Int(120) }
test! { fibonacci, List(
    Rc::new(vec![
        Int(0),
        Int(1),
        Int(1),
        Int(2),
        Int(3),
        Int(5),
        Int(8),
    ]))
}

test! { if_true, Int(71) }

test! { if_false, Int(72) }

test! { if_comparison, Int(0) }

test! { if_else_if, Int(77) }

test! { if_without_else, Null }

test! { comparison_true, Bool(true) }

test! { comparison_false, Bool(false) }

test! { release_mode_only, big_recursive, Int(1_000_000) }

test! { object_empty, Object(Rc::new(HashMap::new())) }

test! { object_simple,
    {
        let mut obj = HashMap::new();
        obj.insert(Key::Str("cat".to_owned()), Int(1));
        Object(Rc::new(obj))
    }
}

test! { object_access, Int(2) }

test! { object_access_expression, Int(251) }

test! { object_access_fn, Int(491) }

test! { object_nested, Int(22) }

test! { object_spread, List(Rc::new(vec![Int(15), Int(20), Int(99)])) }

test! { boolean_and, Bool(false) }

test! { boolean_or, Bool(true) }

test! { boolean_xor, Bool(true) }

test! { boolean_precedence, Bool(true) }

test! { symbol, Symbol(0) } // first symbol is always 0

test! { symbol_as_value, Symbol(1) } // second symbol is always 1

test! { symbol_equality, Bool(true) }

test! { trailing_commas, Int(2) }

test! { list, List(Rc::new(vec![Int(1), Int(2), Int(3)])) }

test! { list_index, Int(29) }

test! { list_index_expression, Int(32) }

test! { list_negative_index, Int(53) }

test! { list_spread, List(Rc::new(vec![Int(1), Int(2), Int(3), Int(4), Int(5), Int(6)])) }

test! { int, Int(5) }

test! { int_negative, Int(-5) }

test! { num_addition, Int(8) }

test! { num_subtraction, Int(-2) }

test! { num_multiplication, Int(15) }

test! { num_negative_subtraction, Int(14) }

test! { num_division, Int(2) }

test! { mut_num, Int(2) }

test! { mut_multi, Int(111) }

test! { mut_multi_let, Int(999) }

test! { mut_increment, Int(61) }

test! { mut_list_index, Int(99) }

test! { mut_object_access, Bool(true) }

test! { mut_deep, Int(3) }

test! { handle_continue, Int(9) }

test! { handle_multiple_continue, Int(81) }

test! { handle_no_continue, Int(3) }

test! { handle_nested, Int(55) }

test! { handle_nested_continue, Int(20) }

test! { handle_two_effect_types, Int(25) }

#[test]
fn handle_implicit() {
    let val = eval_file("tests/handle_implicit.kal");
    match val {
        Effect(effect) => assert!(effect.value == Int(4)),
        _ => panic!("Expected an effect value, got something else."),
    }
}

#[test]
fn handle_empty() {
    let val = eval_file("tests/handle_empty.kal");
    match val {
        Effect(effect) => assert!(effect.value == Bool(true)),
        _ => panic!("Expected an effect value, got something else."),
    }
}

#[test]
fn size_of_value() {
    // The Value type is a Rust enum. It has 8 bytes for the discriminant, plus
    // the size in bytes of the largest variant. I would like all variants to be
    // 8 bytes or smaller for performance
    assert_eq!(std::mem::size_of::<Value>(), 16);
}

test! { comments, Int(3) }

test! { loop_break, Null }

test! { loop_break_value, Int(77) }

test! { loop_yield, Int(8) }

test! { loop_continue, Int(5) }

test! { loop_collect, List(Rc::new(vec![Int(0), Int(1), Int(2), Int(3), Int(4)])) }

test! { expression_as_statement, Int(2) }


test! { pattern_fn_spread_last, Bool(true) }
test! { pattern_fn_spread_nameless, Bool(true) }
test! { pattern_fn_spread_only, Bool(true) }
test! { pattern_fn_spread_spread_both, Bool(true) }
test! { pattern_let_list_spread_nameless_only, Null }
test! { pattern_let_list_spread_nameless, Bool(true) }
test! { pattern_let_list_spread, Bool(true) }
test! { pattern_let_list, Bool(true) }
test! { pattern_let_list_nested, Bool(true) }
test_error! { pattern_let_list_spread_too_many }
test_error! { pattern_let_list_spread_not_enough }
test_error! { pattern_let_list_spread_not_enough_spread }
test! { pattern_let_list_empty, Null }
test! { pattern_let_object, Bool(true) }
test! { pattern_let_object_property, Bool(true) }
test! { pattern_let_object_spread_nameless, Bool(true) }
test! { pattern_let_object_spread_nameless_only, Null }
test! { pattern_let_object_nested, Bool(true) }
test! { pattern_let_object_wildcard, Bool(true) }
