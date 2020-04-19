use crate::ast::BooleanExpression;
use crate::ast::NotExpression;
use crate::ast::{
    Assignment, Block, BooleanOperator, ComparisonExpression, ComparisonOperator, DotExpression,
    Expression, FunctionInvocation, Ident, IfExpression, IfPart, IndexExpression, LetStatement,
    ListLiteral, ListLiteralElem, Literal, Location, NegativeExpression, NumericExpression,
    NumericOperator, ObjectLiteral, Statement,
};

use crate::kal_ref::KalRef;

pub mod types {
    use crate::{ast::Function, kal_ref::KalRef};
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Int(i64),
        List(KalRef<Vec<Value>>),
        Object(KalRef<HashMap<String, Value>>),
        Closure(KalRef<Closure>),
        Symbol(u64),
    }

    pub struct SymbolGenerator {
        counter: u64,
    }

    impl SymbolGenerator {
        pub fn new() -> Self {
            SymbolGenerator { counter: 0 }
        }

        pub fn gen(&mut self) -> Value {
            let n = self.counter;
            self.counter += 1;
            Value::Symbol(n)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Closure {
        pub code: &'static Function,
        pub parent_ctx: KalRef<Context>,
    }

    impl Closure {
        pub fn new(code: &'static Function, parent_ctx: KalRef<Context>) -> Self {
            Closure { code, parent_ctx }
        }
    }

    impl PartialEq for Closure {
        fn eq(&self, _other: &Closure) -> bool {
            false
        }
    }

    #[derive(Debug)]
    pub struct Context {
        parent: Option<KalRef<Context>>,
        scope: HashMap<String, Value>,
    }

    impl Context {
        pub fn new() -> Self {
            Context {
                parent: None,
                scope: HashMap::new(),
            }
        }

        pub fn extend(ctx: KalRef<Self>) -> Self {
            Context {
                parent: Some(ctx),
                scope: HashMap::new(),
            }
        }

        pub fn add_binding(&mut self, k: String, v: Value) {
            self.scope.insert(k, v);
        }

        pub fn resolve_location(&mut self, k: &str) -> &mut Value {
            let mut ctx = self;
            loop {
                if ctx.scope.contains_key(k) {
                    return ctx.scope.get_mut(k).unwrap();
                }
                let parent = ctx.parent.as_mut().unwrap_or_else(|| {
                    panic!("Can't resolve location of {:?}. it doesn't exist.", k)
                });
                let parent_ref = parent.borrow_mut().unwrap_or_else(|| panic!("Couldn't resolve location of {:?}. It may or may not exist but we can't borrow it mutably.", k));
                ctx = parent_ref;
            }
        }

        pub fn resolve_name(&self, k: &str) -> Value {
            let mut ctx = self;
            loop {
                match ctx.scope.get(k) {
                    Some(value) => return value.clone(),
                    None => match &ctx.parent {
                        Some(parent) => ctx = &*parent,
                        None => panic!("Could not resolve name {:?}", k),
                    },
                }
            }
        }
    }
}

use self::types::*;
use std::collections::HashMap;

pub fn eval(ast: &'static Block) -> Value {
    let ctx = KalRef::new(Context::new());
    let mut sym_gen = SymbolGenerator::new();

    eval_block(ctx, &mut sym_gen, ast)
}

fn eval_block(ctx: KalRef<Context>, sym_gen: &mut SymbolGenerator, block: &'static Block) -> Value {
    let mut ctx = KalRef::new(Context::extend(ctx));
    for statement in block.statements.iter() {
        match statement {
            Statement::Let(let_statement) => {
                let LetStatement { variable, expr, .. } = let_statement;
                let value = eval_expression(ctx.clone(), sym_gen, expr);

                // new binding gets added into a new scope if there are multiple references to the current context.
                let ctx_ref = match ctx.borrow_mut() {
                    Some(ctx_ref) => ctx_ref,
                    None => {
                        ctx = KalRef::new(Context::extend(ctx));
                        // unwrap OK because we just created a new context
                        ctx.borrow_mut().unwrap()
                    }
                };
                ctx_ref.add_binding(variable.name.clone(), value);
            }
            Statement::Assignment(assignment) => {
                let Assignment { location, expr } = assignment;

                let value = eval_expression(ctx.clone(), sym_gen, expr);
                let location = match ctx.borrow_mut() {
                    Some(ctx_ref) => eval_location(ctx_ref, location),
                    None => panic!("Can't mutate the following location: {:?}", expr),
                };
                *location = value;
            }
        }
    }
    match block.expression {
        None => Value::Null,
        Some(ref expr) => eval_expression(ctx, sym_gen, expr),
    }
}

fn eval_location<'location>(
    ctx: &'location mut Context,
    location: &'static Location,
) -> &'location mut Value {
    match location {
        Location::Ident(ident) => eval_location_ident(ctx, ident),
    }
}

