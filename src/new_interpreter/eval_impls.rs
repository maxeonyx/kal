use super::{
    eval::{Eval, UnimplementedEval},
    Closure, Effect, FunctionContext, Interpreter, Scope, SubContext, SubContextType, Value,
};
use crate::{ast, kal_ref::KalRef};
use std::{fmt, rc::Rc};

struct Custom<T: Fn(&mut Interpreter)> {
    name: &'static str,
    function: T,
}
impl<T: Fn(&mut Interpreter)> fmt::Debug for Custom<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Custom")
            .field("name", &self.name)
            .finish()
    }
}
impl<T: Fn(&mut Interpreter)> Eval for Custom<T> {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        (self.function)(int)
    }
    fn short_name(&self) -> &str {
        self.name
    }
}

impl UnimplementedEval for ast::DotExpression {
    fn short_name(&self) -> &str {
        "Dot"
    }
}
impl UnimplementedEval for ast::IndexExpression {
    fn short_name(&self) -> &str {
        "Index"
    }
}
impl Eval for ast::ComparisonExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let operator = self.operator;
        int.push_eval(Rc::new(Custom {
            name: "ComparisonInner",
            function: move |int| {
                let left = int.pop_value();
                let right = int.pop_value();

                use super::Value::*;
                use ast::ComparisonOperator;
                use ast::ComparisonOperator::*;

                fn full_compare<T: std::cmp::Eq + std::cmp::PartialOrd>(
                    operator: &ComparisonOperator,
                    left: T,
                    right: T,
                ) -> bool {
                    match operator {
                        Equal => left == right,
                        NotEqual => left != right,
                        Less => left < right,
                        Greater => left > right,
                        LessEqual => left <= right,
                        GreaterEqual => left >= right,
                    }
                }
                fn eq_compare<T: std::cmp::PartialEq + std::fmt::Debug>(
                    operator: &ComparisonOperator,
                    left: T,
                    right: T,
                ) -> bool {
                    match operator {
                        Equal => left == right,
                        NotEqual => left != right,
                        _ => panic!(
                            "Invalid comparison. Cannot apply {:?} to {:?} and {:?}",
                            operator, left, right
                        ),
                    }
                }
                let result = match &(operator, &left, &right) {
                    (operator, Int(left), Int(right)) => full_compare(operator, left, right),
                    (operator, Bool(left), Bool(right)) => eq_compare(operator, left, right),
                    (operator, Symbol(left), Symbol(right)) => eq_compare(operator, left, right),
                    (operator, List(left), List(right)) => eq_compare(operator, left, right),
                    (operator, Object(left), Object(right)) => eq_compare(operator, left, right),
                    (operator, Closure(left), Closure(right)) => eq_compare(operator, left, right),
                    (operator, Null, Null) => eq_compare(operator, left, right),
                    _ => panic!(
                        "Invalid comparison. Cannot apply {:?} to {:?} and {:?}",
                        operator, left, right
                    ),
                };
                int.push_value(Value::Bool(result));
            },
        }));

        int.push_eval(self.left.clone().into_eval());
        int.push_eval(self.right.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "Comparison"
    }
}

#[derive(Debug)]
pub struct IfInner {
    index: usize,
    ifs: Vec<ast::IfPart>,
    else_body: Option<Rc<ast::Block>>,
}
impl Eval for IfInner {
    fn eval(mut self: Rc<Self>, int: &mut Interpreter) {
        let value = int.pop_value();
        let value = match value {
            Value::Bool(b) => b,
            _ => panic!("If condition value must be a bool."),
        };

        let if_part = self.ifs.get(self.index).unwrap();
        if value {
            int.push_eval(if_part.body.clone());
        } else if self.index < self.ifs.len() - 1 {
            Rc::get_mut(&mut self)
                .expect("Implementation error - can't get IfInner as mut, it is aliased")
                .index += 1;
            let cond_expr = self.ifs.get(self.index).unwrap().cond.clone().into_eval();
            int.push_eval(self);
            int.push_eval(cond_expr);
        } else if let Some(else_body) = self.else_body.as_ref() {
            int.push_eval(else_body.clone())
        } else {
            int.push_value(Value::Null);
        }
    }
    fn short_name(&self) -> &str {
        "IfInner"
    }
}

