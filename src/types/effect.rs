/*!
`Effect` contains a yielded value, symbol, and internally a continuation used when `resume`ing.
*/

use super::Value;
use crate::interpreter::FunctionContext;

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
