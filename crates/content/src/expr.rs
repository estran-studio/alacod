use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A compile-time type for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Num,
    Bool,
}

/// The value result of evaluating an expression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Num(Fixed),
    Bool(bool),
}

impl Value {
    pub fn as_fixed(self) -> Fixed {
        match self {
            Value::Num(n) => n,
            _ => Fixed::ZERO,
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            Value::Bool(b) => b,
            _ => false,
        }
    }
}

/// Parse error with position information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub pos: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at {}: {}", self.pos, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Context trait for identifier lookups during evaluation
pub trait Context {
    fn get(&self, name: &str) -> Option<Fixed>;
}

impl Context for BTreeMap<String, Fixed> {
    fn get(&self, name: &str) -> Option<Fixed> {
        self.get(name).copied()
    }
}

/// AST node for expressions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Expr {
    source: String,
    ast: AstNode,
    kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AstNode {
    Num(Fixed),
    #[allow(dead_code)]
    Bool(bool),
    Ident(String),
    BinOp {
        op: BinOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<AstNode>,
    },
    Call {
        name: String,
        args: Vec<AstNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnaryOp {
    Neg,
    Not,
}

impl Expr {
    /// Parse an expression from a string
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let source = source.trim();
        let mut parser = Parser::new(source);
        let ast = parser.parse_expr()?;
        let kind = infer_kind(&ast)?;
        Ok(Expr {
            source: source.to_string(),
            ast,
            kind,
        })
    }

    /// Get the static type of this expression
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Get all identifiers used in this expression (sorted, deduplicated)
    pub fn identifiers(&self) -> Vec<String> {
        let mut idents = Vec::new();
        collect_idents(&self.ast, &mut idents);
        idents.sort();
        idents.dedup();
        idents
    }

    /// Evaluate the expression in the given context
    pub fn eval(&self, ctx: &dyn Context) -> Value {
        eval_node(&self.ast, ctx)
    }

    /// Evaluate as a fixed-point number (panics if kind != Num)
    pub fn eval_num(&self, ctx: &dyn Context) -> Fixed {
        if self.kind != Kind::Num {
            panic!("eval_num called on non-numeric expression");
        }
        self.eval(ctx).as_fixed()
    }

    /// Evaluate as a boolean (panics if kind != Bool)
    pub fn eval_bool(&self, ctx: &dyn Context) -> bool {
        if self.kind != Kind::Bool {
            panic!("eval_bool called on non-boolean expression");
        }
        self.eval(ctx).as_bool()
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl From<Expr> for String {
    fn from(e: Expr) -> Self {
        e.source
    }
}

impl TryFrom<String> for Expr {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Expr::parse(&s)
    }
}

/// A numeric expression (type-checked at parse time to ensure kind == Num)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumExpr(Expr);

impl NumExpr {
    /// Parse a numeric expression, failing if it evaluates to a boolean
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let expr = Expr::parse(source)?;
        if expr.kind() != Kind::Num {
            return Err(ParseError {
                pos: 0,
                message: "expression must be numeric".to_string(),
            });
        }
        Ok(NumExpr(expr))
    }

    /// Get the underlying expression
    pub fn expr(&self) -> &Expr {
        &self.0
    }

    /// Evaluate the expression
    pub fn eval(&self, ctx: &dyn Context) -> Fixed {
        self.0.eval_num(ctx)
    }
}

impl TryFrom<String> for NumExpr {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        NumExpr::parse(&s)
    }
}

impl From<NumExpr> for String {
    fn from(e: NumExpr) -> Self {
        e.0.source
    }
}

impl fmt::Display for NumExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A boolean expression (type-checked at parse time to ensure kind == Bool)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoolExpr(Expr);

impl BoolExpr {
    /// Parse a boolean expression, failing if it evaluates to a number
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let expr = Expr::parse(source)?;
        if expr.kind() != Kind::Bool {
            return Err(ParseError {
                pos: 0,
                message: "expression must be boolean".to_string(),
            });
        }
        Ok(BoolExpr(expr))
    }

    /// Get the underlying expression
    pub fn expr(&self) -> &Expr {
        &self.0
    }

    /// Evaluate the expression
    pub fn eval(&self, ctx: &dyn Context) -> bool {
        self.0.eval_bool(ctx)
    }
}

impl TryFrom<String> for BoolExpr {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        BoolExpr::parse(&s)
    }
}

