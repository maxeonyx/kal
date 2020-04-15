use crate::ast::{
    AddExpression, Block, Expression, FunctionInvocation, Ident, LetExpression, Literal,
};

pub mod types {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value<'ast> {
        Null,
        Bool(bool),
        Str(String),
        Int(i64),
        List(Vec<Value<'ast>>),
        Object(Object<'ast>),
        Closure(Closure<'ast>),
    }

    // TODO: not really a closure yet. Doesn't capture lexical scope
    #[derive(Debug, Clone)]
    pub struct Closure<'ast> {
        pub code: &'ast crate::ast::Function,
    }

    impl<'ast> PartialEq for Closure<'ast> {
        fn eq(&self, _other: &Closure<'ast>) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Object<'ast> {
        pub inner: HashMap<String, Value<'ast>>,
    }

    impl<'ast> Object<'ast> {
        pub fn new() -> Self {
            Object {
                inner: HashMap::new(),
            }
        }
    }
}

use self::types::*;

pub struct Context<'ast> {
    pub scopes: Vec<Object<'ast>>,
}

impl<'ast> Context<'ast> {
    pub fn new() -> Self {
        Context {
            scopes: vec![Object::new()],
        }
    }
}

pub fn eval<'ast>(ast: &'ast Block) -> Value<'ast> {
    let mut ctx = Context::new();

    eval_block(&mut ctx, ast)
}

fn eval_block<'ast>(ctx: &mut Context<'ast>, block: &'ast Block) -> Value<'ast> {
    ctx.scopes.push(Object::new());

    let mut last_val = Value::Null;
    for expr in block.expressions.iter() {
        last_val = eval_expression(ctx, expr);
    }

    last_val
}

fn eval_expression<'ast>(ctx: &mut Context<'ast>, expr: &'ast Expression) -> Value<'ast> {
    match expr {
        Expression::Let(expr) => assign_var(ctx, expr),
        Expression::Literal(literal) => eval_literal(literal),
        Expression::FunctionInvocation(func_invo) => eval_function_invocation(ctx, func_invo),
        Expression::Add(add_expr) => eval_add(ctx, add_expr),
        Expression::Ident(ident) => eval_ident(ctx, ident),
        _ => unimplemented!("Expression type {:?}.", expr), // TODO
    }
}

fn eval_ident<'ast>(ctx: &mut Context<'ast>, ident: &'ast Ident) -> Value<'ast> {
    resolve_name(ctx, &ident.name)
}

fn eval_add<'ast>(ctx: &mut Context<'ast>, expr: &'ast AddExpression) -> Value<'ast> {
    let AddExpression { left, right } = expr;
    let left = eval_expression(ctx, left);
    let right = eval_expression(ctx, right);
    let left = match left {
        Value::Int(i) => i,
        _ => panic!("Cant add, left side not an Int: {:?}.", expr),
    };
    let right = match right {
        Value::Int(i) => i,
        _ => panic!("Cant add, right side not an Int: {:?}.", expr),
    };
    Value::Int(left + right)
}

fn resolve_name<'ast>(ctx: &mut Context<'ast>, name: &str) -> Value<'ast> {
    for scope in ctx.scopes.iter().rev() {
        if scope.inner.contains_key(name) {
            // TODO: Garbage collection means we won't have to clone
            return scope.inner.get(name).unwrap().clone();
        }
    }
    println!("    =================    ");
    println!("{:?}", ctx.scopes);
    panic!("Could not resolve name {:?}", name);
}

fn eval_function_invocation<'ast>(
    ctx: &mut Context<'ast>,
    invocation: &'ast FunctionInvocation,
) -> Value<'ast> {
    let name = eval_expression(ctx, &invocation.closure_expression);
    let closure = match name {
        Value::Closure(c) => c,
        _ => panic!(
            "could not call, the following expression is not a function {:?}",
            &invocation.closure_expression
        ),
    };

    let mut closure_ctx = Context::new();
    let scope = closure_ctx.scopes.last_mut().unwrap();
    // TODO: TBC if I will allow function overloading, or dynamic parameter lists like JS
    // For now required to be the same
    assert!(
        closure.code.parameters.len() == invocation.parameters.len(),
        "must have the same number of parameters"
    );
    for i in 0..invocation.parameters.len() {
        let expression = &invocation.parameters[i];
        let name = &closure.code.parameters[i].name;
        let val = eval_expression(ctx, expression);
        scope.inner.insert(name.to_owned(), val);
    }

    eval_block(&mut closure_ctx, &closure.code.body)
}

fn eval_literal(literal: &Literal) -> Value {
    match literal {
        Literal::Int(num) => Value::Int(*num),
        Literal::Function(func) => Value::Closure(Closure { code: &func }),
        _ => unimplemented!("Literal type {:?}.", literal),
    }
}

// `let` expressions return a value and so be chained.
// `let x = let y = 42;` will assign both x and y to 42.
fn assign_var<'ast>(ctx: &mut Context<'ast>, expr: &'ast LetExpression) -> Value<'ast> {
    let LetExpression { variable, expr } = expr;

    let value = eval_expression(ctx, expr);

    // `unwrap` is OK because context always has at least one scope object
    let current_scope_object = ctx.scopes.last_mut().unwrap();

    current_scope_object
        .inner
        // TODO: Garbage collection means we won't have to clone
        .insert(variable.name.clone(), value.clone());

    value
}
