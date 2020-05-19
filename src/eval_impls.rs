use super::{
    eval::Eval,
    interpreter::{
        Closure, Effect, FunctionContext, Interpreter, Key, Scope, SubContext, SubContextType,
        Value,
    },
};
use crate::{
    ast,
    eval::{Custom, Location},
    interpreter::symbols,
    interpreter::symbols::error_codes,
};
use std::{collections::HashMap, rc::Rc};

impl Eval for ast::Object {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let self2 = self.clone();

        int.push_eval(Rc::new(Custom::new("ObjectInner", move |int| {
            let mut map = HashMap::new();

            for elem in self2.elems.iter() {

                match elem {
                    ast::ObjectElem::Kv(name, _) => {
                        let value = int.pop_value();
        
                        map.insert(Key::Str(name.clone()), value);
                    }
                    ast::ObjectElem::Spread(_) => {
                        let value = int.pop_value();
                        let value = match value {
                            Value::Object(obj) => obj,
                            _ => panic!("Can only use the ... operator in an object literal on an object."),
                        };

                        map.extend(value.iter().map(|(key, val)| (key.clone(), val.clone())));
                    }
                }
            }

            int.push_value(Value::Object(Rc::new(map)));
        })));

        for elem in self.elems.iter() {
            match elem {
                ast::ObjectElem::Kv(_, expr) => {
                    int.push_eval(expr.clone().into_eval());
                }
                ast::ObjectElem::Spread(expr) => {
                    int.push_eval(CheckTypeObject::new());
                    int.push_eval(expr.clone().into_eval());
                }
            }
        }
    }
    fn short_name(&self) -> &str {
        "Object"
    }
}
impl Eval for ast::ComparisonExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let operator = self.operator;
        int.push_eval(Rc::new(Custom::new("ComparisonInner", move |int| {
            let left = int.pop_value();
            let right = int.pop_value();

            let fail = |operator, left, right| {
                panic!(
                    "Invalid comparison. Cannot apply {:?} to {:?} and {:?}",
                    operator, left, right
                );
            };
            use ast::ComparisonOperator::*;
            use Value::*;
            let full_compare = |operator, left, right| match operator {
                Equal => left == right,
                NotEqual => left != right,
                Less => left < right,
                Greater => left > right,
                LessEqual => left <= right,
                GreaterEqual => left >= right,
            };
            // This code is super long so that I can still take advantage of the Exhaustive Patterns error
            // for Value variants.
            let result = match &(operator, &left, &right) {
                (Equal, Null, Null) => left == right,
                (NotEqual, Null, Null) => left != right,
                (operator, Null, Null) => fail(operator, left, right),

                (Equal, Bool(left), Bool(right)) => left == right,
                (NotEqual, Bool(left), Bool(right)) => left != right,
                (operator, Bool(_), Bool(_)) => fail(operator, left, right),

                (operator, Int(left), Int(right)) => full_compare(*operator, left, right),

                (Equal, Symbol(left), Symbol(right)) => left == right,
                (NotEqual, Symbol(left), Symbol(right)) => left != right,
                (operator, Symbol(_), Symbol(_)) => fail(operator, left, right),

                (Equal, List(left), List(right)) => left == right,
                (NotEqual, List(left), List(right)) => left != right,
                (operator, List(_), List(_)) => fail(operator, left, right),

                (Equal, Object(left), Object(right)) => left == right,
                (NotEqual, Object(left), Object(right)) => left != right,
                (operator, Object(_), List(_)) => fail(operator, left, right),

                (Equal, Closure(left), Closure(right)) => left == right,
                (NotEqual, Closure(left), Closure(right)) => left != right,
                (operator, Closure(_), Closure(_)) => fail(operator, left, right),

                (Equal, Effect(left), Effect(right)) => left == right,
                (NotEqual, Effect(left), Effect(right)) => left != right,
                (operator, Effect(_), Effect(_)) => fail(operator, left, right),

                (Equal, Intrinsic(left), Intrinsic(right)) => left == right,
                (NotEqual, Intrinsic(left), Intrinsic(right)) => left != right,
                (operator, Intrinsic(_), Intrinsic(_)) => fail(operator, left, right),

                // Cover all cases with two different variants.
                (Equal, Null, _) => false,
                (NotEqual, Null, _) => true,
                (operator, Null, _) => fail(operator, left, right),

                (Equal, Bool(_), _) => false,
                (NotEqual, Bool(_), _) => true,
                (operator, Bool(_), _) => fail(operator, left, right),

                (Equal, Int(_), _) => false,
                (NotEqual, Int(_), _) => true,
                (operator, Int(_), _) => fail(operator, left, right),

                (Equal, Symbol(_), _) => false,
                (NotEqual, Symbol(_), _) => true,
                (operator, Symbol(_), _) => fail(operator, left, right),

                (Equal, List(_), _) => false,
                (NotEqual, List(_), _) => true,
                (operator, List(_), _) => fail(operator, left, right),

                (Equal, Object(_), _) => false,
                (NotEqual, Object(_), _) => true,
                (operator, Object(_), _) => fail(operator, left, right),

                (Equal, Closure(_), _) => false,
                (NotEqual, Closure(_), _) => true,
                (operator, Closure(_), _) => fail(operator, left, right),

                (Equal, Effect(_), _) => false,
                (NotEqual, Effect(_), _) => true,
                (operator, Effect(_), _) => fail(operator, left, right),

                (Equal, Intrinsic(_), _) => false,
                (NotEqual, Intrinsic(_), _) => true,
                (operator, Intrinsic(_), _) => fail(operator, left, right),
            };
            int.push_value(Value::Bool(result));
        })));

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

