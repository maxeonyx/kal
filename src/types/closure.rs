/*!
`Closure` is kal's native function type.
*/

use crate::{ast::Function, interpreter::Scope};
use std::rc::Rc;

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
