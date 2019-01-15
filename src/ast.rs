use std::collections::HashMap;

#[derive(Debug)]
pub enum Expression {
	Literal(Box<Literal>),
	FunctionInvocation(Box<FunctionInvocation>),
	Let(Box<LetExpression>),
	If(Box<IfExpression>),
}

#[derive(Debug)]
pub struct LetExpression(pub Box<Ident>, pub Box<Expression>);

#[derive(Debug)]
pub struct IfExpression(pub Box<Expression>, pub Box<Block>);

#[derive(Debug)]
pub enum Literal {
	Null,
	Bool(bool),
	Int(i64),
	String(String),
	Object(Box<Object>),
	List(Box<List>),
	Function(Box<Function>),
}

#[derive(Debug)]
pub struct Function {
	pub parameters: Vec<Ident>,
	pub body: Block,
}

#[derive(Debug)]
pub struct Block(pub Vec<Expression>);

#[derive(Debug)]
pub struct FunctionInvocation(pub Box<Ident>, pub Vec<Expression>);

#[derive(Debug)]
pub struct Object(pub HashMap<Ident, Expression>);

#[derive(Debug)]
pub struct List(pub Vec<Expression>);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Ident(pub String);
