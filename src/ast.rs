use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub enum Statement {
    Let(LetStatement),
    Assignment(Assignment),
}

#[derive(Debug)]
pub struct LetStatement {
    pub mutable: bool,
    pub variable: Ident,
    pub expr: Rc<Expression>,
}

#[derive(Debug)]
pub struct Assignment {
    pub location: Rc<Location>,
    pub expr: Rc<Expression>,
}

#[derive(Debug)]
pub enum Location {
    Ident(Ident),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Ident(Ident),
    FunctionInvocation(FunctionInvocation),
    If(IfExpression),
    Numeric(NumericExpression),
    Comparison(ComparisonExpression),
    Dot(DotExpression),
    Index(IndexExpression),
    Boolean(BooleanExpression),
    Not(NotExpression),
    Negative(NegativeExpression),
}

#[derive(Debug)]
pub struct NegativeExpression {
    pub expr: Rc<Expression>,
}

#[derive(Debug)]
pub struct NotExpression {
    pub expr: Rc<Expression>,
}

#[derive(Debug)]
pub struct BooleanExpression {
    pub left: Rc<Expression>,
    pub right: Rc<Expression>,
    pub operator: BooleanOperator,
}

#[derive(Debug)]
pub enum BooleanOperator {
    And,
    Or,
    Xor,
}

#[derive(Debug)]
pub struct DotExpression {
    pub base: Rc<Expression>,
    pub prop: Ident,
}

#[derive(Debug)]
pub struct IndexExpression {
    pub base: Rc<Expression>,
    pub index: Rc<Expression>,
}

#[derive(Debug)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessEqual,
    Less,
    GreaterEqual,
    Greater,
}

#[derive(Debug)]
pub struct ComparisonExpression {
    pub left: Rc<Expression>,
    pub right: Rc<Expression>,
    pub operator: ComparisonOperator,
}

#[derive(Debug, Clone, Copy)]
pub enum NumericOperator {
    Add,
    Multiply,
    Subtract,
    Divide,
}

#[derive(Debug)]
pub struct NumericExpression {
    pub left: Rc<Expression>,
    pub right: Rc<Expression>,
    pub operator: NumericOperator,
}

#[derive(Debug)]
pub struct IfExpression {
    pub ifs: Vec<IfPart>,
    pub else_body: Option<Block>,
}

#[derive(Debug)]
pub struct IfPart {
    pub cond: Expression,
    pub body: Block,
}

#[derive(Debug)]
pub enum Literal {
    Null,
    Bool(bool),
    Symbol,
    Int(i64),
    Object(ObjectLiteral),
    List(ListLiteral),
    Function(Function),
}

#[derive(Debug)]
pub struct Function {
    pub parameters: Vec<Ident>,
    pub body: Block,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Rc<Statement>>,
    pub expression: Option<Rc<Expression>>,
}

#[derive(Debug)]
pub struct FunctionInvocation {
    pub closure_expression: Rc<Expression>,
    pub parameters: Vec<Expression>,
}

#[derive(Debug)]
pub struct ObjectLiteral {
    pub map: HashMap<Ident, Expression>,
}

#[derive(Debug)]
pub struct ListLiteral {
    pub elements: Vec<ListLiteralElem>,
}

#[derive(Debug)]
pub enum ListLiteralElem {
    Spread(Expression),
    Elem(Expression),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
}
