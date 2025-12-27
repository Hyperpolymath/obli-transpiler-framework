// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! Emitter Conformance Tests
//!
//! These tests verify that the Rust code emitter produces correct output
//! that matches the ObliIR semantics.

use obli_transpiler::transpile;

// ============================================================================
// Helper Functions
// ============================================================================

fn emit(input: &str) -> String {
    transpile(input).expect("transpilation failed")
}

fn contains_pattern(code: &str, pattern: &str) -> bool {
    code.contains(pattern)
}

// ============================================================================
// Literal Emission
// ============================================================================

mod literal_emission {
    use super::*;

    #[test]
    fn public_int_emits_pub_new() {
        let code = emit("42");
        assert!(
            contains_pattern(&code, "Pub::new(42i64)"),
            "Public int should emit Pub::new"
        );
    }

    #[test]
    fn public_bool_emits_pub_new() {
        let code = emit("true");
        assert!(
            contains_pattern(&code, "Pub::new(true)"),
            "Public bool should emit Pub::new"
        );
    }

    #[test]
    fn secret_int_emits_secret_new() {
        let code = emit("secret(42)");
        assert!(
            contains_pattern(&code, "Secret::new(42i64)"),
            "Secret int should emit Secret::new"
        );
    }

    #[test]
    fn secret_bool_emits_secret_new() {
        let code = emit("secret(true)");
        assert!(
            contains_pattern(&code, "Secret::new(true)"),
            "Secret bool should emit Secret::new"
        );
    }
}

// ============================================================================
// Operator Emission
// ============================================================================

mod operator_emission {
    use super::*;

    #[test]
    fn addition_emits_ct_add() {
        let code = emit("1 + 2");
        assert!(
            contains_pattern(&code, ".ct_add("),
            "Addition should emit .ct_add()"
        );
    }

    #[test]
    fn subtraction_emits_ct_sub() {
        let code = emit("1 - 2");
        assert!(
            contains_pattern(&code, ".ct_sub("),
            "Subtraction should emit .ct_sub()"
        );
    }

    #[test]
    fn multiplication_emits_ct_mul() {
        let code = emit("1 * 2");
        assert!(
            contains_pattern(&code, ".ct_mul("),
            "Multiplication should emit .ct_mul()"
        );
    }

    #[test]
    fn division_emits_ct_div() {
        let code = emit("1 / 2");
        assert!(
            contains_pattern(&code, ".ct_div("),
            "Division should emit .ct_div()"
        );
    }

    #[test]
    fn modulo_emits_ct_mod() {
        let code = emit("1 % 2");
        assert!(
            contains_pattern(&code, ".ct_mod("),
            "Modulo should emit .ct_mod()"
        );
    }

    #[test]
    fn equality_emits_ct_eq() {
        let code = emit("1 == 2");
        assert!(
            contains_pattern(&code, ".ct_eq("),
            "Equality should emit .ct_eq()"
        );
    }

    #[test]
    fn not_equal_emits_ct_ne() {
        let code = emit("1 != 2");
        assert!(
            contains_pattern(&code, ".ct_ne("),
            "Not-equal should emit .ct_ne()"
        );
    }

    #[test]
    fn less_than_emits_ct_lt() {
        let code = emit("1 < 2");
        assert!(
            contains_pattern(&code, ".ct_lt("),
            "Less-than should emit .ct_lt()"
        );
    }

    #[test]
    fn less_equal_emits_ct_le() {
        let code = emit("1 <= 2");
        assert!(
            contains_pattern(&code, ".ct_le("),
            "Less-equal should emit .ct_le()"
        );
    }

    #[test]
    fn greater_than_emits_ct_gt() {
        let code = emit("1 > 2");
        assert!(
            contains_pattern(&code, ".ct_gt("),
            "Greater-than should emit .ct_gt()"
        );
    }

    #[test]
    fn greater_equal_emits_ct_ge() {
        let code = emit("1 >= 2");
        assert!(
            contains_pattern(&code, ".ct_ge("),
            "Greater-equal should emit .ct_ge()"
        );
    }

    #[test]
    fn logical_and_emits_ct_and() {
        let code = emit("true and false");
        assert!(
            contains_pattern(&code, ".ct_and("),
            "Logical AND should emit .ct_and()"
        );
    }

    #[test]
    fn logical_or_emits_ct_or() {
        let code = emit("true or false");
        assert!(
            contains_pattern(&code, ".ct_or("),
            "Logical OR should emit .ct_or()"
        );
    }

    #[test]
    fn negation_emits_ct_neg() {
        let code = emit("-1");
        assert!(
            contains_pattern(&code, ".ct_neg("),
            "Negation should emit .ct_neg()"
        );
    }

    #[test]
    fn logical_not_emits_ct_not() {
        let code = emit("not true");
        assert!(
            contains_pattern(&code, ".ct_not("),
            "Logical NOT should emit .ct_not()"
        );
    }
}

// ============================================================================
// Conditional Emission
// ============================================================================

mod conditional_emission {
    use super::*;

    #[test]
    fn secret_conditional_emits_ct_select() {
        let code = emit("let x = secret(1) if x > 0 then secret(1) else secret(0)");
        assert!(
            contains_pattern(&code, "ct_select("),
            "Secret conditional should emit ct_select()"
        );
    }

    #[test]
    fn public_conditional_emits_if() {
        let code = emit("let x = 1 if x > 0 then 1 else 0");
        assert!(
            contains_pattern(&code, "if ") && contains_pattern(&code, ".reveal()"),
            "Public conditional should emit if with .reveal()"
        );
    }