impl Eval for ast::IfExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(IfInner {
            index: 0,
            ifs: self.ifs.clone(),
            else_body: self.else_body.clone(),
        }));

        int.push_eval(self.ifs.get(0).unwrap().cond.clone().into_eval())
    }
    fn short_name(&self) -> &str {
        "If"
    }
}
impl UnimplementedEval for ast::Object {
    fn short_name(&self) -> &str {
        "Object"
    }
}
impl UnimplementedEval for ast::List {
    fn short_name(&self) -> &str {
        "List"
    }
}
impl UnimplementedEval for ast::Assignment {
    fn short_name(&self) -> &str {
        "Assignment"
    }
}
impl Eval for ast::NotExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(Custom {
            name: "NotInner",
            function: |int| {
                let val = int.pop_value();
                let val = match val {
                    Value::Bool(b) => b,
                    _ => panic!("not operator can only be applied to bools."),
                };
                int.push_value(Value::Bool(!val))
            },
        }));
        int.push_eval(self.expr.clone().into_eval())
    }
    fn short_name(&self) -> &str {
        "Not"
    }
}

impl Eval for ast::Null {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_value(Value::Null)
    }
    fn short_name(&self) -> &str {
        "LiteralNull"
    }
}

impl Eval for ast::Symbol {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let symbol = int.sym_gen.gen();
        int.push_value(symbol);
    }
    fn short_name(&self) -> &str {
        "Symbol"
    }
}

impl Eval for ast::Bool {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_value(Value::Bool(self.0))
    }
    fn short_name(&self) -> &str {
        "Bool"
    }
}

impl Eval for ast::Int {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_value(Value::Int(self.0))
    }
    fn short_name(&self) -> &str {
        "Int"
    }
}

impl Eval for ast::Function {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let scope = int.branch_scope();
        let value = Value::Closure(KalRef::new(Closure::new(self, scope)));
        int.push_value(value);
    }
    fn short_name(&self) -> &str {
        "Function"
    }
}

#[derive(Debug)]
pub struct PopScope;
impl Eval for PopScope {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.pop_scope();
    }
    fn short_name(&self) -> &str {
        "PopScope"
    }
}

#[derive(Debug)]
pub struct PushScope;
impl Eval for PushScope {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_scope();
    }
    fn short_name(&self) -> &str {
        "PushScope"
    }
}

impl Eval for ast::Block {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(PopScope));

        if let Some(expr) = self.expression.as_ref() {
            int.push_eval(expr.clone().into_eval());
        } else {
            int.push_eval(Rc::new(ast::Null));
        }

        for statement in self.statements.iter().rev() {
            int.push_eval(statement.clone().into_eval());
        }
        int.push_eval(Rc::new(PushScope));
    }
    fn short_name(&self) -> &str {
        "Block"
    }
}

#[derive(Debug)]
pub struct LetInner {
    name: String,
}
impl Eval for LetInner {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let val = int.pop_value();
        int.create_binding(self.name.clone(), val);
    }
    fn short_name(&self) -> &str {
        "LetInner"
    }
}

impl Eval for ast::LetStatement {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let name = self.ident.clone();
        int.push_eval(Rc::new(LetInner { name }));
        int.push_eval(self.expr.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "Let"
    }
}

impl Eval for ast::NumericExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let operator = self.operator;
        int.push_eval(Rc::new(Custom {
            name: "NumericInner",
            function: move |int| {
                let left = int.pop_value();
                let right = int.pop_value();

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
                int.push_value(val)
            },
        }));
        int.push_eval(self.left.clone().into_eval());
        int.push_eval(self.right.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        use ast::NumericOperator::*;
        match self.operator {
            Add => "Add",
            Multiply => "Multiply",
            Subtract => "Subtract",
            Divide => "Divide",
        }
    }
}

