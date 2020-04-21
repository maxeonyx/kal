use super::{eval::Eval, Context, Key, Value};
use crate::ast;
use ast::Literal;
use std::{collections::HashMap, fmt, rc::Rc};

struct Custom<T: Fn(&mut Context) -> Option<Value>> {
    name: &'static str,
    function: T,
}
impl<T: Fn(&mut Context) -> Option<Value>> fmt::Debug for Custom<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Custom")
            .field("name", &self.name)
            .finish()
    }
}
impl<T: Fn(&mut Context) -> Option<Value>> Eval for Custom<T> {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        (self.function)(ctx)
    }
}

impl Eval for ast::Statement {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        use ast::Statement::*;
        match self {
            Let(let_statement) => let_statement.eval(ctx),
            _ => unimplemented!(),
            //Assignment(assignment) => assignment.eval(ctx),
        }
    }
}

impl Eval for ast::Expression {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        use ast::Expression::*;
        match self {
            Literal(literal) => literal.eval(ctx),
            //FunctionInvocation(func_invo) => func_invo.eval(ctx),
            Numeric(numeric) => numeric.eval(ctx),
            Ident(ident) => ident.eval(ctx),
            //If(if_expr) => if_expr.eval(ctx),
            //Comparison(comparison) => comparison.eval(ctx),
            //Dot(dot_expr) => dot_expr.eval(ctx),
            //Index(index_expr) => index_expr.eval(ctx),
            //Boolean(bool_expr) => bool_expr.eval(ctx),
            //Not(not_expr) => not_expr.eval(ctx),
            Negative(neg) => neg.eval(ctx),
            _ => unimplemented!(),
        }
    }
}

impl Eval for ast::Literal {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        use ast::Literal::*;
        match self {
            Null => Some(Value::Null),
            //Bool(val) => Some(Value::Bool(*val)),
            Int(num) => Some(Value::Int(*num)),
            //Symbol => Some(ctx.sym_gen.gen()),
            //Function(func) => func.eval(ctx),
            //Object(obj) => obj.eval(ctx),
            //List(list) => list.eval(ctx),
            _ => unimplemented!(),
        }
    }
}

impl Eval for ast::Block {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        ctx.eval_stack.push(Rc::new(Custom {
            name: "PopScope",
            function: |ctx| {
                ctx.scopes.pop();
                None
            },
        }));

        if let Some(expr) = &self.expression {
            ctx.eval_stack.push(expr.clone());
        } else {
            ctx.eval_stack.push(Rc::new(Literal::Null));
        }

        for statement in self.statements.iter().rev() {
            ctx.eval_stack.push(statement.clone());
        }
        ctx.eval_stack.push(Rc::new(Custom {
            name: "PushScope",
            function: |ctx| {
                ctx.scopes.push(HashMap::new());
                None
            },
        }));
        None
    }
}

impl Eval for ast::LetStatement {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        let name = self.variable.name.clone();
        ctx.eval_stack.push(Rc::new(Custom {
            name: "Let",
            function: move |ctx| {
                let val = ctx
                    .value_stack
                    .pop()
                    .expect("Implementation error - Not enough values for Let");
                ctx.scopes
                    .last_mut()
                    .unwrap()
                    .insert(Key::Str(name.clone()), val);
                None
            },
        }));
        ctx.eval_stack.push(self.expr.clone());
        None
    }
}

impl Eval for ast::NumericExpression {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        let operator = self.operator;
        ctx.eval_stack.push(Rc::new(Custom {
            name: "Numeric",
            function: move |ctx| {
                let left = ctx
                    .value_stack
                    .pop()
                    .expect("Implementation error - Not enough values for Numeric");
                let right = ctx
                    .value_stack
                    .pop()
                    .expect("Implementation error - Not enough values for Numeric");

                let left = match left {
                    Value::Int(i) => i,
                    _ => panic!("Cant add, left side not an Int."),
                };
                let right = match right {
                    Value::Int(i) => i,
                    _ => panic!("Cant add, right side not an Int."),
                };
                use ast::NumericOperator::*;
                let val = match operator {
                    Add => Value::Int(left + right),
                    Multiply => Value::Int(left * right),
                    Subtract => Value::Int(left - right),
                    Divide => Value::Int(left / right),
                };
                Some(val)
            },
        }));
        ctx.eval_stack.push(self.left.clone());
        ctx.eval_stack.push(self.right.clone());
        None
    }
}

impl Eval for ast::NegativeExpression {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        ctx.eval_stack.push(Rc::new(Custom {
            name: "Negative",
            function: move |ctx| {
                let val = ctx
                    .value_stack
                    .pop()
                    .expect("Implementation error - Not enough values for Negative");
                let val = match val {
                    Value::Int(i) => i,
                    _ => panic!("Cant negate, val not an Int."),
                };
                if val == std::i64::MIN {
                    // TODO: BigInteger wrapping
                    panic!("Can't negate i64::min.");
                }
                let val = -val;
                Some(Value::Int(val))
            },
        }));
        ctx.eval_stack.push(self.expr.clone());
        None
    }
}

impl Eval for ast::Ident {
    fn eval(&self, ctx: &mut Context) -> Option<Value> {
        for scope in ctx.scopes.iter().rev() {
            if let Some(value) = scope.get(&Key::Str(self.name.clone())) {
                return Some(value.clone());
            }
        }
        panic!("Could not resolve name {:?}", self.name)
    }
}
