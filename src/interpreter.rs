use crate::ast;
use std::{collections::HashMap, rc::Rc};

use crate::eval::Eval;
use crate::{
    eval_impls::{Handler, WrapperFunction, LoopContext},
    intrinsics::{self, Intrinsic},
    core,
};
use ast::{Expression, Function, LocationChain};

pub mod symbols {
    pub const ERROR: u64 = u64::MAX;
    pub mod error_codes {
        const ERROR_CODE_START: u64 = u64::MAX - 10000;
        pub const TYPE_ERROR_INT: u64 = ERROR_CODE_START - 1;
        pub const ERROR_LOOP: u64 = ERROR_CODE_START - 2;
        pub const INT_MIN_NEGATION: u64 = ERROR_CODE_START - 3;

    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Null,
    Bool(bool),
    Int(i64),
    Symbol(u64),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    List(Rc<Vec<Value>>),
    Object(Rc<HashMap<Key, Value>>),
    Closure(Rc<Closure>),
    Symbol(u64),
    Effect(Rc<Effect>),
    Intrinsic(Intrinsic),
}

impl Value {
    pub fn unwrap_object(&self) -> Rc<HashMap<Key, Value>> {
        match self {
            Value::Object(val) => val.clone(),
            _ => panic!("Implementation error - could not unwrap, Value was not an Object."),
        }
    }
    pub fn unwrap_symbol(&self) -> u64 {
        match self {
            Value::Symbol(val) => *val,
            _ => panic!("Implementation error - could not unwrap, Value was not a Symbol."),
        }
    }
    pub fn unwrap_int(&self) -> i64 {
        match self {
            Value::Int(val) => *val,
            _ => panic!("Implementation error - could not unwrap, Value was not an Int."),
        }
    }
}

#[derive(Debug)]
pub struct Closure {
    pub code: Rc<Function>,
    // captured scope immediately outside the closure
    pub parent_scope: Rc<Scope>,
}

impl Closure {
    pub fn new(code: Rc<Function>, scope: Rc<Scope>) -> Self {
        Closure {
            code,
            parent_scope: scope,
        }
    }
}

impl PartialEq for Closure {
    fn eq(&self, _other: &Closure) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Effect {
    pub symbol: u64,
    pub value: Value,
    pub ctx: FunctionContext,
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
    parent: Option<Rc<Scope>>,
    bindings: HashMap<String, Value>,
}
impl Scope {
    pub fn with_bindings(parent: Option<Rc<Scope>>, bindings: HashMap<String, Value>) -> Self {
        Self { parent, bindings }
    }

    pub fn extend(parent: Rc<Scope>) -> Rc<Self> {
        Rc::new(Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        })
    }

    pub fn resolve_binding<'scope>(self: &'scope Rc<Scope>, name: &str) -> Option<&'scope Value> {
        let mut scope = self;
        loop {
            if scope.bindings.contains_key(name) {
                return Some(scope.bindings.get(name).unwrap());
            }
            if let Some(parent) = &scope.parent {
                scope = parent;
            } else {
                // Couldn't go further up the scope chain because we reached the end.
                return None;
            }
        }
    }

    pub fn resolve_binding_mut<'scope>(
        self: &'scope mut Rc<Scope>,
        name: &str,
    ) -> Option<&'scope mut Value> {
        // This will always succeed the first time since the current scope is never aliased.
        let mut scope = Rc::get_mut(self).unwrap_or_else(|| {
            panic!(
                "Implementation error - get_mut failed in resolve_binding_mut for {:?}.",
                &name,
            )
        });
        loop {
            if scope.bindings.contains_key(name) {
                return Some(scope.bindings.get_mut(name).unwrap());
            }

            if let Some(parent) = &mut scope.parent {
                if let Some(parent) = Rc::get_mut(parent) {
                    scope = parent;
                } else {
                    // Couldn't go further up the scope chain because it is branched
                    return None;
                }
            } else {
                // Couldn't go further up the scope chain because we reached the end.
                return None;
            }
        }
    }
}

#[derive(Debug)]
pub enum SubContextType {
    Plain,
    Handle(Rc<Handler>, Box<FunctionContext>),
    Loop(Rc<LoopContext>),
}

#[derive(Debug)]
pub struct SubContext {
    num_scopes: u64,
    pub typ: SubContextType,
    eval_stack: Vec<Rc<dyn Eval>>,
    value_stack: Vec<Value>,
}

