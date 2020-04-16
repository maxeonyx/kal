use std::collections::HashMap;

use gc::Finalize;
use gc_derive::Trace;

#[derive(Debug)]
pub enum Statement {
    Let(LetStatement),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Ident(Ident),
    FunctionInvocation(FunctionInvocation),
    If(IfExpression),
    Add(AddExpression),
    Subtract(SubtractExpression),
    Multiply(MultiplyExpression),
}

#[derive(Debug)]
pub struct AddExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct SubtractExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct MultiplyExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct LetStatement {
    pub variable: Ident,
    pub expr: Box<Expression>,
}

#[derive(Debug)]
pub struct IfExpression {
    pub expr: Box<Expression>,
    pub body: Block,
    pub else_body: Option<Block>,
}

#[derive(Debug)]
pub enum Literal {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Object(Object),
    List(List),
    Function(Function),
}

#[derive(Debug, Trace)]
pub struct Function {
    #[unsafe_ignore_trace]
    pub parameters: Vec<Ident>,

    #[unsafe_ignore_trace]
    pub body: Block,
}

impl Finalize for Function {
    fn finalize(&self) {}
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Box<Expression>,
}

#[derive(Debug)]
pub struct FunctionInvocation {
    pub closure_expression: Box<Expression>,
    pub parameters: Vec<Expression>,
}

#[derive(Debug)]
pub struct Object {
    pub map: HashMap<Ident, Expression>,
}

#[derive(Debug)]
pub struct List {
    pub elements: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
}
