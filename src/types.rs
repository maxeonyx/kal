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
    Ref(Ref<Value>),
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

            Ref(_) => false,
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
        let val = match self {
            Null => Some(Null),
            Bool(val) => Some(Bool(*val)),
            Int(val) => Some(Int(*val)),
            Symbol(val) => Some(Symbol(*val)),

            // Closure is a reference type but can never be mutable, cloning always succeeds.
            Closure(val) => Some(Closure(val.clone())),

            // For other reference types cloning might fail with a runtime error.
            Ref(val) => val.try_clone().map(|val_ref| Ref(val_ref)),
            List(val) => val.try_clone(),
            Object(val) => val.try_clone(),

            // Language implementation should never try to clone a mutable reference.
            Mut(_) => panic!("Implementation error - tried to clone a mut ref."),
            ListMut(_) => panic!("Implementation error - tried to clone a mut list."),
            ObjectMut(_) => panic!("Implementation error - tried to clone a mut object."),
            Effect(_) => panic!("Implementation error - tried to clone an effect."),
        };
        match val {
            Some(val) => val,
            None => panic!("Couldn't clone the value, it has been mutably borrowed."),
        }
    }
}