// Infinite looping part of the loop.
#[derive(Debug)]
pub struct LoopBody {
    body: Rc<ast::Block>,
}
impl Eval for LoopBody {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let body = self.body.clone();
        int.push_eval(self); // execute the LoopBody again afterwards (endless loop)
        int.push_eval(Rc::new(Custom::new("IgnoreValue", |int| {
            int.pop_value();
        })));
        int.push_eval(body);
    }
    fn short_name(&self) -> &str {
        "LoopBody"
    }
}

// Establishes a loop sub-context. Runs once and then after every "continue".
#[derive(Debug)]
pub struct LoopContext {
    body: Rc<ast::Block>,
}
impl Eval for LoopContext {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_sub_context(SubContext::new(SubContextType::Loop(self.clone())));
        int.push_eval(Rc::new(LoopBody {
            body: self.body.clone(),
        }));
    }
    fn short_name(&self) -> &str {
        "LoopContext"
    }
}

// Starts a loop. Runs once.
impl Eval for ast::LoopExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(LoopContext {
            body: self.body.clone(),
        }));
    }
    fn short_name(&self) -> &str {
        "Loop"
    }
}

impl Eval for ast::List {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let self2 = self.clone();
        int.push_eval(Rc::new(Custom::new("ListInner", move |int| {
            let mut list = Vec::with_capacity(self2.elements.len());
            for elem in &self2.elements {
                let value = int.pop_value();
                match elem {
                    ast::ListElem::Spread(_) => {
                        let spread_list = match value {
                            Value::List(rc_vec) => rc_vec,
                            _ => panic!(
                                "The ... operator in a list literal can only be applied to a list."
                            ),
                        };
                        list.reserve(spread_list.len());
                        for value in spread_list.iter() {
                            list.push(value.clone());
                        }
                    }
                    ast::ListElem::Elem(_) => {
                        list.push(value.clone());
                    }
                }
            }
            int.push_value(Value::List(Rc::new(list)))
        })));

        for elem in &self.elements {
            let expr = match elem {
                ast::ListElem::Spread(expr) => expr,
                ast::ListElem::Elem(expr) => expr,
            };
            int.push_eval(expr.clone().into_eval());
        }
    }
    fn short_name(&self) -> &str {
        "List"
    }
}
impl Eval for ast::Assignment {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let self2 = self.clone();
        int.push_eval(Rc::new(Custom::new("AssignmentInner", move |int| {
            let value = int.pop_value();

            *int.resolve_location_chain_mut(&self2.location) = value;
        })));

        int.push_eval(self.expr.clone().into_eval());

