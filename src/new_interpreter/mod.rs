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
    pub code: &'static Function,
    pub ctx: KalRef<Context>,
}

impl Closure {
    pub fn new(code: &'static Function, ctx: KalRef<Context>) -> Self {
        Closure { code, ctx }
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
pub struct Context {
    scopes: Vec<HashMap<Key, Value>>,
    eval_stack: Vec<Rc<dyn Eval>>,
    value_stack: Vec<Value>,
    sym_gen: Rc<SymbolGenerator>,
}

impl Context {
    fn new(sym_gen: Rc<SymbolGenerator>) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            eval_stack: vec![],
            value_stack: vec![],
            sym_gen,
        }
    }
}

pub struct Interpreter {
    ctx: Context,
}

impl Interpreter {
    pub fn new() -> Self {
        let sym_gen = Rc::new(SymbolGenerator::new());
        Interpreter {
            ctx: Context::new(sym_gen),
        }
    }

    pub fn eval(&mut self, statement: Rc<dyn Eval>) -> Value {
        self.ctx.eval_stack.push(statement);
        while self.ctx.eval_stack.len() > 0 {
            let statement = self.ctx.eval_stack.pop().unwrap();

            let result = statement.eval(&mut self.ctx);

            result.map(|val| self.ctx.value_stack.push(val));
        }

        debug_assert!(self.ctx.value_stack.len() == 1);
        debug_assert!(self.ctx.eval_stack.len() == 0);

        self.ctx.value_stack.pop().expect("There was no value left on the value stack when execution finished. This is an implementation bug.")
    }
}
