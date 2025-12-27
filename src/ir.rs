// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! Oblivious Intermediate Representation.
//!
//! This IR represents programs where all operations are constant-time.
//! Branching on secrets is replaced with constant-time selection.

use crate::ast::BinOp;

/// Oblivious binary operators (constant-time).
#[derive(Debug, Clone, PartialEq)]
pub enum ObliBinOp {
    /// Constant-time addition
    CtAdd,
    /// Constant-time subtraction
    CtSub,
    /// Constant-time multiplication
    CtMul,
    /// Constant-time division (careful: timing may leak via hardware)
    CtDiv,
    /// Constant-time modulo
    CtMod,
    /// Constant-time equality
    CtEq,
    /// Constant-time not-equal
    CtNe,
    /// Constant-time less-than
    CtLt,
    /// Constant-time less-or-equal
    CtLe,
    /// Constant-time greater-than
    CtGt,
    /// Constant-time greater-or-equal
    CtGe,
    /// Constant-time logical AND
    CtAnd,
    /// Constant-time logical OR
    CtOr,
}

impl From<&BinOp> for ObliBinOp {
    fn from(op: &BinOp) -> Self {
        match op {
            BinOp::Add => ObliBinOp::CtAdd,
            BinOp::Sub => ObliBinOp::CtSub,
            BinOp::Mul => ObliBinOp::CtMul,
            BinOp::Div => ObliBinOp::CtDiv,
            BinOp::Mod => ObliBinOp::CtMod,
            BinOp::Eq => ObliBinOp::CtEq,
            BinOp::Ne => ObliBinOp::CtNe,
            BinOp::Lt => ObliBinOp::CtLt,
            BinOp::Le => ObliBinOp::CtLe,
            BinOp::Gt => ObliBinOp::CtGt,
            BinOp::Ge => ObliBinOp::CtGe,
            BinOp::And => ObliBinOp::CtAnd,
            BinOp::Or => ObliBinOp::CtOr,
        }
    }
}

/// Oblivious unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum ObliUnaryOp {
    /// Constant-time negation
    CtNeg,
    /// Constant-time logical NOT
    CtNot,
}

/// Oblivious expression - all operations are constant-time.
#[derive(Debug, Clone, PartialEq)]
pub enum ObliExpr {
    /// Public integer literal
    PubInt(i64),
    /// Public boolean literal
    PubBool(bool),
    /// Secret integer (runtime value)
    SecretInt(i64),
    /// Secret boolean (runtime value)
    SecretBool(bool),
    /// Variable reference (with secrecy flag)
    Var {
        name: String,
        is_secret: bool,
    },
    /// Constant-time binary operation
    BinOp {
        op: ObliBinOp,
        left: Box<ObliExpr>,
        right: Box<ObliExpr>,
        /// True if result depends on secret data
        is_secret: bool,
    },
    /// Constant-time unary operation
    UnaryOp {
        op: ObliUnaryOp,
        expr: Box<ObliExpr>,
        is_secret: bool,
    },
    /// Constant-time selection (replaces if-then-else on secrets)
    /// `ct_select(cond, then_val, else_val)` - always evaluates both branches
    CtSelect {
        cond: Box<ObliExpr>,
        then_val: Box<ObliExpr>,
        else_val: Box<ObliExpr>,
    },
    /// Public if-then-else (only when condition is public)
    PubIf {
        cond: Box<ObliExpr>,
        then_branch: Box<ObliExpr>,
        else_branch: Box<ObliExpr>,
    },
    /// Let binding
    Let {
        name: String,
        value: Box<ObliExpr>,
        body: Box<ObliExpr>,
        is_secret: bool,
    },
}

impl ObliExpr {
    /// Check if this expression is secret (depends on secret data).
    pub fn is_secret(&self) -> bool {
        match self {
            ObliExpr::PubInt(_) | ObliExpr::PubBool(_) => false,
            ObliExpr::SecretInt(_) | ObliExpr::SecretBool(_) => true,
            ObliExpr::Var { is_secret, .. } => *is_secret,
            ObliExpr::BinOp { is_secret, .. } => *is_secret,
            ObliExpr::UnaryOp { is_secret, .. } => *is_secret,
            ObliExpr::CtSelect { .. } => true, // ct_select always produces secret
            ObliExpr::PubIf { then_branch, else_branch, .. } => {
                then_branch.is_secret() || else_branch.is_secret()
            }
            ObliExpr::Let { is_secret, .. } => *is_secret,
        }
    }
}
