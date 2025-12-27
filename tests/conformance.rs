// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! IR Conformance Tests
//!
//! These tests verify that the ObliIR transformation satisfies
//! the properties defined in docs/IR_SPEC.adoc.

use obli_transpiler::ast::Expr;
use obli_transpiler::ir::{ObliBinOp, ObliExpr, ObliUnaryOp};
use obli_transpiler::Lexer;
use obli_transpiler::Parser;
use obli_transpiler::to_oblivious;

// ============================================================================
// Helper Functions
// ============================================================================

fn parse(input: &str) -> Expr {
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.filter_map(Result::ok).collect();
    let mut parser = Parser::new(&tokens);
    parser.parse().expect("parse failed")
}

fn transform(input: &str) -> ObliExpr {
    to_oblivious(&parse(input))
}

/// Check if IR contains any PubIf with secret condition (VIOLATION)
fn contains_secret_pub_if(expr: &ObliExpr) -> bool {
    match expr {
        ObliExpr::PubIf { cond, then_branch, else_branch } => {
            // Violation: PubIf with secret condition
            if cond.is_secret() {
                return true;
            }
            contains_secret_pub_if(cond)
                || contains_secret_pub_if(then_branch)
                || contains_secret_pub_if(else_branch)
        }
        ObliExpr::CtSelect { cond, then_val, else_val } => {
            contains_secret_pub_if(cond)
                || contains_secret_pub_if(then_val)
                || contains_secret_pub_if(else_val)
        }
        ObliExpr::BinOp { left, right, .. } => {
            contains_secret_pub_if(left) || contains_secret_pub_if(right)
        }
        ObliExpr::UnaryOp { expr, .. } => contains_secret_pub_if(expr),
        ObliExpr::Let { value, body, .. } => {
            contains_secret_pub_if(value) || contains_secret_pub_if(body)
        }
        _ => false,
    }
}

/// Check if IR contains CtSelect (used to verify secret conditionals transform)
fn contains_ct_select(expr: &ObliExpr) -> bool {
    match expr {
        ObliExpr::CtSelect { .. } => true,
        ObliExpr::PubIf { cond, then_branch, else_branch } => {
            contains_ct_select(cond)
                || contains_ct_select(then_branch)
                || contains_ct_select(else_branch)
        }
        ObliExpr::BinOp { left, right, .. } => {
            contains_ct_select(left) || contains_ct_select(right)
        }
        ObliExpr::UnaryOp { expr, .. } => contains_ct_select(expr),
        ObliExpr::Let { value, body, .. } => {
            contains_ct_select(value) || contains_ct_select(body)
        }
        _ => false,
    }
}

/// Check if IR contains PubIf (used to verify public conditionals stay as PubIf)
fn contains_pub_if(expr: &ObliExpr) -> bool {
    match expr {
        ObliExpr::PubIf { .. } => true,
        ObliExpr::CtSelect { cond, then_val, else_val } => {
            contains_pub_if(cond) || contains_pub_if(then_val) || contains_pub_if(else_val)
        }
        ObliExpr::BinOp { left, right, .. } => {
            contains_pub_if(left) || contains_pub_if(right)
        }
        ObliExpr::UnaryOp { expr, .. } => contains_pub_if(expr),
        ObliExpr::Let { value, body, .. } => {
            contains_pub_if(value) || contains_pub_if(body)
        }
        _ => false,
    }
}

