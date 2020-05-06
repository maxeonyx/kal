mod closure;
mod effect;
mod list;
mod object;
mod ref_mut;

pub use closure::*;
pub use effect::*;
pub use list::*;
pub use object::*;
pub use ref_mut::*;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Symbol(u64),
    Mut(Mut<Value>),
    List(List),
    ListMut(ListMut),
    Object(Object),
    ObjectMut(ObjectMut),
    Closure(Rc<Closure>),
    Effect(Box<Effect>),
}

impl Value {
    pub fn is_mut(&self) -> bool {
        use Value::*;
        match self {
            Null => false,
            Bool(_) => false,
            Int(_) => false,
            Symbol(_) => false,
            List(_) => false,
            Object(_) => false,
            Closure(_) => false,

            Mut(_) => true,
            ListMut(_) => true,
            ObjectMut(_) => true,
            Effect(_) => true,
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        use Value::*;
        match self {
            Null => Null,
            Bool(val) => Bool(*val),
            Int(val) => Int(*val),
            Symbol(val) => Symbol(*val),
            Mut(_) => panic!("Implementation error - tried to clone a mut ref."),
            List(val) => List(val.clone()),
            ListMut(_) => panic!("Implementation error - tried to clone a mut list."),
            Object(val) => Object(val.clone()),
            ObjectMut(_) => panic!("Implementation error - tried to clone a mut object."),
            Closure(val) => Closure(val.clone()),
            Effect(_) => panic!("Implementation error - tried to clone an effect."),
        }
    }
}
