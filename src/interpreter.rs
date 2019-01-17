use crate::ast::{Block, Expression, LetExpression, Literal};

pub mod types {
	use std::collections::HashMap;

	#[derive(Debug, Clone, PartialEq, Eq)]
	pub enum Value {
		Null,
		Bool(bool),
		Str(String),
		Int(i64),
		List(Vec<Value>),
		Object(Object),
	}

	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Object {
		pub inner: HashMap<String, Value>,
	}

	impl Object {
		pub fn new() -> Self {
			Object {
				inner: HashMap::new(),
			}
		}
	}
}

use self::types::*;

pub struct Context {
	pub scope: Vec<Object>,
}

impl Context {
	pub fn new() -> Self {
		Context {
			scope: vec![Object::new()],
		}
	}
}

pub fn eval_block(block: &Block) -> Value {
	let mut ctx = Context::new();

	let mut last_val = Value::Null;
	for expr in block.expressions.iter() {
		last_val = eval_expression(&mut ctx, expr);
	}

	last_val
}

fn eval_expression(ctx: &mut Context, expr: &Expression) -> Value {
	match expr {
		Expression::Let(expr) => assign_var(ctx, &expr),
		Expression::Literal(ref literal) => eval_literal(literal),
		_ => panic!("Only let and literal expressions are implemented."), // TODO
	}
}

fn eval_literal(literal: &Literal) -> Value {
	match literal {
		Literal::Int(num) => Value::Int(*num),
		_ => panic!("Only int literals are implemented."),
	}
}

// `let` expressions return a value and so be chained.
// `let x = let y = 42;` will assign both x and y to 42.
fn assign_var(ctx: &mut Context, expr: &LetExpression) -> Value {
	let LetExpression { variable, expr } = expr;

	let value = eval_expression(ctx, expr);

	// `unwrap` is OK because context always has at least one scope object
	let current_scope_object = ctx.scope.last_mut().unwrap();

	current_scope_object
		.inner
		.insert(variable.name.clone(), value.clone());

	value
}
