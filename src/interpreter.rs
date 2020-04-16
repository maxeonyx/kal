use crate::ast::{
    AddExpression, Block, Expression, FunctionInvocation, Ident, LetStatement, Literal, Statement,
};

pub mod types {
    use crate::ast::Function;
    use gc::{custom_trace, Finalize, Gc};
    use gc_derive::Trace;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Trace)]
    pub enum Value {
        Null,
        Bool(bool),
        Str(String),
        Int(i64),
        List(Vec<Value>),
        Object(Object),
        Closure(Closure),
    }

    impl Finalize for Value {
        fn finalize(&self) {}
    }

    // TODO: not really a closure yet. Doesn't capture lexical scope
    #[derive(Debug, Clone, Trace)]
    pub struct Closure {
        pub code: &'static Function,
        pub parent_ctx: Gc<Context>,
    }

    impl Finalize for Closure {
        fn finalize(&self) {}
    }

    impl Closure {
        pub fn new(code: &'static Function, parent_ctx: Gc<Context>) -> Self {
            Closure {
                code: code,
                parent_ctx,
            }
        }
    }

    impl PartialEq for Closure {
        fn eq(&self, _other: &Closure) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Object {
        inner: HashMap<String, Value>,
    }

    unsafe impl gc::Trace for Object {
        custom_trace!(this, {
            for val in this.inner.values() {
                mark(val);
            }
        });
    }

    impl Finalize for Object {
        fn finalize(&self) {}
    }

    impl Object {
        pub fn new() -> Self {
            Object {
                inner: HashMap::new(),
            }
        }

        pub fn add_binding(&mut self, k: String, v: Value) {
            self.inner.insert(k, v);
        }

        pub fn remove_binding(&mut self, k: String) {
            self.inner.remove(&k);
        }
    }

    #[derive(Debug, Clone, PartialEq, Trace)]
    pub struct Context {
        parent: Option<Gc<Context>>,
        scope: Object,
    }

    impl Finalize for Context {
        fn finalize(&self) {}
    }

    impl Context {
        pub fn new() -> Self {
            Context {
                parent: None,
                scope: Object::new(),
            }
        }

        pub fn extend(ctx: Gc<Self>) -> Self {
            Context {
                parent: Some(ctx),
                scope: Object::new(),
            }
        }

        pub fn pop_scope(self) {}

        pub fn add_binding(&mut self, k: String, v: Value) {
            self.scope.add_binding(k, v);
        }

        pub fn remove_binding(&mut self, k: String) {
            self.scope.remove_binding(k);
        }

        pub fn current_scope(&self) -> &Object {
            // unwrap ok because we never remove all scopes
            &self.scope
        }

        pub fn resolve_name(&self, name: &str) -> Value {
            let mut ctx = self;
            loop {
                if ctx.current_scope().inner.contains_key(name) {
                    return ctx.current_scope().inner.get(name).unwrap().clone();
                }
                if let Some(ref parent) = ctx.parent {
                    ctx = parent;
                } else {
                    break;
                }
            }
            panic!("Could not resolve name {:?}", name);
        }
    }
}

use self::types::*;
use gc::Gc;

pub fn eval(ast: &'static Block) -> Value {
    let ctx = Gc::new(Context::new());

    eval_block(ctx, ast)
}

fn eval_block(ctx: Gc<Context>, block: &'static Block) -> Value {
    let mut ctx = Context::extend(ctx);
    for statement in block.statements.iter() {
        match statement {
            Statement::Let(let_statement) => {
                let LetStatement { variable, expr } = let_statement;
                let ctx_before_let = Gc::new(ctx);
                let value = eval_expression(ctx_before_let.clone(), expr);

                ctx = Context::extend(ctx_before_let);
                ctx.add_binding(variable.name.clone(), value.clone());
            }
        }
    }

    eval_expression(Gc::new(ctx), &block.expression)
}

fn eval_expression(ctx: Gc<Context>, expr: &'static Expression) -> Value {
    match expr {
        Expression::Literal(literal) => eval_literal(ctx, literal),
        Expression::FunctionInvocation(func_invo) => eval_function_invocation(ctx, func_invo),
        Expression::Add(add_expr) => eval_add(ctx, add_expr),
        Expression::Ident(ident) => eval_ident(ctx, ident),
        _ => unimplemented!("Expression type {:?}.", expr), // TODO
    }
}

fn eval_ident(ctx: Gc<Context>, ident: &Ident) -> Value {
    ctx.resolve_name(&ident.name)
}

fn eval_add(ctx: Gc<Context>, expr: &'static AddExpression) -> Value {
    let AddExpression { left, right } = expr;
    let left = eval_expression(ctx.clone(), left);
    let right = eval_expression(ctx.clone(), right);
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

fn eval_function_invocation(ctx: Gc<Context>, invocation: &'static FunctionInvocation) -> Value {
    let val = eval_expression(ctx.clone(), &invocation.closure_expression);

    match val {
        Value::Closure(ref closure) => {
            // TODO: TBC if I will allow function overloading, or dynamic parameter lists like JS
            // For now required to be the same
            assert!(
                closure.code.parameters.len() == invocation.parameters.len(),
                "must have the same number of parameters"
            );
            let mut params = Vec::with_capacity(invocation.parameters.len());
            for i in 0..invocation.parameters.len() {
                let expression = &invocation.parameters[i];
                let val = eval_expression(ctx.clone(), expression);
                params.push(val);
            }

            let mut closure_ctx = Context::extend(closure.parent_ctx.clone());
            for (i, val) in params.into_iter().enumerate() {
                let name = &closure.code.parameters[i].name;
                closure_ctx.add_binding(name.to_owned(), val);
            }

            eval_block(Gc::new(closure_ctx), &closure.code.body)
        }
        _ => panic!(
            "could not call, the following expression is not a function {:?}",
            &invocation.closure_expression
        ),
    }
}

fn eval_literal(ctx: Gc<Context>, literal: &'static Literal) -> Value {
    match literal {
        Literal::Null => Value::Null,
        Literal::Int(num) => Value::Int(*num),
        Literal::Function(func) => Value::Closure(Closure::new(&func, ctx.clone())),
        _ => unimplemented!("Literal type {:?}.", literal),
    }
}