fn eval_location_ident<'location>(
    ctx: &'location mut Context,
    ident: &'static Ident,
) -> &'location mut Value {
    ctx.resolve_location(&ident.name)
}

fn eval_expression(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    expr: &'static Expression,
) -> Value {
    match expr {
        Expression::Literal(literal) => eval_literal(ctx, sym_gen, literal),
        Expression::FunctionInvocation(func_invo) => {
            eval_function_invocation(ctx, sym_gen, func_invo)
        }
        Expression::Numeric(numeric) => eval_numeric(ctx, sym_gen, numeric),
        Expression::Ident(ident) => eval_ident(ctx, ident),
        Expression::If(if_expr) => eval_if(ctx, sym_gen, if_expr),
        Expression::Comparison(comparison) => eval_comparison(ctx, sym_gen, comparison),
        Expression::Dot(dot_expr) => eval_dot(ctx, sym_gen, dot_expr),
        Expression::Index(index_expr) => eval_index(ctx, sym_gen, index_expr),
        Expression::Boolean(bool_expr) => eval_bool(ctx, sym_gen, bool_expr),
        Expression::Not(not_expr) => eval_not(ctx, sym_gen, not_expr),
        Expression::Negative(neg) => eval_negative(ctx, sym_gen, neg),
    }
}

fn eval_negative(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    neg: &'static NegativeExpression,
) -> Value {
    let NegativeExpression { expr } = neg;

    let val = eval_expression(ctx, sym_gen, expr);
    match val {
        Value::Int(i) => {
            if i == std::i64::MIN {
                // TODO: BigInteger wrapping
                panic!("Can't negate i64::min.");
            }
            Value::Int(-i)
        }
        _ => panic!("Can't apply negation to value other than an int."),
    }
}

fn eval_not(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    not_expr: &'static NotExpression,
) -> Value {
    let NotExpression { expr } = not_expr;

    let val = eval_expression(ctx, sym_gen, expr);
    let val = match val {
        Value::Bool(val) => val,
        _ => panic!(
            "Cant apply the not operator, the operand is not a bool: {:?}.",
            not_expr
        ),
    };

    Value::Bool(!val)
}

fn eval_bool(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    bool_expr: &'static BooleanExpression,
) -> Value {
    let BooleanExpression {
        left,
        right,
        operator,
    } = bool_expr;

    match operator {
        BooleanOperator::And => {
            let left = eval_expression(ctx.clone(), sym_gen, left);
            let left = match left {
                Value::Bool(i) => i,
                _ => panic!("Cant compare, left side not a bool: {:?}.", bool_expr),
            };

            // short-ciKalRefuit
            if !left {
                Value::Bool(false)
            } else {
                let right = eval_expression(ctx, sym_gen, right);
                let right = match right {
                    Value::Bool(i) => i,
                    _ => panic!("Cant compare, right side not a bool: {:?}.", bool_expr),
                };

                Value::Bool(left && right)
            }
        }
        BooleanOperator::Or => {
            let left = eval_expression(ctx.clone(), sym_gen, left);
            let left = match left {
                Value::Bool(i) => i,
                _ => panic!("Cant compare, left side not a bool: {:?}.", bool_expr),
            };

            // short-ciKalRefuit
            if left {
                Value::Bool(true)
            } else {
                let right = eval_expression(ctx, sym_gen, right);
                let right = match right {
                    Value::Bool(i) => i,
                    _ => panic!("Cant compare, right side not a bool: {:?}.", bool_expr),
                };

                Value::Bool(left || right)
            }
        }
        BooleanOperator::Xor => {
            let left = eval_expression(ctx.clone(), sym_gen, left);
            let left = match left {
                Value::Bool(i) => i,
                _ => panic!("Cant compare, left side not a bool: {:?}.", bool_expr),
            };
            let right = eval_expression(ctx, sym_gen, right);
            let right = match right {
                Value::Bool(i) => i,
                _ => panic!("Cant compare, right side not a bool: {:?}.", bool_expr),
            };

            Value::Bool((left && !right) || (!left && right))
        }
    }
}

