use super::{eval::Eval, Context, Interpreter, Scope, Value};
use crate::ast;
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
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        (self.function)(int)
    }
    fn short_name(&self) -> &str {
        self.name
    }
}

impl Eval for ast::Null {
    fn eval(&self, _: &mut Interpreter) -> Option<Value> {
        Some(Value::Null)
    }
    fn short_name(&self) -> &str {
        "LiteralNull"
    }
}

impl Eval for ast::Symbol {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        Some(int.sym_gen.gen())
    }
    fn short_name(&self) -> &str {
        "Symbol"
    }
}

impl Eval for ast::Bool {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        Some(Value::Bool(self.0))
    }
    fn short_name(&self) -> &str {
        "Bool"
    }
}

impl Eval for ast::Int {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        Some(Value::Int(self.0))
    }
    fn short_name(&self) -> &str {
        "Int"
    }
}

impl Eval for ast::Block {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        int.push_eval(Rc::new(Custom {
            name: "PopScope",
            function: |int| {
                int.pop_scope();
                None
            },
        }));

        if let Some(expr) = &self.expression {
            int.push_eval(expr.into_eval());
        } else {
            int.push_eval(Rc::new(ast::Null));
        }

        for statement in self.statements.iter().rev() {
            int.push_eval(statement.into_eval());
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
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        let name = self.ident.clone();
        int.push_eval(Rc::new(Custom {
            name: "LetInner",
            function: |int| {
                let val = int.pop_value();
                int.create_binding(name.clone(), val);
                None
            },
        }));
        int.push_eval(self.expr.into_eval());
        None
    }
    fn short_name(&self) -> &str {
        "Let"
    }
}

impl Eval for ast::NumericExpression {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
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
        int.push_eval(self.left.into_eval());
        int.push_eval(self.right.into_eval());
        None
    }
    fn short_name(&self) -> &str {
        "Numeric"
    }
}

impl Eval for ast::NegativeExpression {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
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
        int.push_eval(self.expr.into_eval());
        None
    }
    fn short_name(&self) -> &str {
        "Negative"
    }
}

impl Eval for String {
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        let mut scope = &int.ctx().scope_chain;
        loop {
            if let Some(value) = scope.bindings.get(self) {
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
    fn eval(&self, int: &mut Interpreter) -> Option<Value> {
        let num_params_provided = self.parameters.len();
        int.push_eval(Rc::new(Custom {
            name: "FunctionInvocationInner",
            function: move |int| {
                let closure = int.pop_value();
                let closure = match closure {
                    Value::Closure(closure) => closure,
                    _ => panic!("Cannot call value other than closure."),
                };
                let param_names = closure.code.parameters;

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

                for (name, value) in param_names.into_iter().zip(values) {
                    int.create_binding(name, value);
                }

                int.push_eval(body);

                None
            },
        }));

        int.push_eval(self.base.into_eval());

        for expr in self.parameters.iter() {
            int.push_eval(expr.into_eval());
        }

        None
    }
    fn short_name(&self) -> &str {
        "FunctionInvocation"
    }
}