impl SubContext {
    pub fn new(typ: SubContextType) -> Self {
        SubContext {
            num_scopes: 0,
            typ,
            eval_stack: Vec::new(),
            value_stack: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionContext {
    scope: Rc<Scope>,
    sub_context_stack: Vec<SubContext>,
}

impl FunctionContext {
    pub fn new(scope: Rc<Scope>) -> Self {
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
        let scope = intrinsics::scope(None);
        let scope = core::scope(Some(scope));
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
    pub fn eval(&mut self, expression: Rc<dyn Expression>) -> Value {
        let wrapper_function = WrapperFunction { body: expression };
        self.push_eval(Rc::new(wrapper_function));
        let value_left_over = loop {
            // function contexts
            let value_left_over = loop {
                // sub contexts (loop, handle)
                let value_left_over = loop {
                    let statement = self
                        .current_eval_stack()
                        .pop()
                        .expect("Implementation error - no more values to pop.");

                    statement.eval(self);

                    if self.current_eval_stack().is_empty() {
                        debug_assert_eq!(
                            self.current_value_stack().len(),
                            1,
                            "There should only be one value left on the stack."
                        );
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

    pub fn branch_scope(&mut self) -> Rc<Scope> {
        let scope1 = Scope::extend(self.current_fn_context().scope.clone());
        let scope2 = Scope::extend(self.current_fn_context().scope.clone());
        self.current_fn_context().scope = scope1;
        scope2
    }

    pub fn current_fn_context(&mut self) -> &mut FunctionContext {
        self.fn_context_stack
            .last_mut()
            .expect("Implementation error - no function contexts.")
    }

    pub fn push_fn_context(&mut self, ctx: FunctionContext) {
        self.fn_context_stack.push(ctx);
    }

    pub fn pop_fn_context(&mut self) -> FunctionContext {
        self.fn_context_stack
            .pop()
            .expect("Implementation error - no more function contexts to pop.")
    }

    pub fn current_sub_context(&mut self) -> &mut SubContext {
        self.current_fn_context()
            .sub_context_stack
            .last_mut()
            .expect("Implementation error - no sub contexts.")
    }

    pub fn push_sub_context(&mut self, ctx: SubContext) {
        self.current_fn_context().sub_context_stack.push(ctx);
    }

    pub fn pop_sub_context(&mut self) -> SubContext {
        // release scopes that were created while the subcontext was active
        // we do this because we can't branch scopes and also have mutability.
        // this means that the subcontexts can't own scopes, but they *can* remember how many they created.
        let num_scopes = self.current_sub_context().num_scopes;
        for _ in 0..num_scopes {
            self.pop_scope();
        }

        self.current_fn_context()
            .sub_context_stack
            .pop()
            .expect("Implementation error - no more sub contexts to pop.")
    }

    pub fn current_eval_stack(&mut self) -> &mut Vec<Rc<dyn Eval>> {
        &mut self.current_sub_context().eval_stack
    }

    pub fn current_value_stack(&mut self) -> &mut Vec<Value> {
        &mut self.current_sub_context().value_stack
    }

    pub fn push_eval(&mut self, eval: Rc<dyn Eval>) {
        self.current_eval_stack().push(eval)
    }

    pub fn push_value(&mut self, value: Value) {
        self.current_value_stack().push(value)
    }

    pub fn pop_value(&mut self) -> Value {
        self.current_value_stack()
            .pop()
            .expect("Implementation error - not enough values on value_stack.")
    }

    pub fn current_scope(&mut self) -> &mut Rc<Scope> {
        &mut self.current_fn_context().scope
    }

    pub fn push_scope(&mut self) {
        let scope = self.current_scope();
        *scope = Scope::extend(scope.clone());
        self.current_sub_context().num_scopes += 1;
    }

    pub fn pop_scope(&mut self) {
        let scope = self.current_scope();
        *scope = scope
            .parent
            .as_ref()
            .expect("Implementation error - no more scopes to pop.")
            .clone();
        self.current_sub_context().num_scopes -= 1;
    }

    pub fn create_binding(&mut self, name: String, value: Value) {
        // This should always succeed because the current scope will never be aliased since we branch it
        // when creating closures.
        Rc::get_mut(&mut self.current_fn_context().scope)
            .unwrap_or_else(|| {
                panic!(
                    "Implementation error - get_mut failed in create_binding for {:?}.",
                    &name,
                )
            })
            .bindings
            .insert(name, value);
    }

    pub fn resolve_location_chain(&mut self, location_chain: &LocationChain) -> Value {
        let fnctx = self.current_fn_context();
        let scope = &fnctx.scope;
        let value_stack = &mut fnctx.sub_context_stack.last_mut().unwrap().value_stack;

        let mut pop_value = || value_stack.pop().unwrap();

        let val = match &location_chain.base {
            ast::LocationChainBase::Ident(ident) => scope.resolve_binding(&ident).unwrap_or_else(|| panic!("Could not resolve name {:?}.", ident)).clone(),
            ast::LocationChainBase::Expression(_) => pop_value(),
        };
        let mut val_ref = &val;
        for part in location_chain.parts.iter() {
            val_ref = part.resolve(&mut pop_value, val_ref);
        }

        val_ref.clone()
    }

    pub fn resolve_location_chain_mut(&mut self, location_chain: &LocationChain) -> &mut Value {
        let fnctx = self.current_fn_context();
        let scope = &mut fnctx.scope;
        let value_stack = &mut fnctx.sub_context_stack.last_mut().unwrap().value_stack;

        let mut pop_value = || value_stack.pop().unwrap();

        let mut val_ref_mut = match &location_chain.base {
            ast::LocationChainBase::Ident(ident) => {
                scope.resolve_binding_mut(&ident).unwrap()
            }
            _ => panic!("Implementation error - grammar should not allow a LocationChainExpression on the left hand side of an assignment."),
        };
        for part in location_chain.parts.iter() {
            val_ref_mut = part.resolve_mut(&mut pop_value, val_ref_mut);
        }

        val_ref_mut
    }

    pub fn gen_symbol(&mut self) -> Value {
        self.sym_gen.gen()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new()
    }
}
