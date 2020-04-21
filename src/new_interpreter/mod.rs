use crate::ast;
use crate::kal_ref::KalRef;
use std::{cell::Cell, collections::HashMap, rc::Rc};

mod eval;
mod eval_impls;

use ast::Function;
use eval::Eval;

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
    List(KalRef<Vec<Value>>),
    Object(KalRef<HashMap<String, Value>>),
    Closure(KalRef<Closure>),
    Symbol(u64),
}

#[derive(Debug, Clone)]
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
struct SymbolGenerator {
    counter: Cell<u64>,
}

impl SymbolGenerator {
    fn new() -> Self {
        SymbolGenerator {
            counter: Cell::new(0),
        }
    }

    fn gen(&self) -> Value {
        let n = self.counter.get();
        self.counter.set(n + 1);
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
pub struct Context {
    scope_chain: KalRef<Scope>,
    eval_stack: Vec<Rc<dyn Eval>>,
    value_stack: Vec<Value>,
    sym_gen: Rc<SymbolGenerator>,
}

impl Context {
    fn new(sym_gen: Rc<SymbolGenerator>, scope_chain: KalRef<Scope>) -> Self {
        Self {
            scope_chain,
            eval_stack: vec![],
            value_stack: vec![],
            sym_gen,
        }
    }
}

pub struct Interpreter {
    ctx_stack: Vec<Context>,
}

impl Interpreter {
    pub fn new() -> Self {
        let sym_gen = Rc::new(SymbolGenerator::new());
        let scope = KalRef::new(Scope::new());
        Interpreter {
            ctx_stack: vec![Context::new(sym_gen, scope)],
        }
    }

    fn print_eval_stack(&self) {
        println!("===== eval stack =====");
        print!("[ ");
        for eval in self.ctx().eval_stack.iter() {
            print!("{} ", eval.short_name());
        }
        println!("]");
    }

    fn print_value_stack(&self) {
        println!("===== value stack =====");
        println!("{:#?}", self.ctx().value_stack);
    }

    fn print_ctx_stack(&self) {
        println!("===== ctx stack =====");
        println!("{:#?}", self.ctx_stack);
    }

    fn print_scope_chain(&self) {
        println!("===== scope chain =====");
        print!("[ ");
        let mut scope = &self.ctx().scope_chain;
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

    pub fn eval(&mut self, statement: Rc<dyn Eval>) -> Value {
        self.ctx_mut().eval_stack.push(statement);
        loop {
            while self.ctx().eval_stack.len() > 0 {
                let statement = self.ctx_mut().eval_stack.pop().unwrap();

                let result = statement.eval(self);

                result.map(|val| self.ctx_mut().value_stack.push(val));
            }

            self.print_value_stack();
            let value_left_over = self.pop_value();
            self.pop_context();
            if self.ctx_stack.len() > 0 {
                self.ctx_mut().value_stack.push(value_left_over);
            } else {
                return value_left_over;
            }
        }
    }

    fn ctx(&self) -> &Context {
        self.ctx_stack.last().unwrap()
    }

    fn branch_scope(&mut self) -> KalRef<Scope> {
        let scope1 = Scope::extend(self.ctx().scope_chain.clone());
        let scope2 = Scope::extend(self.ctx().scope_chain.clone());
        self.ctx_mut().scope_chain = scope1;
        scope2
    }

    fn ctx_mut(&mut self) -> &mut Context {
        self.ctx_stack
            .last_mut()
            .expect("Not enough values in the context stack.")
    }

    fn push_context(&mut self, ctx: Context) {
        self.ctx_stack.push(ctx);
    }

    fn pop_context(&mut self) {
        self.ctx_stack
            .pop()
            .expect("Implementation error - no more contexts to pop.");
    }

    fn push_eval(&mut self, eval: Rc<dyn Eval>) {
        self.ctx_mut().eval_stack.push(eval)
    }

    fn pop_value(&mut self) -> Value {
        self.ctx_mut()
            .value_stack
            .pop()
            .expect("Implementation error - not enough values on value_stack.")
    }

    fn push_scope(&mut self) {
        let ctx = self.ctx_mut();
        ctx.scope_chain = Scope::extend(ctx.scope_chain.clone())
    }

    fn pop_scope(&mut self) {
        let ctx = self.ctx_mut();
        ctx.scope_chain = ctx
            .scope_chain
            .parent
            .as_ref()
            .expect("Implementation error - no more scopes to pop.")
            .clone();
    }

    fn create_binding(&mut self, name: String, value: Value) {
        // create new scope if the current one has been borrowed? (by a closure)

        self.ctx_mut()
            .scope_chain
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