fn eval_dot(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    dot_expr: &'static DotExpression,
) -> Value {
    let DotExpression { base, prop } = dot_expr;

    let base = eval_expression(ctx, sym_gen, base);

    match base {
        Value::Object(ref obj) => obj
            .get(&prop.name)
            .unwrap_or_else(|| panic!("could not acces property {:?}", &prop.name))
            .clone(),
        _ => panic!(
            "Tried to use dot expression on {:?} from {:?}",
            base, dot_expr
        ),
    }
}

fn eval_index(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    index_expr: &'static IndexExpression,
) -> Value {
    let IndexExpression { base, index } = index_expr;

    let base = eval_expression(ctx.clone(), sym_gen, base);
    let index = eval_expression(ctx, sym_gen, index);

    match base {
        Value::List(ref list) => match index {
            Value::Int(i) => {
                if i >= 0 {
                    let i = i as usize;
                    match list.get(i) {
                        None => panic!(
                            "Index out of range. Index was {:?}, but list has {:?} elements.",
                            i,
                            list.len()
                        ),
                        Some(el) => el.clone(),
                    }
                } else {
                    // negative indices navigate from the end, like python
                    let ui = (-i) as usize;
                    let len = list.len();
                    match list.get(len - ui) {
                        None => panic!(
                            "Index out of range. Index was {:?}, but list has {:?} elements.",
                            i,
                            list.len()
                        ),
                        Some(el) => el.clone(),
                    }
                }
            }
            _ => panic!(
                "Tried to index a list with value other than int. list: {:?}, int: {:?}",
                base, index
            ),
        },
        _ => panic!(
            "Tried to use dot expression on {:?} from {:?}",
            base, index_expr
        ),
    }
}

fn eval_comparison(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    comparison: &'static ComparisonExpression,
) -> Value {
    let ComparisonExpression {
        left,
        right,
        operator,
    } = comparison;
    let left = eval_expression(ctx.clone(), sym_gen, left);
    let right = eval_expression(ctx, sym_gen, right);

    let result = match (operator, &left, &right) {
        (operator, Value::Bool(left), Value::Bool(right)) => compare(operator, left, right),
        (operator, Value::Int(left), Value::Int(right)) => compare(operator, left, right),
        (ComparisonOperator::Equal, Value::Symbol(left), Value::Symbol(right)) => left == right,
        (ComparisonOperator::NotEqual, Value::Symbol(left), Value::Symbol(right)) => left != right,
        (ComparisonOperator::Equal, Value::Null, Value::Null) => true,
        (ComparisonOperator::NotEqual, Value::Null, Value::Null) => false,
        (ComparisonOperator::Equal, Value::List(left), Value::List(right)) => left == right,
        (ComparisonOperator::NotEqual, Value::List(left), Value::List(right)) => left != right,
        (ComparisonOperator::Equal, Value::Object(left), Value::Object(right)) => left == right,
        (ComparisonOperator::NotEqual, Value::Object(left), Value::Object(right)) => left != right,
        (ComparisonOperator::Equal, Value::Closure(_), Value::Closure(_)) => false,
        (ComparisonOperator::NotEqual, Value::Closure(_), Value::Closure(_)) => true,
        _ => panic!(
            "Invalid comparison. Cannot apply {:?} to {:?} and {:?}",
            operator, left, right
        ),
    };

    Value::Bool(result)
}

fn compare<T: std::cmp::Eq + std::cmp::PartialOrd>(
    operator: &ComparisonOperator,
    left: T,
    right: T,
) -> bool {
    match operator {
        ComparisonOperator::Equal => left == right,
        ComparisonOperator::NotEqual => left != right,
        ComparisonOperator::Less => left < right,
        ComparisonOperator::Greater => left > right,
        ComparisonOperator::LessEqual => left <= right,
        ComparisonOperator::GreaterEqual => left >= right,
    }
}

