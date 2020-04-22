use super::{
    eval::{Eval, UnimplementedEval},
    Closure, Context, Interpreter, Scope, Value,
};
use crate::{ast, kal_ref::KalRef};
use std::{fmt, rc::Rc};

struct Custom<T: Fn(&mut Interpreter) -> Option<Value>> {
    name: &'static str,
    function: T,
}
impl<T: Fn(&mut Interpreter) -> Option<Value>> fmt::Debug for Custom<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Custom")
            .field("name", &self.name)
            .finish()
    }
}
impl<T: Fn(&mut Interpreter) -> Option<Value>> Eval for Custom<T> {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
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
impl UnimplementedEval for ast::SendExpr {
    fn short_name(&self) -> &str {
        "Send"
    }
}
impl UnimplementedEval for ast::Handle {
    fn short_name(&self) -> &str {
        "Handle"
    }
}
impl UnimplementedEval for ast::ComparisonExpression {
    fn short_name(&self) -> &str {
        "Comparison"
    }
}
impl UnimplementedEval for ast::IfExpression {
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
impl UnimplementedEval for ast::NotExpression {
    fn short_name(&self) -> &str {
        "Not"
    }
}

impl Eval for ast::Null {
    fn eval(self: Rc<Self>, _: &mut Interpreter) -> Option<Value> {
        Some(Value::Null)
    }
    fn short_name(&self) -> &str {
        "LiteralNull"
    }
}

impl Eval for ast::Symbol {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
        Some(int.sym_gen.gen())
    }
    fn short_name(&self) -> &str {
        "Symbol"
    }
}

impl Eval for ast::Bool {
    fn eval(self: Rc<Self>, _: &mut Interpreter) -> Option<Value> {
        Some(Value::Bool(self.0))
    }
    fn short_name(&self) -> &str {
        "Bool"
    }
}

impl Eval for ast::Int {
    fn eval(self: Rc<Self>, _: &mut Interpreter) -> Option<Value> {
        Some(Value::Int(self.0))
    }
    fn short_name(&self) -> &str {
        "Int"
    }
}

impl Eval for ast::Function {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
        Some(Value::Closure(KalRef::new(Closure::new(
            self,
            int.branch_scope(),
        ))))
    }
    fn short_name(&self) -> &str {
        "Function"
    }
}

impl Eval for ast::Block {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
        int.push_eval(Rc::new(Custom {
            name: "PopScope",
            function: |int| {
                int.pop_scope();
                None
            },
        }));

        if let Some(expr) = self.expression.as_ref() {
            int.push_eval(expr.clone().into_eval());
        } else {
            int.push_eval(Rc::new(ast::Null));
        }

        for statement in self.statements.iter().rev() {
            int.push_eval(statement.clone().into_eval());
        }
        int.push_eval(Rc::new(Custom {
            name: "PushScope",
            function: |int| {
                int.push_scope();
                None
            },
        }));
        None
    }
    fn short_name(&self) -> &str {
        "Block"
    }
}

impl Eval for ast::LetStatement {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
        let name = self.ident.clone();
        int.push_eval(Rc::new(Custom {
            name: "LetInner",
            function: move |int| {
                let val = int.pop_value();
                int.create_binding(name.clone(), val);
                None
            },
        }));
        int.push_eval(self.expr.clone().into_eval());
        None
    }
    fn short_name(&self) -> &str {
        "Let"
    }
}

impl Eval for ast::NumericExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
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
                Some(val)
            },
        }));
        int.push_eval(self.left.clone().into_eval());
        int.push_eval(self.right.clone().into_eval());
        None
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
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
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
                Some(val)
            },
        }));

        // no short-circuiting at the moment
        int.push_eval(self.left.clone().into_eval());
        int.push_eval(self.right.clone().into_eval());
        None
    }
    fn short_name(&self) -> &str {
        "Boolean"
    }
}

impl Eval for ast::NegativeExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
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
                Some(Value::Int(val))
            },
        }));
        int.push_eval(self.expr.clone().into_eval());
        None
    }
    fn short_name(&self) -> &str {
        "Negative"
    }
}

impl Eval for String {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
        let mut scope = &int.ctx().scope_chain;
        loop {
            if let Some(value) = scope.bindings.get(&*self) {
                return Some(value.clone());
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
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value> {
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

                int.push_context(Context::new(scope));

                for (name, value) in param_names.iter().zip(values) {
                    int.create_binding(name.clone(), value);
                }

                int.push_eval(body);

                None
            },
        }));

        int.push_eval(self.base.clone().into_eval());

        for expr in self.parameters.iter() {
            int.push_eval(expr.clone().into_eval());
        }

        None
    }
    fn short_name(&self) -> &str {
        "FunctionInvocation"
    }
}
