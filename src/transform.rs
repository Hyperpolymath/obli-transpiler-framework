// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! Obliviousness transformation pass.
//!
//! This module transforms regular AST expressions into oblivious IR.
//! The key transformation is replacing `if-then-else` on secret conditions
//! with constant-time selection (`ct_select`).

use crate::ast::{Expr, UnaryOp};
use crate::ir::{ObliBinOp, ObliExpr, ObliUnaryOp};
use std::collections::HashSet;

/// Context for tracking which variables are secret.
struct TransformCtx {
    secret_vars: HashSet<String>,
}

impl TransformCtx {
    fn new() -> Self {
        Self {
            secret_vars: HashSet::new(),
        }
    }

    fn mark_secret(&mut self, name: &str) {
        self.secret_vars.insert(name.to_string());
    }

    fn is_secret(&self, name: &str) -> bool {
        self.secret_vars.contains(name)
    }
}

/// Transform an AST expression into oblivious IR.
pub fn to_oblivious(expr: &Expr) -> ObliExpr {
    let mut ctx = TransformCtx::new();
    transform_expr(expr, &mut ctx)
}

fn transform_expr(expr: &Expr, ctx: &mut TransformCtx) -> ObliExpr {
    match expr {
        Expr::Int(n) => ObliExpr::PubInt(*n),
        Expr::Bool(b) => ObliExpr::PubBool(*b),
        Expr::Var(name) => ObliExpr::Var {
            name: name.clone(),
            is_secret: ctx.is_secret(name),
        },
        Expr::Secret(inner) => {
            // Mark inner value as secret
            match inner.as_ref() {
                Expr::Int(n) => ObliExpr::SecretInt(*n),
                Expr::Bool(b) => ObliExpr::SecretBool(*b),
                _ => {
                    // For complex expressions, transform and mark as secret
                    let transformed = transform_expr(inner, ctx);
                    mark_as_secret(transformed)
                }
            }
        }
        Expr::BinOp { op, left, right } => {
            let left_obli = transform_expr(left, ctx);
            let right_obli = transform_expr(right, ctx);
            let is_secret = left_obli.is_secret() || right_obli.is_secret();

            ObliExpr::BinOp {
                op: ObliBinOp::from(op),
                left: Box::new(left_obli),
                right: Box::new(right_obli),
                is_secret,
            }
        }
        Expr::UnaryOp { op, expr: inner } => {
            let inner_obli = transform_expr(inner, ctx);
            let is_secret = inner_obli.is_secret();

            ObliExpr::UnaryOp {
                op: match op {
                    UnaryOp::Neg => ObliUnaryOp::CtNeg,
                    UnaryOp::Not => ObliUnaryOp::CtNot,
                },
                expr: Box::new(inner_obli),
                is_secret,
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_obli = transform_expr(cond, ctx);
            let then_obli = transform_expr(then_branch, ctx);
            let else_obli = transform_expr(else_branch, ctx);

            // KEY TRANSFORMATION: If condition is secret, use ct_select
            if cond_obli.is_secret() {
                ObliExpr::CtSelect {
                    cond: Box::new(cond_obli),
                    then_val: Box::new(then_obli),
                    else_val: Box::new(else_obli),
                }
            } else {
                // Public condition can use regular branching
                ObliExpr::PubIf {
                    cond: Box::new(cond_obli),
                    then_branch: Box::new(then_obli),
                    else_branch: Box::new(else_obli),
                }
            }
        }
        Expr::Let { name, value, body } => {
            let value_obli = transform_expr(value, ctx);
            let is_secret = value_obli.is_secret();

            // Track if this variable is secret
            if is_secret {
                ctx.mark_secret(name);
            }

            let body_obli = transform_expr(body, ctx);

            ObliExpr::Let {
                name: name.clone(),
                value: Box::new(value_obli),
                body: Box::new(body_obli),
                is_secret,
            }
        }
    }
}

/// Mark an expression as secret (propagate secrecy).
fn mark_as_secret(expr: ObliExpr) -> ObliExpr {
    match expr {
        ObliExpr::PubInt(n) => ObliExpr::SecretInt(n),
        ObliExpr::PubBool(b) => ObliExpr::SecretBool(b),
        ObliExpr::Var { name, .. } => ObliExpr::Var {
            name,
            is_secret: true,
        },
        ObliExpr::BinOp {
            op, left, right, ..
        } => ObliExpr::BinOp {
            op,
            left,
            right,
            is_secret: true,
        },
        ObliExpr::UnaryOp { op, expr, .. } => ObliExpr::UnaryOp {
            op,
            expr,
            is_secret: true,
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_and_transform(input: &str) -> ObliExpr {
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.filter_map(Result::ok).collect();
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();
        to_oblivious(&ast)
    }

    #[test]
    fn test_public_value() {
        let obli = parse_and_transform("42");
        assert!(!obli.is_secret());
    }

    #[test]
    fn test_secret_value() {
        let obli = parse_and_transform("secret(42)");
        assert!(obli.is_secret());
    }

    #[test]
    fn test_secret_if_becomes_ct_select() {
        let obli = parse_and_transform("let x = secret(1) if x > 0 then 1 else 0");
        // Should contain CtSelect, not PubIf
        match obli {
            ObliExpr::Let { body, .. } => {
                assert!(matches!(*body, ObliExpr::CtSelect { .. }));
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_public_if_stays_pub_if() {
        let obli = parse_and_transform("let x = 1 if x > 0 then 1 else 0");
        match obli {
            ObliExpr::Let { body, .. } => {
                assert!(matches!(*body, ObliExpr::PubIf { .. }));
            }
            _ => panic!("Expected Let"),
        }
    }
}
