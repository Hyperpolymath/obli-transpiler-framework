// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! Parser for MiniObli language.
//!
//! Grammar (simplified):
//! ```text
//! expr     → let_expr | if_expr | or_expr
//! let_expr → "let" IDENT "=" expr expr
//! if_expr  → "if" expr "then" expr "else" expr
//! or_expr  → and_expr ("or" and_expr)*
//! and_expr → cmp_expr ("and" cmp_expr)*
//! cmp_expr → add_expr (("==" | "!=" | "<" | "<=" | ">" | ">=") add_expr)?
//! add_expr → mul_expr (("+" | "-") mul_expr)*
//! mul_expr → unary (("*" | "/" | "%") unary)*
//! unary    → ("not" | "-") unary | primary
//! primary  → INT | BOOL | IDENT | "secret" "(" expr ")" | "(" expr ")"
//! ```

use crate::ast::{BinOp, Expr, UnaryOp};
use crate::lexer::Token;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token: {0:?}, expected {1}")]
    UnexpectedToken(Token, &'static str),
    #[error("unexpected end of input")]
    UnexpectedEof,
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if t == expected => {
                self.advance();
                Ok(())
            }
            Some(t) => Err(ParseError::UnexpectedToken(
                t.clone(),
                "specific token",
            )),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Let)?;

        let name = match self.advance() {
            Some(Token::Ident(n)) => n.clone(),
            Some(t) => return Err(ParseError::UnexpectedToken(t.clone(), "identifier")),
            None => return Err(ParseError::UnexpectedEof),
        };

        self.expect(&Token::Assign)?;
        let value = self.parse_expr()?;
        let body = self.parse_expr()?;

        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::If)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::Then)?;
        let then_branch = self.parse_expr()?;
        self.expect(&Token::Else)?;
        let else_branch = self.parse_expr()?;

        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;

        while matches!(self.peek(), Some(Token::Or)) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_cmp()?;

        while matches!(self.peek(), Some(Token::And)) {
            self.advance();
            let right = self.parse_cmp()?;
            left = Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add()?;

        let op = match self.peek() {
            Some(Token::Eq) => BinOp::Eq,
            Some(Token::Ne) => BinOp::Ne,
            Some(Token::Lt) => BinOp::Lt,
            Some(Token::Le) => BinOp::Le,
            Some(Token::Gt) => BinOp::Gt,
            Some(Token::Ge) => BinOp::Ge,
            _ => return Ok(left),
        };

        self.advance();
        let right = self.parse_add()?;

        Ok(Expr::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul()?;

        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Some(Token::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::Int(n)) => Ok(Expr::Int(*n)),
            Some(Token::Bool(b)) => Ok(Expr::Bool(*b)),
            Some(Token::Ident(name)) => Ok(Expr::Var(name.clone())),
            Some(Token::Secret) => {
                self.expect(&Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::Secret(Box::new(expr)))
            }
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(t) => Err(ParseError::UnexpectedToken(t.clone(), "expression")),
            None => Err(ParseError::UnexpectedEof),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> Result<Expr, ParseError> {
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.filter_map(Result::ok).collect();
        let mut parser = Parser::new(&tokens);
        parser.parse()
    }

    #[test]
    fn test_simple_expr() {
        let expr = parse("1 + 2").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Add, .. }));
    }

    #[test]
    fn test_secret() {
        let expr = parse("secret(42)").unwrap();
        assert!(matches!(expr, Expr::Secret(_)));
    }

    #[test]
    fn test_if_then_else() {
        let expr = parse("if x > 0 then 1 else 0").unwrap();
        assert!(matches!(expr, Expr::If { .. }));
    }

    #[test]
    fn test_let() {
        let expr = parse("let x = 1 x + 1").unwrap();
        assert!(matches!(expr, Expr::Let { .. }));
    }
}