        for part in self.location.parts.iter() {
            part.push_exprs(int);
        }
    }
    fn short_name(&self) -> &str {
        "Assignment"
    }
}
impl Eval for ast::NotExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(Custom::new("NotInner", |int| {
            let val = int.pop_value();
            let val = match val {
                Value::Bool(b) => b,
                _ => panic!("not operator can only be applied to bools."),
            };
            int.push_value(Value::Bool(!val))
        })));
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
        let value = Value::Closure(Rc::new(Closure::new(self, scope)));
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

impl Eval for ast::ExpressionStatement {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(Custom::new("IgnoreValue", |int| {
            int.pop_value();
        })));
        int.push_eval(self.expr.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "ExpressionStatement"
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

#[derive(Debug)]
struct Error {
    code: u64,
}
impl Error {
    pub fn new(code: u64) -> Rc<Self> {
        Rc::new(Self {
            code,
        })
    }
}
impl Eval for Error {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_value(Value::Object(Rc::new({
            let mut map = HashMap::new();
            map.insert(Key::Str("code".to_string()), Value::Symbol(self.code));
            map
        })));

        int.push_value(Value::Symbol(symbols::ERROR));
        
        int.push_eval(Rc::new(SendInner));
    }
    fn short_name(&self) -> &str {
        "Error"
    }
}

impl Eval for ast::NumericExpression {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let operator = self.operator;
        int.push_eval(Rc::new(Custom::new("NumericInner", move |int| {
            let left = int.pop_value().unwrap_int();
            let right = int.pop_value().unwrap_int();
            use ast::NumericOperator::*;
            let val = match operator {
                Add => Value::Int(left + right),
                Multiply => Value::Int(left * right),
                Subtract => Value::Int(left - right),
                Divide => Value::Int(left / right),
            };
            int.push_value(val)
        })));
        int.push_eval(CheckTypeInt::new());
        int.push_eval(self.left.clone().into_eval());
        int.push_eval(CheckTypeInt::new());
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
        int.push_eval(Rc::new(Custom::new("BooleanInner", move |int| {
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
        })));

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
        int.push_eval(Rc::new(Custom::new("NegativeInner", move |int| {
            let val = int.pop_value().unwrap_int();
            let val = -val;
            int.push_value(Value::Int(val))
        })));
        int.push_eval(CheckIntMin::new());
        int.push_eval(CheckTypeInt::new());
        int.push_eval(self.expr.clone().into_eval());
    }
    fn short_name(&self) -> &str {
        "Negative"
    }
}

impl Eval for String {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let val_ref = match int.current_scope().resolve_binding(self.as_str()) {
            Some(val_ref) => val_ref,
            None => panic!("Could not resolve name {:?}", self.as_str()),
        };
        let value = val_ref.clone();
        int.push_value(value);
    }
    fn short_name(&self) -> &str {
        "Ident"
    }
}

impl Eval for ast::FunctionInvocation {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let num_params_provided = self.parameters.len();
        int.push_eval(Rc::new(Custom::new(
            "FunctionInvocationInner",
            move |int| {
                let closure = int.pop_value();
                let closure = match closure {
                    Value::Closure(closure) => closure,
                    Value::Intrinsic(intrinsic) => {
                        assert!(
                            num_params_provided == intrinsic.num_parameters(),
                            "Must call function with the exact number of params."
                        );

                        int.push_eval(intrinsic.code());

                        return;
                    }
                    _ => panic!("Cannot call value other than closure."),
                };
                let param_names = &closure.code.parameters;

                let body = closure.code.body.clone();
                let scope = Scope::extend(closure.parent_scope.clone());

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
        )));

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

        let Effect { symbol, value, ctx } = Rc::try_unwrap(effect).expect("Couldn't get the context out of an effect. The effect was aliased when it shouldn't have been.");

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
            int.push_eval(Rc::new(ContinueInner));
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

        int.push_value(Value::Effect(Rc::new(Effect { symbol, value, ctx })))
    }
    fn short_name(&self) -> &str {
        "SendInner"
    }
}

