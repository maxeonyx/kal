/*!
`Object` is kal's native map type.
*/

use super::{Mut, Ref, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Null,
    Bool(bool),
    Int(i64),
    Symbol(u64),
    Str(String),
}

#[derive(Debug, PartialEq)]
pub struct Object {
    map: Ref<HashMap<Key, Value>>,
}

#[derive(Debug, PartialEq)]
pub struct ObjectMut {
    map: Mut<HashMap<Key, Value>>,
}

impl Clone for Object {
    fn clone(&self) -> Self {
        Object {
            map: self.map.try_clone().expect(
                "Implementation error - failed to clone an Object, it is borrowed as ObjectMut.",
            ),
        }
    }
}
