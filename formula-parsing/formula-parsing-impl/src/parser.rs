use serde::{Serialize, Deserialize};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Formula(pub Expression);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Expression {
    #[serde(rename(serialize = "Unary"))]
    #[serde(rename(deserialize = "Unary"))]
    UnaryExpression(UnaryExpression),
    #[serde(rename(serialize = "Binary"))]
    #[serde(rename(deserialize = "Binary"))]
    Binary(BinaryExpression),
    #[serde(rename(serialize = "NAry"))]
    #[serde(rename(deserialize = "NAry"))]
    NAryExpression(NAryExpression),
    #[serde(rename(serialize = "Literal"))]
    #[serde(rename(deserialize = "Literal"))]
    Literal(Literal),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UnaryExpression {
    #[serde(rename(serialize = "op"))]
    #[serde(rename(deserialize = "op"))]
    pub operator: TokenType,
    #[serde(rename(serialize = "var"))]
    #[serde(rename(deserialize = "var"))]
    pub argument: Box<Expression>,

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BinaryExpression {
    #[serde(rename(serialize = "op"))]
    #[serde(rename(deserialize = "op"))]
    pub operator: TokenType,
    #[serde(rename(serialize = "lhs"))]
    #[serde(rename(deserialize = "lhs"))]
    pub left: Box<Expression>,
    #[serde(rename(serialize = "rhs"))]
    #[serde(rename(deserialize = "rhs"))]
    pub right: Box<Expression>,
    
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NAryExpression {
    #[serde(rename(serialize = "op"))]
    #[serde(rename(deserialize = "op"))]
    pub operator: TokenType,
    #[serde(rename(serialize = "vars"))]
    #[serde(rename(deserialize = "vars"))]
    pub operands: Vec<Box<Expression>>,

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Literal {
    #[serde(rename(serialize = "constant"))]
    #[serde(rename(deserialize = "constant"))]
    NumericLiteral(f64),
    #[serde(rename(serialize = "var"))]
    #[serde(rename(deserialize = "var"))]
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum TokenType {
    NumberLiteral,
    Identifier,
    IgnoreToken,

    #[serde(rename(serialize = "Addition"))]
    #[serde(rename(deserialize = "Addition"))]
    OperatorAdd,
    #[serde(rename(serialize = "Subtraction"))]
    #[serde(rename(deserialize = "Subtraction"))]
    OperatorSubtract,
    #[serde(rename(serialize = "Division"))]
    #[serde(rename(deserialize = "Division"))]
    OperatorDivide,
    #[serde(rename(serialize = "Multiplication"))]
    #[serde(rename(deserialize = "Multiplication"))]
    OperatorMultiply,
    #[serde(rename(serialize = "Power"))]
    #[serde(rename(deserialize = "Power"))]
    OperatorPower,

    Semicolon,
    Comma,
    Hash,
    
    // All types of parantheses that exist
    OpenParen,
    CloseParen,
    OpenAngle,
    CloseAngle,
    OpenBracket,
    CloseBracket,
    OpenCurly,
    CloseCurly,
    #[serde(rename(serialize = "Absoulte"))]
    AbsLine,

    // There is no SUMME Keyword or any others
    #[serde(rename(serialize = "Squareroot"))]
    #[serde(rename(deserialize = "Squareroot"))]
    KeywordWurzel,
    #[serde(rename(serialize = "Minima"))]
    #[serde(rename(deserialize = "Minima"))]
    KeywordMin,
    #[serde(rename(serialize = "Maxima"))]
    #[serde(rename(deserialize = "Maxima"))]
    KeywordMax,

    Unrecognized(char),
}