fn eval_if(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    if_expr: &'static IfExpression,
) -> Value {
    let IfExpression { ifs, else_body } = if_expr;

    // evaluate the initial if and any else ifs
    for if_part in ifs.iter() {
        let IfPart { cond, body } = if_part;
        let val = eval_expression(ctx.clone(), sym_gen, &cond);
        let val = match val {
            Value::Bool(val) => val,
            _ => panic!(
                "Conditional evaled to {:?} instead of a boolean: {:?}.",
                val, &cond
            ),
        };

        if val {
            return eval_block(ctx, sym_gen, body);
        }
    }

    // if there is an else block, evaluate it
    if let Some(else_block) = else_body {
        return eval_block(ctx, sym_gen, else_block);
    }

    // if none of the conditions were met, and there is no else block, the if expression evaluates to null.
    Value::Null
}

fn eval_ident(ctx: KalRef<Context>, ident: &Ident) -> Value {
    ctx.resolve_name(&ident.name)
}

fn eval_numeric(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    expr: &'static NumericExpression,
) -> Value {
    let NumericExpression {
        left,
        right,
        operator,
    } = expr;
    let left = eval_expression(ctx.clone(), sym_gen, left);
    let right = eval_expression(ctx, sym_gen, right);
    let left = match left {
        Value::Int(i) => i,
        _ => panic!("Cant add, left side not an Int: {:?}.", expr),
    };
    let right = match right {
        Value::Int(i) => i,
        _ => panic!("Cant add, right side not an Int: {:?}.", expr),
    };

    match operator {
        NumericOperator::Add => Value::Int(left + right),
        NumericOperator::Multiply => Value::Int(left * right),
        NumericOperator::Subtract => Value::Int(left - right),
        NumericOperator::Divide => Value::Int(left / right),
    }
}

fn eval_function_invocation(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    invocation: &'static FunctionInvocation,
) -> Value {
    let val = eval_expression(ctx.clone(), sym_gen, &invocation.closure_expression);

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
                let val = eval_expression(ctx.clone(), sym_gen, expression);
                params.push(val);
            }

            let mut closure_ctx = Context::extend(closure.parent_ctx.clone());
            for (i, val) in params.into_iter().enumerate() {
                let name = &closure.code.parameters[i].name;
                closure_ctx.add_binding(name.to_owned(), val);
            }

            eval_block(KalRef::new(closure_ctx), sym_gen, &closure.code.body)
        }
        _ => panic!(
            "could not call, the following expression is not a function {:?}",
            &invocation.closure_expression
        ),
    }
}

fn eval_literal(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    literal: &'static Literal,
) -> Value {
    match literal {
        Literal::Null => Value::Null,
        Literal::Bool(val) => Value::Bool(*val),
        Literal::Int(num) => Value::Int(*num),
        Literal::Function(func) => Value::Closure(KalRef::new(Closure::new(&func, ctx))),
        Literal::Object(obj) => literal_object(ctx, sym_gen, obj),
        Literal::Symbol => sym_gen.gen(),
        Literal::List(list) => literal_list(ctx, sym_gen, list),
        _ => unimplemented!("Literal type {:?}.", literal),
    }
}

fn literal_list(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    list_literal: &'static ListLiteral,
) -> Value {
    let mut list = Vec::with_capacity(list_literal.elements.len());
    for elem in list_literal.elements.iter() {
        match elem {
            ListLiteralElem::Elem(expr) => {
                let val = eval_expression(ctx.clone(), sym_gen, expr);
                list.push(val);
            }
            ListLiteralElem::Spread(expr) => {
                let val = eval_expression(ctx.clone(), sym_gen, expr);
                match val {
                    Value::List(l) => {
                        list.reserve(l.len());
                        for elem in l.iter() {
                            list.push(elem.clone());
                        }
                    }
                    _ => panic!("Cannot spread {:?} into a list.", val),
                }
            }
        }
    }
    Value::List(KalRef::new(list))
}

fn literal_object(
    ctx: KalRef<Context>,
    sym_gen: &mut SymbolGenerator,
    obj_literal: &'static ObjectLiteral,
) -> Value {
    let mut obj = HashMap::new();
    for (ident, expr) in obj_literal.map.iter() {
        let name = &ident.name;
        let val = eval_expression(ctx.clone(), sym_gen, expr);
        obj.insert(name.to_owned(), val);
    }
    Value::Object(KalRef::new(obj))
}