impl Eval for ast::SendExpr {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(SendInner));

        int.push_eval(Rc::new(self.symbol.clone()));

        if let Some(expr) = &self.expr {
            int.push_eval(expr.clone().into_eval());
        } else {
            int.push_value(Value::Null);
        }
    }
    fn short_name(&self) -> &str {
        "Send"
    }
}

#[derive(Debug)]
pub struct ContinueInner;
impl Eval for ContinueInner {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let value = int.pop_value();

        // discard current context (either handle match arm or loop iteration) as we do not want to run any more code after the Continue.
        // this wipes the eval_stack and value_stack.
        let SubContext { typ, .. } = int.pop_sub_context();
        match typ {
            SubContextType::Plain => {
                panic!("Cannot use \"continue\" except in a loop or effect handler")
            }
            SubContextType::Handle(handler, ctx) => {
                // re-establish fresh handler
                int.push_eval(handler);

                // switch to the context from the handled continuation.
                int.push_fn_context(*ctx);

                // put value on the value stack (as if it was the result of the "send" that created the continuation)
                int.push_value(value)
            }
            SubContextType::Loop(loop_ctx) => {
                // re-establish fresh loop context
                int.push_eval(loop_ctx);

                // we ignore the value from the continue.
            }
        }
    }
    fn short_name(&self) -> &str {
        "ContinueInner"
    }
}

impl Eval for ast::Continue {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(ContinueInner));

        if let Some(expr) = &self.expr {
            int.push_eval(expr.clone().into_eval());
        } else {
            int.push_value(Value::Null);
        }
    }
    fn short_name(&self) -> &str {
        "Continue"
    }
}

#[derive(Debug)]
pub struct BreakInner;
impl Eval for BreakInner {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let value = int.pop_value();

        // discard current context (either handle match arm or loop iteration) as we do not want to run any more code after the break.
        let SubContext { typ, .. } = int.pop_sub_context();
        match typ {
            SubContextType::Plain => {
                panic!("Cannot use \"break\" except in a loop or effect handler");
            }
            SubContextType::Handle(_handler, _ctx) => {
                // put value on the value stack in the new (outer) subcontext
                int.push_value(value);
            }
            SubContextType::Loop(_loop_expr) => {
                // put value on the value stack in the new (outer) subcontext
                int.push_value(value);
            }
        }
    }
    fn short_name(&self) -> &str {
        "BreakInner"
    }
}

impl Eval for ast::Break {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(BreakInner));

        if let Some(expr) = &self.expr {
            int.push_eval(expr.clone().into_eval());
        } else {
            int.push_value(Value::Null);
        }
    }
    fn short_name(&self) -> &str {
        "Break"
    }
}

#[derive(Debug)]
pub struct WrapperFunction {
    pub body: Rc<dyn ast::Expression>,
}
impl Eval for WrapperFunction {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        int.push_eval(Rc::new(Custom::new("WrapperFunctionInner", |int| {
            let value = int.pop_value();
            int.push_value(value)
        })));

        let self2 = Rc::try_unwrap(self)
            .expect("Implementation error - Couldn't unwrap a WrapperFunction, it is aliased.");
        let scope = Scope::extend(int.current_scope().clone());

        int.push_fn_context(FunctionContext::new(scope));
        int.push_eval(self2.body.into_eval());
    }

    fn short_name(&self) -> &str {
        "WrapperFunction"
    }
}

impl Eval for ast::LocationChain {
    fn eval(self: Rc<Self>, int: &mut Interpreter) {
        let self2 = self.clone();
        int.push_eval(Rc::new(Custom::new("LocationChainInner", move |int| {
            let value = int.resolve_location_chain(&self2);
            int.push_value(value);
        })));

        if let ast::LocationChainBase::Expression(expr) = &self.base {
            int.push_eval(expr.clone().into_eval());
        }

        for part in self.parts.iter() {
            part.push_exprs(int);
        }
    }
    fn short_name(&self) -> &str {
        "LocationChain"
    }
}

impl Location for ast::DotLocation {
    fn push_exprs(&self, _int: &mut Interpreter) {

    }

