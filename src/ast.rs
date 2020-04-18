use std::collections::HashMap;

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
    pub expr: Box<Expression>,
}

#[derive(Debug)]
pub struct NotExpression {
    pub expr: Box<Expression>,
}

#[derive(Debug)]
pub struct BooleanExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
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
    pub base: Box<Expression>,
    pub prop: Ident,
}

#[derive(Debug)]
pub struct IndexExpression {
    pub base: Box<Expression>,
    pub index: Box<Expression>,
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
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator: ComparisonOperator,
}

#[derive(Debug)]
pub enum NumericOperator {
    Add,
    Multiply,
    Subtract,
    Divide,
}

#[derive(Debug)]
pub struct NumericExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator: NumericOperator,
}

#[derive(Debug)]
pub struct LetStatement {
    pub variable: Ident,
    pub expr: Box<Expression>,
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
    String(String),
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
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}

#[derive(Debug)]
pub struct FunctionInvocation {
    pub closure_expression: Box<Expression>,
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
