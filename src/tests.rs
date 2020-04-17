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
    assert!(
        closure(got),
        "Value returned from example did not match the expected value."
    );
}

macro_rules! test {
    {release_mode_only, $test_name:ident, $expected_val:expr } => {
        #[cfg(not(debug_assertions))]
        #[test]
        pub fn $test_name() {
            test_file(&format!("examples/{}.kal", stringify!($test_name)), |val| val == $expected_val);
        }
    };
    {$test_name:ident, $expected_val:expr} => {
        #[test]
        pub fn $test_name() {
            test_file(&format!("examples/{}.kal", stringify!($test_name)), |val| val == $expected_val);
        }
    };
}

test! { empty_file, Value::Null }

test! { let_expr_basic, Value::Int(42) }

test! { fn_add_one, Value::Int(6) }

test! { fn_nameless, Value::Int(452) }

test! { fn_nested, Value::Int(4) }

test! { fn_chained, Value::Int(23) }

test! { fn_null, Value::Null }

test! { fn_recursive_factorial, Value::Int(120) }

test! { if_expression_true, Value::Int(71) }

test! { if_expression_false, Value::Int(72) }

test! { comparison_true, Value::Bool(true) }

test! { comparison_false, Value::Bool(false) }

test! { if_expression_comparison, Value::Int(0) }

test! { release_mode_only, big_file, Value::Int(109621) }

test! { release_mode_only, big_recursive, Value::Int(1133) }

test! { object_empty, Value::Object(Rc::new(Object::new())) }

test! { object_simple,
    {
        let mut obj = Object::new();
        obj.add_binding("cat".to_owned(), Value::Int(1));
        Value::Object(Rc::new(obj))
    }
}

test! { object_access, Value::Int(2) }

test! { object_nested, Value::Int(22) }

test! { boolean_and, Value::Bool(false) }

test! { boolean_or, Value::Bool(true) }

test! { boolean_xor, Value::Bool(true) }

test! { boolean_precedence, Value::Bool(true) }

test! { symbol, Value::Symbol(0) } // first symbol is always 0

test! { symbol_as_value, Value::Symbol(1) } // second symbol is always 1

test! { symbol_equality, Value::Bool(false) }

test! { trailing_commas, Value::Int(2) }

test! { list, Value::List(Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])) }