    fn resolve<'s, 'int>(
        &'s self,
        _pop_value: &mut dyn FnMut() -> Value,
        base: &'int Value,
    ) -> &'int Value {
        let base = match base {
            Value::Object(obj) => obj,
            _ => panic!("Can only use the . operator on objects."),
        };
        match base.get(&Key::Str(self.prop.clone())) {
            Some(val) => val,
            None => panic!("The object does not contain the key {:?}.", self.prop),
        }
    }
    fn resolve_mut<'s, 'int>(
        &'s self,
        _pop_value: &mut dyn FnMut() -> Value,
        base: &'int mut Value,
    ) -> &'int mut Value {
        let base = match base {
            Value::Object(obj) => obj,
            _ => panic!("Can only use the . operator on objects."),
        };
        let base = match Rc::get_mut(base) {
            Some(base) => base,
            None => panic!("Couldn't get the object as mut, it is aliased."),
        };

        match base.get_mut(&Key::Str(self.prop.clone())) {
            Some(val) => val,
            None => panic!("The object does not contain the key {:?}.", self.prop),
        }
    }
}

fn wrap_list_index(len: usize, index: i64) -> usize {
    if index < 0 {
        len - ((-index) as usize)
    } else {
        index as usize
    }
}

impl Location for ast::IndexLocation {
    fn push_exprs(&self, int: &mut Interpreter) {
        int.push_eval(self.index.clone().into_eval());
    }

    fn resolve<'s, 'int>(
        &'s self,
        pop_value: &mut dyn FnMut() -> Value,
        base: &'int Value,
    ) -> &'int Value {
        let base = match base {
            Value::List(obj) => obj,
            _ => panic!("Can only use the . operator on objects."),
        };

        let index = pop_value();
        let index = match index {
            Value::Int(i) => i,
            _ => panic!("Can only index a list with int values."),
        };
        let index = wrap_list_index(base.len(), index);
        match base.get(index) {
            Some(val) => val,
            None => panic!("The index {} is out of range.", index),
        }
    }
    fn resolve_mut<'s, 'int>(
        &'s self,
        pop_value: &mut dyn FnMut() -> Value,
        base: &'int mut Value,
    ) -> &'int mut Value {
        let base = match base {
            Value::List(obj) => obj,
            _ => panic!("Can only use the . operator on objects."),
        };
        let base = match Rc::get_mut(base) {
            Some(base) => base,
            None => panic!("Couldn't get the object as mut, it is aliased."),
        };

        let index = pop_value();
        let index = match index {
            Value::Int(i) => i,
            _ => panic!("Can only index a list with int values."),
        };
        let index = wrap_list_index(base.len(), index);
        match base.get_mut(index) {
            Some(val) => val,
            None => panic!("The index {} is out of range.", index),
        }
    }
}

macro_rules! error_check {
    { $name:ident, $error_code:expr, $check_fn:expr } => {
        #[derive(Debug)]
        struct $name {
            first: bool,
        }
        impl $name {
            fn new() -> Rc<Self> {
                Rc::new(Self { first: true })
            }
        }
        impl Eval for $name {
            fn eval(mut self: Rc<Self>, int: &mut Interpreter) {
                let value = int.pop_value();
                let check_passed = ($check_fn)(&value);
                if check_passed {
                    int.push_value(value);
                } else {
                    match self.first {
                        true => {
                            Rc::get_mut(&mut self).unwrap_or_else(|| panic!("Implementation error - could not do Rc::get_mut on a {}", stringify!($name))).first = false;
                            int.push_eval(self);
                            int.push_eval(Error::new($error_code));
                        }
                        false => {
                            int.push_eval(self);
                            int.push_eval(Error::new(error_codes::ERROR_LOOP));
                        }
                    }
                }
            }
            fn short_name(&self) -> &str {
                stringify!($name)
            }
        }
    }
}

error_check! { CheckTypeInt, error_codes::TYPE_ERROR_INT,
    |value: &Value| match value {
        Value::Int(_) => true,
        _ => false,
    }
}

error_check! { CheckIntMin, error_codes::INT_MIN_NEGATION,
    |value: &Value| value.unwrap_int() != i64::MIN
}

error_check! { CheckTypeObject, error_codes::TYPE_ERROR_OBJECT,
    |value: &Value| match value {
        Value::Object(_) => true,
        _ => false,
    }
}