impl From<BoolExpr> for String {
    fn from(e: BoolExpr) -> Self {
        e.0.source
    }
}

impl fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- Type inference ---

fn infer_kind(node: &AstNode) -> Result<Kind, ParseError> {
    match node {
        AstNode::Num(_) => Ok(Kind::Num),
        AstNode::Bool(_) => Ok(Kind::Bool),
        AstNode::Ident(_) => Ok(Kind::Num), // Identifiers are always numeric
        AstNode::BinOp { op, left, right } => {
            let left_kind = infer_kind(left)?;
            let right_kind = infer_kind(right)?;
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    if left_kind != Kind::Num || right_kind != Kind::Num {
                        return Err(ParseError {
                            pos: 0,
                            message: format!(
                                "arithmetic operator {:?} requires numeric operands",
                                op
                            ),
                        });
                    }
                    Ok(Kind::Num)
                }
                BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte | BinOp::Eq | BinOp::Neq => {
                    if left_kind != Kind::Num || right_kind != Kind::Num {
                        return Err(ParseError {
                            pos: 0,
                            message: format!(
                                "comparison operator {:?} requires numeric operands",
                                op
                            ),
                        });
                    }
                    Ok(Kind::Bool)
                }
                BinOp::And | BinOp::Or => {
                    if left_kind != Kind::Bool || right_kind != Kind::Bool {
                        return Err(ParseError {
                            pos: 0,
                            message: format!("logical operator {:?} requires boolean operands", op),
                        });
                    }
                    Ok(Kind::Bool)
                }
            }
        }
        AstNode::UnaryOp { op, operand } => {
            let operand_kind = infer_kind(operand)?;
            match op {
                UnaryOp::Neg => {
                    if operand_kind != Kind::Num {
                        return Err(ParseError {
                            pos: 0,
                            message: "unary negation requires numeric operand".to_string(),
                        });
                    }
                    Ok(Kind::Num)
                }
                UnaryOp::Not => {
                    if operand_kind != Kind::Bool {
                        return Err(ParseError {
                            pos: 0,
                            message: "logical not requires boolean operand".to_string(),
                        });
                    }
                    Ok(Kind::Bool)
                }
            }
        }
        AstNode::Call { name, args } => match name.as_str() {
            "min" | "max" => {
                if args.len() != 2 {
                    return Err(ParseError {
                        pos: 0,
                        message: format!("{} requires 2 arguments", name),
                    });
                }
                let arg1_kind = infer_kind(&args[0])?;
                let arg2_kind = infer_kind(&args[1])?;
                if arg1_kind != Kind::Num || arg2_kind != Kind::Num {
                    return Err(ParseError {
                        pos: 0,
                        message: format!("{} requires numeric arguments", name),
                    });
                }
                Ok(Kind::Num)
            }
            "clamp" => {
                if args.len() != 3 {
                    return Err(ParseError {
                        pos: 0,
                        message: "clamp requires 3 arguments".to_string(),
                    });
                }
                for arg in args {
                    if infer_kind(arg)? != Kind::Num {
                        return Err(ParseError {
                            pos: 0,
                            message: "clamp requires numeric arguments".to_string(),
                        });
                    }
                }
                Ok(Kind::Num)
            }
            "abs" | "floor" => {
                if args.len() != 1 {
                    return Err(ParseError {
                        pos: 0,
                        message: format!("{} requires 1 argument", name),
                    });
                }
                if infer_kind(&args[0])? != Kind::Num {
                    return Err(ParseError {
                        pos: 0,
                        message: format!("{} requires numeric argument", name),
                    });
                }
                Ok(Kind::Num)
            }
            _ => Err(ParseError {
                pos: 0,
                message: format!("unknown function: {}", name),
            }),
        },
    }
}

// --- Identifier collection ---

fn collect_idents(node: &AstNode, idents: &mut Vec<String>) {
    match node {
        AstNode::Num(_) | AstNode::Bool(_) => {}
        AstNode::Ident(name) => idents.push(name.clone()),
        AstNode::BinOp { left, right, .. } => {
            collect_idents(left, idents);
            collect_idents(right, idents);
        }
        AstNode::UnaryOp { operand, .. } => collect_idents(operand, idents),
        AstNode::Call { args, .. } => {
            for arg in args {
                collect_idents(arg, idents);
            }
        }
    }
}

// --- Evaluation ---

