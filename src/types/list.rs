/*!
`List` is kal's native array type.
*/

use super::{Mut, Ref, Value};

#[derive(Debug, PartialEq)]
pub struct List {
    vec_ref: Ref<Vec<Value>>,
}

#[derive(Debug, PartialEq)]
pub struct ListMut {
    vec_mut: Mut<Vec<Value>>,
}

impl List {
    pub fn try_clone(&self) -> Option<Value> {
        self.vec_ref
            .try_clone()
            .map(|vec_ref| Value::List(List { vec_ref }))
    }
}
