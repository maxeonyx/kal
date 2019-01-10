#[derive(Debug)]
pub enum Statement {
	Input,
	Print(Box<Expression>),
}

#[derive(Debug)]
pub enum Expression {
	True,
	False,
	And(Box<Expression>, Box<Expression>),
	Or(Box<Expression>, Box<Expression>),
}