impl Eval for ast::BooleanExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let operator = self.operator;
        int.push_eval(Rc::new(Custom {
            name: "BooleanInner",
            function: move |int| {
                let left = int.pop_value();
                let right = int.pop_value();

                let left = match left {
                    Value::Bool(i) => i,
                    _ => panic!("Cant compare, left side not a bool."),
                };
                let right = match right {
                    Value::Bool(i) => i,
                    _ => panic!("Cant compare, right side not a bool."),
                };
                use ast::BooleanOperator::*;
                let val = match operator {
                    And => Value::Bool(left && right),
                    Or => Value::Bool(left || right),
                    Xor => Value::Bool((!left && right) || (!right && left)),
                };
                int.push_value(val)
            },
        }));

        // no short-circuiting at the moment
        int.push_eval(self.left.clone().into_eval());
        int.push_eval(self.right.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "Boolean"
    }
}

impl Eval for ast::NegativeExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(Custom {
            name: "NegativeInner",
            function: move |int| {
                let val = int.pop_value();
                let val = match val {
                    Value::Int(i) => i,
                    _ => panic!("Cant negate, val not an Int."),
                };
                if val == std::i64::MIN {
                    // TODO: BigInteger wrapping
                    panic!("Can't negate i64::min.");
                }
                let val = -val;
                int.push_value(Value::Int(val))
            },
        }));
        int.push_eval(self.expr.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "Negative"
    }
}

impl Eval for String {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let mut scope = &int.current_fn_context().scope.clone();
        loop {
            if let Some(value) = scope.bindings.get(&*self) {
                int.push_value(value.clone());
                return;
            }

            if let Some(parent) = &scope.parent {
                scope = parent;
                continue;
            }

            panic!("Could not resolve name {:?}", self);
        }
    }
    fn short_name(&self) -> &str {
        "Ident"
    }
}

impl Eval for ast::FunctionInvocation {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let num_params_provided = self.parameters.len();
        int.push_eval(Rc::new(Custom {
            name: "FunctionInvocationInner",
            function: move |int| {
                let closure = int.pop_value();
                let closure = match closure {
                    Value::Closure(closure) => closure,
                    _ => panic!("Cannot call value other than closure."),
                };
                let param_names = &closure.code.parameters;

                let body = closure.code.body.clone();
                let scope = Scope::extend(closure.scope.clone());

                // param lists of different length
                assert!(
                    num_params_provided == param_names.len(),
                    "Must call function with the exact number of params."
                );

                let mut values = Vec::with_capacity(num_params_provided);
                for _ in 0..num_params_provided {
                    values.push(int.pop_value());
                }

                int.push_fn_context(FunctionContext::new(scope));

                for (name, value) in param_names.iter().zip(values) {
                    int.create_binding(name.clone(), value);
                }

                int.push_eval(body);
            },
        }));

        int.push_eval(self.base.clone().into_eval());

        for expr in self.parameters.iter() {
            int.push_eval(expr.clone().into_eval());
        }
    }
    fn short_name(&self) -> &str {
        "FunctionInvocation"
    }
}
#[derive(Debug)]
pub struct Handler {
    match_arms: Vec<(u64, ast::HandleMatch)>,
}
impl Eval for Handler {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let effect = int.pop_value();
        let effect = match effect {
            Value::Effect(e) => e,
            // if function returned normally, handle evaluates to that value.
            _ => {
                int.push_value(effect);
                return;
            }
        };

        let Effect { symbol, value, ctx } = effect.try_into_inner().expect("Couldn't get the context out of an effect. The effect was aliased when it shouldn't have been.");

        int.push_sub_context(SubContext::new(SubContextType::Handle(
            self.clone(),
            Box::new(ctx),
        )));

        let match_arm = self
            .match_arms
            .clone()
            .into_iter()
            .find(|(sym, _)| *sym == symbol);

        if let Some((_, ast::HandleMatch { param, block, .. })) = match_arm {
            int.push_eval(Rc::new(PopScope));
            int.push_eval(block);
            int.push_eval(Rc::new(LetInner { name: param }));
            // if PushScope added/consumed values, or changed the context, we would have to push an identity function here instead of value directly.
            int.push_value(value);
            int.push_eval(Rc::new(PushScope));
        } else {
            // if there is no match arm that handles this effect, establish a passthrough.
            // this means sending the effect upwards, then resuming with whatever value we get back
            int.push_eval(Rc::new(ResumeInner));
            int.push_eval(Rc::new(SendInner));
            int.push_value(value);
            int.push_value(Value::Symbol(symbol));
        }
    }
    fn short_name(&self) -> &str {
        "Handler"
    }
}