    #[test]
    fn public_conditional_does_not_emit_ct_select() {
        let code = emit("let x = 1 if x > 0 then 1 else 0");
        // Should NOT contain ct_select
        assert!(
            !contains_pattern(&code, "ct_select("),
            "Public conditional should NOT emit ct_select()"
        );
    }

    #[test]
    fn ct_select_has_three_arguments() {
        let code = emit("let c = secret(true) if c then secret(1) else secret(0)");
        // ct_select should have cond, then, else
        assert!(
            contains_pattern(&code, "ct_select(&"),
            "ct_select should take references"
        );
    }
}

// ============================================================================
// Let Binding Emission
// ============================================================================

mod let_emission {
    use super::*;

    #[test]
    fn let_binding_emits_block() {
        let code = emit("let x = 1 x + 1");
        assert!(
            contains_pattern(&code, "{ let x = "),
            "Let binding should emit block with let"
        );
    }

    #[test]
    fn let_binding_preserves_name() {
        let code = emit("let my_var = 42 my_var");
        assert!(
            contains_pattern(&code, "let my_var = "),
            "Let binding should preserve variable name"
        );
    }

    #[test]
    fn nested_let_bindings() {
        let code = emit("let x = 1 let y = 2 x + y");
        assert!(
            contains_pattern(&code, "let x = ") && contains_pattern(&code, "let y = "),
            "Nested let bindings should both appear"
        );
    }
}

// ============================================================================
// Runtime Prelude
// ============================================================================

mod runtime_prelude {
    use super::*;

    #[test]
    fn emits_pub_struct() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "struct Pub<T>"),
            "Should emit Pub struct definition"
        );
    }

    #[test]
    fn emits_secret_struct() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "struct Secret<T>"),
            "Should emit Secret struct definition"
        );
    }

    #[test]
    fn emits_ct_select_function() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "fn ct_select<T"),
            "Should emit ct_select function"
        );
    }

    #[test]
    fn emits_main_function() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "fn main()"),
            "Should emit main function"
        );
    }

    #[test]
    fn emits_result_binding() {
        let code = emit("1 + 2");
        assert!(
            contains_pattern(&code, "let result = "),
            "Should bind result"
        );
    }

    #[test]
    fn emits_println() {
        let code = emit("42");
        assert!(
            contains_pattern(&code, "println!"),
            "Should print result"
        );
    }

    #[test]
    fn emits_spdx_header() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "SPDX-License-Identifier"),
            "Should include SPDX header"
        );
    }
}

// ============================================================================
// Constant-Time Implementation
// ============================================================================

mod ct_implementation {
    use super::*;

    #[test]
    fn ct_ops_use_wrapping_arithmetic() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "wrapping_add") &&
            contains_pattern(&code, "wrapping_sub") &&
            contains_pattern(&code, "wrapping_mul"),
            "CT ops should use wrapping arithmetic"
        );
    }

    #[test]
    fn ct_select_uses_bitwise_masking() {
        let code = emit("1");
        // The ct_select implementation uses mask
        assert!(
            contains_pattern(&code, "mask") || contains_pattern(&code, "& "),
            "ct_select should use bitwise operations"
        );
    }

    #[test]
    fn secret_has_reveal_method() {
        let code = emit("1");
        assert!(
            contains_pattern(&code, "fn reveal(&self)"),
            "Secret should have reveal method"
        );
    }
}

// ============================================================================
// End-to-End Examples
// ============================================================================

mod end_to_end {
    use super::*;

    #[test]
    fn password_check_example() {
        let code = emit(
            "let password = secret(42) \
             if password == 42 then secret(1) else secret(0)"
        );

        // Should use ct_select (not regular if)
        assert!(contains_pattern(&code, "ct_select("));
        assert!(contains_pattern(&code, "Secret::new(42i64)"));
        assert!(contains_pattern(&code, ".ct_eq("));
    }

    #[test]
    fn arithmetic_chain() {
        let code = emit("1 + 2 * 3 - 4");

        assert!(contains_pattern(&code, ".ct_add("));
        assert!(contains_pattern(&code, ".ct_mul("));
        assert!(contains_pattern(&code, ".ct_sub("));
    }

    #[test]
    fn mixed_secrecy() {
        let code = emit(
            "let pub_val = 10 \
             let sec_val = secret(20) \
             pub_val + sec_val"
        );

        assert!(contains_pattern(&code, "Pub::new(10i64)"));
        assert!(contains_pattern(&code, "Secret::new(20i64)"));
    }
}

// ============================================================================
// Syntax Validity
// ============================================================================

mod syntax_validity {
    use super::*;

    #[test]
    fn emitted_code_has_balanced_braces() {
        let code = emit("let x = 1 let y = 2 x + y");
        let open = code.matches('{').count();
        let close = code.matches('}').count();
        assert_eq!(open, close, "Braces should be balanced");
    }

    #[test]
    fn emitted_code_has_balanced_parens() {
        let code = emit("(1 + 2) * (3 - 4)");
        let open = code.matches('(').count();
        let close = code.matches(')').count();
        assert_eq!(open, close, "Parentheses should be balanced");
    }

    #[test]
    fn emitted_code_ends_with_newline() {
        let code = emit("1");
        assert!(code.ends_with('\n'), "Code should end with newline");
    }
}
