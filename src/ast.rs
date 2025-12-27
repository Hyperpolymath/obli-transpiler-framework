// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! Abstract Syntax Tree for MiniObli.

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    Int(i64),
    /// Boolean literal
    Bool(bool),
    /// Variable reference
    Var(String),
    /// Secret value (marks data as sensitive)
    Secret(Box<Expr>),
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    /// If-then-else expression
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    /// Let binding
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
}

impl Expr {
    /// Check if expression contains any secret values.
    pub fn contains_secret(&self) -> bool {
        match self {
            Expr::Secret(_) => true,
            Expr::Int(_) | Expr::Bool(_) | Expr::Var(_) => false,
            Expr::BinOp { left, right, .. } => left.contains_secret() || right.contains_secret(),
            Expr::UnaryOp { expr, .. } => expr.contains_secret(),
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                cond.contains_secret()
                    || then_branch.contains_secret()
                    || else_branch.contains_secret()
            }
            Expr::Let { value, body, .. } => value.contains_secret() || body.contains_secret(),
        }
    }
}