#[derive(Debug)]
struct CreateHandler {
    match_arms: Vec<ast::HandleMatch>,
    expr: Rc<dyn ast::Expression>,
}
impl Eval for CreateHandler {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let self2 = Rc::try_unwrap(self).expect(
            "Implementation error - can't unwrap CreateHandler. I will need to clone some stuff.",
        );
        let mut symbols = Vec::with_capacity(self2.match_arms.len());
        for _ in 0..self2.match_arms.len() {
            let symbol = int.pop_value();
            let symbol = match symbol {
                Value::Symbol(symbol) => symbol,
                _ => panic!("Effect type in match arm must be a symbol."),
            };
            symbols.push(symbol);
        }

        int.push_eval(Rc::new(Handler {
            match_arms: symbols
                .into_iter()
                .zip(self2.match_arms.into_iter())
                .collect::<Vec<_>>(),
        }));

        int.push_eval(self2.expr.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "CreateHandler"
    }
}

impl Eval for ast::Handle {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let match_arms = self.match_arms.clone();
        let expr = self.expr.clone();
        int.push_eval(Rc::new(CreateHandler { match_arms, expr }));

        // eagerly evaluate the symbols
        for match_arm in &self.match_arms {
            int.push_eval(Rc::new(match_arm.symbol.clone()));
        }
    }
    fn short_name(&self) -> &str {
        "Handle"
    }
}

#[derive(Debug)]
pub struct SendInner;
impl Eval for SendInner {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let symbol = int.pop_value();
        let symbol = match symbol {
            Value::Symbol(symbol) => symbol,
            _ => panic!("Effect type in send must be a symbol."),
        };
        let value = int.pop_value();

        let ctx = int.pop_fn_context();

        int.push_value(Value::Effect(KalRef::new(Effect { symbol, value, ctx })))
    }
    fn short_name(&self) -> &str {
        "SendInner"
    }
}

impl Eval for ast::SendExpr {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(SendInner));

        int.push_eval(Rc::new(self.symbol.clone()));

        int.push_eval(self.expr.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "Send"
    }
}

#[derive(Debug)]
pub struct ResumeInner;
impl Eval for ResumeInner {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let value = int.pop_value();

        // discard current context (either handle match arm or loop iteration) as we do not want to run any more code after the resume.
        let SubContext { typ, .. } = int.pop_sub_context();
        match typ {
            SubContextType::Plain => {
                panic!("Cannot use \"resume\" except in a loop or effect handler")
            }
            SubContextType::Handle(handler, ctx) => {
                // re-establish handler
                int.push_eval(handler);

                // switch to the context from the handled continuation.
                int.push_fn_context(*ctx);

                // put value on the value stack (as if it was the result of the "send" that created the continuation)
                int.push_value(value)
            }
        }
    }
    fn short_name(&self) -> &str {
        "ResumeInner"
    }
}

impl Eval for ast::Resume {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(ResumeInner));

        int.push_eval(self.expr.clone().into_eval())
    }
    fn short_name(&self) -> &str {
        "Resume"
    }
}

#[derive(Debug)]
pub struct WrapperFunction {
    pub body: Rc<dyn ast::Expression>,
}
impl Eval for WrapperFunction {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(Custom {
            name: "WrapperFunctionInner",
            function: |int| {
                let value = int.pop_value();
                int.push_value(value)
            },
        }));

        let self2 = Rc::try_unwrap(self)
            .expect("Implementation error - Couldn't unwrap a WrapperFunction, it is aliased.");
        let scope = Scope::extend(int.current_fn_context().scope.clone());

        int.push_fn_context(FunctionContext::new(scope));
        int.push_eval(self2.body.into_eval());
    }

    fn short_name(&self) -> &str {
        "WrapperFunction"
    }
}
