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
    map_ref: Ref<HashMap<Key, Value>>,
}

#[derive(Debug, PartialEq)]
pub struct ObjectMut {
    map_mut: Mut<HashMap<Key, Value>>,
}

impl Object {
    pub fn try_clone(&self) -> Option<Value> {
        self.map_ref
            .try_clone()
            .map(|map_ref| Value::Object(Object { map_ref }))
    }
}