fn eval_node(node: &AstNode, ctx: &dyn Context) -> Value {
    match node {
        AstNode::Num(n) => Value::Num(*n),
        AstNode::Bool(b) => Value::Bool(*b),
        AstNode::Ident(name) => Value::Num(ctx.get(name).unwrap_or(Fixed::ZERO)),
        AstNode::BinOp { op, left, right } => {
            let left_val = eval_node(left, ctx).as_fixed();
            let right_val = eval_node(right, ctx).as_fixed();
            match op {
                BinOp::Add => Value::Num(left_val.saturating_add(right_val)),
                BinOp::Sub => Value::Num(left_val.saturating_sub(right_val)),
                BinOp::Mul => Value::Num(left_val.saturating_mul(right_val)),
                BinOp::Div => {
                    if right_val == Fixed::ZERO {
                        // Division by zero: return MAX or MIN based on sign
                        if left_val < Fixed::ZERO {
                            Value::Num(Fixed::MIN)
                        } else {
                            Value::Num(Fixed::MAX)
                        }
                    } else {
                        Value::Num(left_val.saturating_div(right_val))
                    }
                }
                BinOp::Lt => Value::Bool(left_val < right_val),
                BinOp::Lte => Value::Bool(left_val <= right_val),
                BinOp::Gt => Value::Bool(left_val > right_val),
                BinOp::Gte => Value::Bool(left_val >= right_val),
                BinOp::Eq => Value::Bool(left_val == right_val),
                BinOp::Neq => Value::Bool(left_val != right_val),
                BinOp::And => {
                    Value::Bool(eval_node(left, ctx).as_bool() && eval_node(right, ctx).as_bool())
                }
                BinOp::Or => {
                    Value::Bool(eval_node(left, ctx).as_bool() || eval_node(right, ctx).as_bool())
                }
            }
        }
        AstNode::UnaryOp { op, operand } => match op {
            UnaryOp::Neg => Value::Num(eval_node(operand, ctx).as_fixed().saturating_neg()),
            UnaryOp::Not => Value::Bool(!eval_node(operand, ctx).as_bool()),
        },
        AstNode::Call { name, args } => {
            match name.as_str() {
                "min" => {
                    let a = eval_node(&args[0], ctx).as_fixed();
                    let b = eval_node(&args[1], ctx).as_fixed();
                    Value::Num(a.min(b))
                }
                "max" => {
                    let a = eval_node(&args[0], ctx).as_fixed();
                    let b = eval_node(&args[1], ctx).as_fixed();
                    Value::Num(a.max(b))
                }
                "clamp" => {
                    let x = eval_node(&args[0], ctx).as_fixed();
                    let lo = eval_node(&args[1], ctx).as_fixed();
                    let hi = eval_node(&args[2], ctx).as_fixed();
                    Value::Num(x.max(lo).min(hi))
                }
                "abs" => Value::Num(eval_node(&args[0], ctx).as_fixed().abs()),
                "floor" => Value::Num(eval_node(&args[0], ctx).as_fixed().floor()),
                _ => Value::Num(Fixed::ZERO), // Should never happen if type-checked
            }
        }
    }
}

// --- Lexer ---

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Number(Fixed),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    Comma,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    And,
    Or,
    Not,
    Eof,
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(s: &str) -> Self {
        Lexer {
            input: s.chars().collect(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Fixed, ParseError> {
        let start = self.pos;
        let mut has_dot = false;

        // Handle leading minus
        if self.current() == Some('-') {
            self.advance();
        }

        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str.parse::<Fixed>().map_err(|_| ParseError {
            pos: start,
            message: format!("invalid number: {}", num_str),
        })
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' || c == '.' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].iter().collect()
    }

    fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace();

        match self.current() {
            None => Ok(Token::Eof),
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('-') => {
                // Could be minus or start of negative number
                // We'll handle negative numbers in the parser
                self.advance();
                Ok(Token::Minus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Star)
            }
            Some('/') => {
                self.advance();
                Ok(Token::Slash)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LeftParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::Lte)
                } else {
                    Ok(Token::Lt)
                }
            }
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::Gte)
                } else {
                    Ok(Token::Gt)
                }
            }
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::Eq)
                } else {
                    Err(ParseError {
                        pos: self.pos - 1,
                        message: "unexpected '='".to_string(),
                    })
                }
            }
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::Neq)
                } else {
                    Ok(Token::Not)
                }
            }
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Err(ParseError {
                        pos: self.pos - 1,
                        message: "unexpected '&'".to_string(),
                    })
                }
            }
            Some('|') => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Err(ParseError {
                        pos: self.pos - 1,
                        message: "unexpected '|'".to_string(),
                    })
                }
            }
            Some(c) if c.is_ascii_digit() => self.read_number().map(Token::Number),
            Some(c) if c.is_alphabetic() || c == '_' => {
                let ident = self.read_ident();
                Ok(Token::Ident(ident))
            }
            Some(c) => Err(ParseError {
                pos: self.pos,
                message: format!("unexpected character: '{}'", c),
            }),
        }
    }
}

