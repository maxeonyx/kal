/*!
`List` is kal's native array type.
*/

use super::{Mut, Ref, Value};

#[derive(Debug, PartialEq)]
pub struct List {
    vec: Ref<Vec<Value>>,
}

#[derive(Debug, PartialEq)]
pub struct ListMut {
    vec: Mut<Vec<Value>>,
}

impl Clone for List {
    fn clone(&self) -> Self {
        List {
            vec: self.vec.try_clone().expect(
                "Implementation error - failed to clone a List, it is borrowed as ListMut.",
            ),
        }
    }
}
