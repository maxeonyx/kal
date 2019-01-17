use std::collections::HashMap;

#[derive(Debug)]
pub enum Expression {
	Literal(Literal),
	FunctionInvocation(FunctionInvocation),
	Let(LetExpression),
	If(IfExpression),
}

#[derive(Debug)]
pub struct LetExpression {
	pub variable: Ident,
	pub expr: Box<Expression>,
}

#[derive(Debug)]
pub struct IfExpression {
	pub expr: Box<Expression>,
	pub body: Block,
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

#[derive(Debug)]
pub struct Function {
	pub parameters: Vec<Ident>,
	pub body: Block,
}

#[derive(Debug)]
pub struct Block {
	pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub struct FunctionInvocation {
	pub name: Ident,
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