/// Verify all BinOp nodes have correct is_secret flag
fn verify_binop_secrecy(expr: &ObliExpr) -> bool {
    match expr {
        ObliExpr::BinOp { left, right, is_secret, .. } => {
            let expected_secret = left.is_secret() || right.is_secret();
            if *is_secret != expected_secret {
                return false;
            }
            verify_binop_secrecy(left) && verify_binop_secrecy(right)
        }
        ObliExpr::UnaryOp { expr, is_secret, .. } => {
            if *is_secret != expr.is_secret() {
                return false;
            }
            verify_binop_secrecy(expr)
        }
        ObliExpr::CtSelect { cond, then_val, else_val } => {
            verify_binop_secrecy(cond)
                && verify_binop_secrecy(then_val)
                && verify_binop_secrecy(else_val)
        }
        ObliExpr::PubIf { cond, then_branch, else_branch } => {
            verify_binop_secrecy(cond)
                && verify_binop_secrecy(then_branch)
                && verify_binop_secrecy(else_branch)
        }
        ObliExpr::Let { value, body, .. } => {
            verify_binop_secrecy(value) && verify_binop_secrecy(body)
        }
        _ => true,
    }
}

// ============================================================================
// Property 1: No Secret Branching (VC-1, VC-2)
// ============================================================================

mod property_no_secret_branching {
    use super::*;

    #[test]
    fn public_if_has_public_condition() {
        let ir = transform("if true then 1 else 0");
        assert!(
            !contains_secret_pub_if(&ir),
            "PubIf must not have secret condition"
        );
    }

    #[test]
    fn secret_condition_uses_ct_select() {
        let ir = transform("let x = secret(1) if x > 0 then secret(1) else secret(0)");
        assert!(
            !contains_secret_pub_if(&ir),
            "Secret condition must not use PubIf"
        );
        assert!(
            contains_ct_select(&ir),
            "Secret condition must use CtSelect"
        );
    }

    #[test]
    fn nested_secret_condition_uses_ct_select() {
        let ir = transform(
            "let x = secret(1) \
             if x > 0 then \
               if x > 5 then secret(2) else secret(1) \
             else secret(0)"
        );
        assert!(
            !contains_secret_pub_if(&ir),
            "Nested secret conditions must not use PubIf"
        );
    }

    #[test]
    fn public_condition_uses_pub_if() {
        let ir = transform("let x = 1 if x > 0 then 1 else 0");
        assert!(
            contains_pub_if(&ir),
            "Public condition should use PubIf"
        );
        assert!(
            !contains_ct_select(&ir),
            "Public condition should not use CtSelect"
        );
    }

    #[test]
    fn mixed_conditions() {
        // Outer public, inner secret
        let ir = transform(
            "let pub_x = 1 \
             let sec_y = secret(2) \
             if pub_x > 0 then \
               if sec_y > 0 then secret(1) else secret(0) \
             else secret(0)"
        );
        assert!(
            !contains_secret_pub_if(&ir),
            "No secret PubIf allowed"
        );
        // Should have both: PubIf for outer, CtSelect for inner
        assert!(contains_pub_if(&ir), "Should have PubIf for public condition");
        assert!(contains_ct_select(&ir), "Should have CtSelect for secret condition");
    }
}

// ============================================================================
// Property 2: Secrecy Propagation (VC-5)
// ============================================================================

mod property_secrecy_propagation {
    use super::*;

    #[test]
    fn public_plus_public_is_public() {
        let ir = transform("1 + 2");
        assert!(!ir.is_secret(), "Pub + Pub = Pub");
    }

    #[test]
    fn secret_plus_public_is_secret() {
        let ir = transform("secret(1) + 2");
        assert!(ir.is_secret(), "Secret + Pub = Secret");
    }

    #[test]
    fn public_plus_secret_is_secret() {
        let ir = transform("1 + secret(2)");
        assert!(ir.is_secret(), "Pub + Secret = Secret");
    }

    #[test]
    fn secret_plus_secret_is_secret() {
        let ir = transform("secret(1) + secret(2)");
        assert!(ir.is_secret(), "Secret + Secret = Secret");
    }

    #[test]
    fn chained_operations_propagate() {
        let ir = transform("1 + 2 + secret(3) + 4");
        assert!(ir.is_secret(), "Any secret in chain makes result secret");
    }