// --- Parser ---

struct Parser {
    lexer: Lexer,
    current: Token,
}

impl Parser {
    fn new(s: &str) -> Self {
        let mut lexer = Lexer::new(s);
        let current = lexer.next_token().unwrap_or(Token::Eof);
        Parser { lexer, current }
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    fn parse_expr(&mut self) -> Result<AstNode, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_and()?;

        while self.current == Token::Or {
            self.advance()?;
            let right = self.parse_and()?;
            left = AstNode::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_comparison()?;

        while self.current == Token::And {
            self.advance()?;
            let right = self.parse_comparison()?;
            left = AstNode::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_additive()?;

        while let Some(op) = match self.current {
            Token::Lt => Some(BinOp::Lt),
            Token::Lte => Some(BinOp::Lte),
            Token::Gt => Some(BinOp::Gt),
            Token::Gte => Some(BinOp::Gte),
            Token::Eq => Some(BinOp::Eq),
            Token::Neq => Some(BinOp::Neq),
            _ => None,
        } {
            self.advance()?;
            let right = self.parse_additive()?;
            left = AstNode::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.current {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_multiplicative()?;
            left = AstNode::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_unary()?;
            left = AstNode::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<AstNode, ParseError> {
        match self.current {
            Token::Minus => {
                self.advance()?;
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Token::Not => {
                self.advance()?;
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<AstNode, ParseError> {
        match &self.current {
            Token::Number(n) => {
                let n = *n;
                self.advance()?;
                Ok(AstNode::Num(n))
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expr()?;
                if self.current != Token::RightParen {
                    return Err(ParseError {
                        pos: self.lexer.pos,
                        message: "expected ')'".to_string(),
                    });
                }
                self.advance()?;
                Ok(expr)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance()?;

                // Check if it's a function call
                if self.current == Token::LeftParen {
                    self.advance()?;
                    let mut args = Vec::new();

                    if self.current != Token::RightParen {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.current == Token::Comma {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }

                    if self.current != Token::RightParen {
                        return Err(ParseError {
                            pos: self.lexer.pos,
                            message: "expected ')'".to_string(),
                        });
                    }
                    self.advance()?;

                    Ok(AstNode::Call { name, args })
                } else {
                    Ok(AstNode::Ident(name))
                }
            }
            _ => Err(ParseError {
                pos: self.lexer.pos,
                message: format!("unexpected token: {:?}", self.current),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(values: &[(&str, f32)]) -> BTreeMap<String, Fixed> {
        values
            .iter()
            .map(|(k, v)| (k.to_string(), Fixed::from_num(*v)))
            .collect()
    }

    // Basic literals
    #[test]
    fn test_integer_literal() {
        let expr = Expr::parse("42").unwrap();
        assert_eq!(expr.kind(), Kind::Num);
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(42));
    }

    #[test]
    fn test_decimal_literal() {
        let expr = Expr::parse("0.5").unwrap();
        assert_eq!(expr.kind(), Kind::Num);
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(0.5));
    }

    #[test]
    fn test_negative_literal() {
        let expr = Expr::parse("-3").unwrap();
        assert_eq!(expr.kind(), Kind::Num);
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(-3));
    }

    // Arithmetic
    #[test]
    fn test_addition() {
        let expr = Expr::parse("1 + 2").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(3));
    }

    #[test]
    fn test_subtraction() {
        let expr = Expr::parse("5 - 3").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(2));
    }

    #[test]
    fn test_multiplication() {
        let expr = Expr::parse("3 * 4").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(12));
    }

    #[test]
    fn test_division() {
        let expr = Expr::parse("10 / 2").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(5));
    }

    // Precedence
    #[test]
    fn test_precedence_1() {
        let expr = Expr::parse("1 + 2 * 3").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(7));
    }

    #[test]
    fn test_precedence_2() {
        let expr = Expr::parse("(1 + 2) * 3").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(9));
    }

    #[test]
    fn test_unary_negation() {
        let expr = Expr::parse("-2 * 3").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(-6));
    }

    #[test]
    fn test_unary_negation_parenthesized() {
        let expr = Expr::parse("2 * -3").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(-6));
    }

    // Decimal arithmetic
    #[test]
    fn test_decimal_add() {
        let expr = Expr::parse("0.5 + 0.25").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(0.75));
    }

    #[test]
    fn test_decimal_multiply() {
        let expr = Expr::parse("0.5 * 0.5").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(0.25));
    }

    // Identifiers
    #[test]
    fn test_simple_ident() {
        let expr = Expr::parse("players").unwrap();
        let c = ctx(&[("players", 4.0)]);
        assert_eq!(expr.eval(&c).as_fixed(), Fixed::from_num(4));
    }

    #[test]
    fn test_ident_with_dot() {
        let expr = Expr::parse("stat.speed").unwrap();
        let c = ctx(&[("stat.speed", 10.0)]);
        assert_eq!(expr.eval(&c).as_fixed(), Fixed::from_num(10));
    }

    #[test]
    fn test_ident_unknown() {
        let expr = Expr::parse("unknown").unwrap();
        let c = ctx(&[]);
        assert_eq!(expr.eval(&c).as_fixed(), Fixed::ZERO);
    }

    #[test]
    fn test_identifiers_list() {
        let expr = Expr::parse("players + wave.count").unwrap();
        let mut idents = expr.identifiers();
        idents.sort();
        assert_eq!(idents, vec!["players", "wave.count"]);
    }

    #[test]
    fn test_identifiers_deduplicated() {
        let expr = Expr::parse("x + x + y").unwrap();
        let mut idents = expr.identifiers();
        idents.sort();
        assert_eq!(idents, vec!["x", "y"]);
    }

    // Comparisons
    #[test]
    fn test_comparison_lt() {
        let expr = Expr::parse("1 < 2").unwrap();
        assert_eq!(expr.kind(), Kind::Bool);
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), true);
    }

    #[test]
    fn test_comparison_gte() {
        let expr = Expr::parse("5 >= 3").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), true);
    }

    #[test]
    fn test_comparison_eq() {
        let expr = Expr::parse("42 == 42").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), true);
    }

    #[test]
    fn test_comparison_neq() {
        let expr = Expr::parse("1 != 2").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), true);
    }

    // Logical operators
    #[test]
    fn test_and_true() {
        let expr = Expr::parse("1 < 2 && 3 < 4").unwrap();
        assert_eq!(expr.kind(), Kind::Bool);
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), true);
    }

    #[test]
    fn test_and_false() {
        let expr = Expr::parse("1 < 2 && 3 > 4").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), false);
    }

    #[test]
    fn test_or_true() {
        let expr = Expr::parse("1 > 2 || 3 < 4").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), true);
    }

    #[test]
    fn test_not() {
        let expr = Expr::parse("!(1 < 2)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_bool(), false);
    }

    // Functions
    #[test]
    fn test_min() {
        let expr = Expr::parse("min(10, 5)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(5));
    }

    #[test]
    fn test_max() {
        let expr = Expr::parse("max(10, 5)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(10));
    }

    #[test]
    fn test_clamp() {
        let expr = Expr::parse("clamp(50, 0, 100)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(50));
    }

    #[test]
    fn test_clamp_below() {
        let expr = Expr::parse("clamp(-10, 0, 100)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(0));
    }

    #[test]
    fn test_clamp_above() {
        let expr = Expr::parse("clamp(150, 0, 100)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(100));
    }

    #[test]
    fn test_abs() {
        let expr = Expr::parse("abs(-5)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(5));
    }

    #[test]
    fn test_floor() {
        let expr = Expr::parse("floor(2.7)").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::from_num(2));
    }

    // Complex expressions
    #[test]
    fn test_expr_with_idents() {
        let expr = Expr::parse("4 + players").unwrap();
        let c = ctx(&[("players", 3.0)]);
        assert_eq!(expr.eval(&c).as_fixed(), Fixed::from_num(7));
    }

    #[test]
    fn test_expr_max_hp_percent() {
        let expr = Expr::parse("max_hp * 0.3").unwrap();
        let c = ctx(&[("max_hp", 100.0)]);
        let result = expr.eval(&c).as_fixed();
        let expected = Fixed::from_num(30);
        // Fixed-point precision: allow small deviation
        assert!((result - expected).abs() < Fixed::from_num(0.001));
    }

    #[test]
    fn test_expr_gauge_compare() {
        let expr = Expr::parse("gauge.sacre >= 0.75").unwrap();
        let c = ctx(&[("gauge.sacre", 0.8)]);
        assert_eq!(expr.eval(&c).as_bool(), true);
    }

    // Saturation
    #[test]
    fn test_saturation_overflow() {
        let expr = Expr::parse("32767 * 32767").unwrap();
        // Should saturate, not panic
        let result = expr.eval(&ctx(&[]));
        // Result should be MAX due to saturation
        assert_eq!(result.as_fixed(), Fixed::MAX);
    }

    // Division by zero
    #[test]
    fn test_division_by_zero_positive() {
        let expr = Expr::parse("10 / 0").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::MAX);
    }

    #[test]
    fn test_division_by_zero_negative() {
        let expr = Expr::parse("-10 / 0").unwrap();
        assert_eq!(expr.eval(&ctx(&[])).as_fixed(), Fixed::MIN);
    }

    // Type checking errors
    #[test]
    fn test_type_error_add_bool_num() {
        let result = Expr::parse("(1 < 2) + 3");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.message.contains("arithmetic") || e.message.contains("numeric"));
        }
    }

    #[test]
    fn test_type_error_and_num() {
        let result = Expr::parse("1 && 2");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.message.contains("logical") || e.message.contains("boolean"));
        }
    }

    #[test]
    fn test_type_error_not_num() {
        let result = Expr::parse("!(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_error_min_mixed() {
        let result = Expr::parse("min(1, 1 < 2)");
        assert!(result.is_err());
    }

    // Display and serde
    #[test]
    fn test_display() {
        let expr = Expr::parse("1 + 2 * 3").unwrap();
        assert_eq!(expr.to_string(), "1 + 2 * 3");
    }

    #[test]
    fn test_display_whitespace_preserved_minimal() {
        let expr = Expr::parse("1+2*3").unwrap();
        assert_eq!(expr.to_string(), "1+2*3");
    }

    #[test]
    fn test_serde_ron() {
        let expr = Expr::parse("4 + players").unwrap();
        let ron_str = ron::to_string(&expr).unwrap();
        let parsed: Expr = ron::from_str(&ron_str).unwrap();
        let c = ctx(&[("players", 3.0)]);
        assert_eq!(parsed.eval(&c).as_fixed(), Fixed::from_num(7));
    }

    #[test]
    fn test_parse_error_position() {
        let result = Expr::parse("1 +++ 2");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.pos >= 0);
        }
    }

    #[test]
    fn test_parse_error_unknown_func() {
        let result = Expr::parse("unknown_func(1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_paren_mismatch() {
        let result = Expr::parse("(1 + 2");
        assert!(result.is_err());
    }

    // NumExpr and BoolExpr
    #[test]
    fn test_num_expr_parse() {
        let num_expr = NumExpr::parse("1 + 2").unwrap();
        assert_eq!(num_expr.eval(&ctx(&[])), Fixed::from_num(3));
    }

    #[test]
    fn test_num_expr_reject_bool() {
        let result = NumExpr::parse("1 < 2");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.message.contains("numeric"));
        }
    }

    #[test]
    fn test_bool_expr_parse() {
        let bool_expr = BoolExpr::parse("1 < 2").unwrap();
        assert_eq!(bool_expr.eval(&ctx(&[])), true);
    }

    #[test]
    fn test_bool_expr_reject_num() {
        let result = BoolExpr::parse("42");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.message.contains("boolean"));
        }
    }

    #[test]
    fn test_num_expr_serde() {
        let num_expr = NumExpr::parse("10 + players").unwrap();
        let ron_str = ron::to_string(&num_expr).unwrap();
        let parsed: NumExpr = ron::from_str(&ron_str).unwrap();
        let c = ctx(&[("players", 5.0)]);
        assert_eq!(parsed.eval(&c), Fixed::from_num(15));
    }

    #[test]
    fn test_bool_expr_serde() {
        let bool_expr = BoolExpr::parse("health > 0").unwrap();
        let ron_str = ron::to_string(&bool_expr).unwrap();
        let parsed: BoolExpr = ron::from_str(&ron_str).unwrap();
        let c = ctx(&[("health", 10.0)]);
        assert_eq!(parsed.eval(&c), true);
    }
}
