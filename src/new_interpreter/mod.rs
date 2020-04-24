use crate::ast;
use crate::kal_ref::KalRef;
use std::{collections::HashMap, rc::Rc};

pub mod eval;
pub mod eval_impls;

use ast::Function;
use eval::Eval;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Null,
    Bool(bool),
    Int(i64),
    Symbol(u64),
    Str(String),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    List(KalRef<Vec<Value>>),
    Object(KalRef<HashMap<Key, Value>>),
    Closure(KalRef<Closure>),
    Symbol(u64),
    Effect(KalRef<Effect>),
}

#[derive(Debug)]
pub struct Closure {
    pub code: Rc<Function>,
    pub scope: KalRef<Scope>,
}

impl Closure {
    pub fn new(code: Rc<Function>, scope: KalRef<Scope>) -> Self {
        Closure { code, scope }
    }
}

impl PartialEq for Closure {
    fn eq(&self, _other: &Closure) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Effect {
    symbol: u64,
    value: Value,
    ctx: FunctionContext,
}

impl PartialEq for Effect {
    fn eq(&self, _other: &Effect) -> bool {
        false
    }
}

#[derive(Debug)]
struct SymbolGenerator {
    counter: u64,
}

impl SymbolGenerator {
    fn new() -> Self {
        SymbolGenerator { counter: 0 }
    }

    fn gen(&mut self) -> Value {
        let n = self.counter;
        self.counter += 1;
        Value::Symbol(n)
    }
}

#[derive(Debug)]
pub struct Scope {
    parent: Option<KalRef<Scope>>,
    bindings: HashMap<String, Value>,
}
impl Scope {
    fn new() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
        }
    }

    fn extend(parent: KalRef<Scope>) -> KalRef<Self> {
        KalRef::new(Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub enum SubContextType {
    Plain,
    Handle(Box<FunctionContext>),
}

#[derive(Debug)]
pub struct SubContext {
    typ: SubContextType,
    eval_stack: Vec<Rc<dyn Eval>>,
    value_stack: Vec<Value>,
}

impl SubContext {
    fn new(typ: SubContextType) -> Self {
        SubContext {
            typ,
            eval_stack: Vec::new(),
            value_stack: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionContext {
    scope: KalRef<Scope>,
    sub_context_stack: Vec<SubContext>,
}

impl FunctionContext {
    fn new(scope: KalRef<Scope>) -> Self {
        Self {
            scope,
            sub_context_stack: vec![SubContext::new(SubContextType::Plain)],
        }
    }
}
pub struct Interpreter {
    sym_gen: SymbolGenerator,
    fn_context_stack: Vec<FunctionContext>,
}

impl Interpreter {
    pub fn new() -> Self {
        let sym_gen = SymbolGenerator::new();
        let scope = KalRef::new(Scope::new());
        Interpreter {
            sym_gen,
            fn_context_stack: vec![FunctionContext::new(scope)],
        }
    }

    #[allow(unused)]
    fn print_eval_stack(&mut self) {
        println!("===== eval stack =====");
        print!("[ ");
        for eval in self.current_sub_context().eval_stack.iter() {
            print!("{} ", eval.short_name());
        }
        println!("]");
    }

    #[allow(unused)]
    fn print_value_stack(&mut self) {
        println!("===== value stack =====");
        println!(
            "{:#?}",
            self.current_fn_context()
                .sub_context_stack
                .last()
                .unwrap()
                .value_stack
        );
    }

    #[allow(unused)]
    fn print_ctx_stack(&self) {
        println!("===== ctx stack =====");
        println!("{:#?}", self.fn_context_stack);
    }

    #[allow(unused)]
    fn print_scope_chain(&mut self) {
        println!("===== scope chain =====");
        print!("[ ");
        let mut scope = &self.current_fn_context().scope;
        loop {
            print!("{{ ");
            for k in scope.bindings.keys() {
                print!("{:?} ", k);
            }
            print!("}} ");

            if let Some(parent) = &scope.parent {
                scope = parent;
            } else {
                break;
            }
        }
        println!("]");
    }

    #[allow(clippy::let_and_return)]
    pub fn eval(&mut self, statement: Rc<dyn Eval>) -> Value {
        self.push_eval(statement);
        let value_left_over = loop {
            // function contexts
            let value_left_over = loop {
                // sub contexts (loop, handle)
                let value_left_over = loop {
                    let statement = self
                        .current_eval_stack()
                        .pop()
                        .expect("Implementation error - no values to pop from the eval stack.");

                    statement.eval(self);

                    if self.current_eval_stack().is_empty() {
                        break self.pop_value();
                    }
                };

                self.pop_sub_context();
                if self.current_fn_context().sub_context_stack.is_empty() {
                    break value_left_over;
                }

                self.push_value(value_left_over);
            };
            self.pop_fn_context();
            if self.fn_context_stack.is_empty() {
                break value_left_over;
            }
            self.push_value(value_left_over);
        };
        value_left_over
    }

    fn branch_scope(&mut self) -> KalRef<Scope> {
        let scope1 = Scope::extend(self.current_fn_context().scope.clone());
        let scope2 = Scope::extend(self.current_fn_context().scope.clone());
        self.current_fn_context().scope = scope1;
        scope2
    }

    fn current_fn_context(&mut self) -> &mut FunctionContext {
        self.fn_context_stack
            .last_mut()
            .expect("Implementation error - no function contexts.")
    }

    fn push_fn_context(&mut self, ctx: FunctionContext) {
        self.fn_context_stack.push(ctx);
    }

    fn pop_fn_context(&mut self) -> FunctionContext {
        self.fn_context_stack
            .pop()
            .expect("Implementation error - no more function contexts to pop.")
    }

    fn current_sub_context(&mut self) -> &mut SubContext {
        self.current_fn_context()
            .sub_context_stack
            .last_mut()
            .expect("Implementation error - no sub contexts.")
    }

    fn push_sub_context(&mut self, ctx: SubContext) {
        self.current_fn_context().sub_context_stack.push(ctx);
    }

    fn pop_sub_context(&mut self) -> SubContext {
        self.current_fn_context()
            .sub_context_stack
            .pop()
            .expect("Implementation error - no more sub contexts to pop.")
    }

    fn current_eval_stack(&mut self) -> &mut Vec<Rc<dyn Eval>> {
        &mut self.current_sub_context().eval_stack
    }

    fn current_value_stack(&mut self) -> &mut Vec<Value> {
        &mut self.current_sub_context().value_stack
    }

    fn push_eval(&mut self, eval: Rc<dyn Eval>) {
        self.current_eval_stack().push(eval)
    }

    fn push_value(&mut self, value: Value) {
        self.current_value_stack().push(value)
    }

    fn pop_value(&mut self) -> Value {
        self.current_value_stack()
            .pop()
            .expect("Implementation error - not enough values on value_stack.")
    }

    fn push_scope(&mut self) {
        let ctx = self.current_fn_context();
        ctx.scope = Scope::extend(ctx.scope.clone())
    }

    fn pop_scope(&mut self) {
        let ctx = self.current_fn_context();
        ctx.scope = ctx
            .scope
            .parent
            .as_ref()
            .expect("Implementation error - no more scopes to pop.")
            .clone();
    }

    fn create_binding(&mut self, name: String, value: Value) {
        // create new scope if the current one has been borrowed? (by a closure)
        self.current_fn_context()
            .scope
            .borrow_mut()
            .unwrap_or_else(|| {
                panic!(
                    "Implementation error - borrow_mut failed create_binding for {:?}.",
                    &name,
                )
            })
            .bindings
            .insert(name, value);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new()
    }
}