    #[test]
    fn comparison_with_secret_is_secret() {
        let ir = transform("secret(1) > 0");
        assert!(ir.is_secret(), "Secret > Pub = Secret<bool>");
    }

    #[test]
    fn binop_secrecy_flags_correct() {
        let ir = transform("secret(1) + 2 * 3");
        assert!(
            verify_binop_secrecy(&ir),
            "All BinOp is_secret flags must match computed secrecy"
        );
    }

    #[test]
    fn complex_expression_secrecy_flags() {
        let ir = transform("(secret(1) + 2) * (3 - 4) + 5");
        assert!(
            verify_binop_secrecy(&ir),
            "Complex expression secrecy flags must be consistent"
        );
    }
}

// ============================================================================
// Property 3: Variable Secrecy Tracking (VC-4)
// ============================================================================

mod property_variable_secrecy {
    use super::*;

    #[test]
    fn let_public_creates_public_var() {
        let ir = transform("let x = 1 x");
        match ir {
            ObliExpr::Let { is_secret, body, .. } => {
                assert!(!is_secret, "let x = 1 should not be secret");
                match *body {
                    ObliExpr::Var { is_secret, .. } => {
                        assert!(!is_secret, "x should not be secret");
                    }
                    _ => panic!("Expected Var in body"),
                }
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn let_secret_creates_secret_var() {
        let ir = transform("let x = secret(1) x");
        match ir {
            ObliExpr::Let { is_secret, body, .. } => {
                assert!(is_secret, "let x = secret(1) should be secret");
                match *body {
                    ObliExpr::Var { is_secret, .. } => {
                        assert!(is_secret, "x should be secret");
                    }
                    _ => panic!("Expected Var in body"),
                }
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn secret_var_in_expression_propagates() {
        let ir = transform("let x = secret(1) x + 1");
        assert!(ir.is_secret(), "Expression using secret var is secret");
    }

    #[test]
    fn public_var_in_expression_stays_public() {
        let ir = transform("let x = 1 x + 1");
        assert!(!ir.is_secret(), "Expression using public var is public");
    }

    #[test]
    fn nested_let_secrecy() {
        let ir = transform("let x = 1 let y = secret(2) x + y");
        assert!(
            ir.is_secret(),
            "Expression with any secret var is secret"
        );
    }
}

// ============================================================================
// Property 4: CtSelect Semantics
// ============================================================================

mod property_ct_select {
    use super::*;

    #[test]
    fn ct_select_is_always_secret() {
        let ir = transform("let x = secret(true) if x then 1 else 0");
        match ir {
            ObliExpr::Let { body, .. } => {
                assert!(
                    body.is_secret(),
                    "CtSelect result is always secret"
                );
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn ct_select_structure() {
        let ir = transform("let c = secret(true) if c then secret(1) else secret(0)");
        match ir {
            ObliExpr::Let { body, .. } => match *body {
                ObliExpr::CtSelect { cond, then_val, else_val } => {
                    assert!(cond.is_secret(), "CtSelect condition is secret");
                    // then_val and else_val should exist (both evaluated)
                    assert!(matches!(*then_val, ObliExpr::SecretInt(1)));
                    assert!(matches!(*else_val, ObliExpr::SecretInt(0)));
                }
                _ => panic!("Expected CtSelect"),
            },
            _ => panic!("Expected Let"),
        }
    }
}

// ============================================================================
// Property 5: Literal Types
// ============================================================================

mod property_literals {
    use super::*;

    #[test]
    fn int_literal_is_pub() {
        let ir = transform("42");
        assert!(matches!(ir, ObliExpr::PubInt(42)));
        assert!(!ir.is_secret());
    }

    #[test]
    fn bool_literal_is_pub() {
        let ir = transform("true");
        assert!(matches!(ir, ObliExpr::PubBool(true)));
        assert!(!ir.is_secret());
    }

    #[test]
    fn secret_int_is_secret() {
        let ir = transform("secret(42)");
        assert!(matches!(ir, ObliExpr::SecretInt(42)));
        assert!(ir.is_secret());
    }

    #[test]
    fn secret_bool_is_secret() {
        let ir = transform("secret(true)");
        assert!(matches!(ir, ObliExpr::SecretBool(true)));
        assert!(ir.is_secret());
    }
}

// ============================================================================
// Property 6: Operator Transformation
// ============================================================================

mod property_operators {
    use super::*;

    #[test]
    fn arithmetic_ops_transform() {
        let test_cases = [
            ("1 + 2", ObliBinOp::CtAdd),
            ("1 - 2", ObliBinOp::CtSub),
            ("1 * 2", ObliBinOp::CtMul),
            ("1 / 2", ObliBinOp::CtDiv),
            ("1 % 2", ObliBinOp::CtMod),
        ];

        for (input, expected_op) in test_cases {
            let ir = transform(input);
            match ir {
                ObliExpr::BinOp { op, .. } => {
                    assert_eq!(op, expected_op, "Failed for: {}", input);
                }
                _ => panic!("Expected BinOp for: {}", input),
            }
        }
    }

    #[test]
    fn comparison_ops_transform() {
        let test_cases = [
            ("1 == 2", ObliBinOp::CtEq),
            ("1 != 2", ObliBinOp::CtNe),
            ("1 < 2", ObliBinOp::CtLt),
            ("1 <= 2", ObliBinOp::CtLe),
            ("1 > 2", ObliBinOp::CtGt),
            ("1 >= 2", ObliBinOp::CtGe),
        ];

        for (input, expected_op) in test_cases {
            let ir = transform(input);
            match ir {
                ObliExpr::BinOp { op, .. } => {
                    assert_eq!(op, expected_op, "Failed for: {}", input);
                }
                _ => panic!("Expected BinOp for: {}", input),
            }
        }
    }

    #[test]
    fn logical_ops_transform() {
        let test_cases = [
            ("true and false", ObliBinOp::CtAnd),
            ("true or false", ObliBinOp::CtOr),
        ];

        for (input, expected_op) in test_cases {
            let ir = transform(input);
            match ir {
                ObliExpr::BinOp { op, .. } => {
                    assert_eq!(op, expected_op, "Failed for: {}", input);
                }
                _ => panic!("Expected BinOp for: {}", input),
            }
        }
    }

    #[test]
    fn unary_ops_transform() {
        let ir = transform("-1");
        match ir {
            ObliExpr::UnaryOp { op: ObliUnaryOp::CtNeg, .. } => {}
            _ => panic!("Expected UnaryOp CtNeg"),
        }

        let ir = transform("not true");
        match ir {
            ObliExpr::UnaryOp { op: ObliUnaryOp::CtNot, .. } => {}
            _ => panic!("Expected UnaryOp CtNot"),
        }
    }
}

// ============================================================================
// Regression Tests
// ============================================================================

mod regression {
    use super::*;

    #[test]
    fn deeply_nested_secret_propagation() {
        let ir = transform(
            "let a = secret(1) \
             let b = a + 1 \
             let c = b * 2 \
             let d = c - 3 \
             d"
        );
        assert!(ir.is_secret(), "Secrecy must propagate through chain");
    }

    #[test]
    fn complex_conditional_nesting() {
        let ir = transform(
            "let s = secret(5) \
             let p = 10 \
             if p > 0 then \
               if s > 0 then secret(1) else secret(0) \
             else \
               secret(0)"
        );
        assert!(!contains_secret_pub_if(&ir), "No secret PubIf");
    }

    #[test]
    fn expression_in_condition() {
        let ir = transform(
            "let x = secret(1) \
             let y = secret(2) \
             if (x + y) > 0 then secret(1) else secret(0)"
        );
        assert!(contains_ct_select(&ir), "Complex secret condition uses CtSelect");
        assert!(!contains_secret_pub_if(&ir), "No secret PubIf");
    }
}
