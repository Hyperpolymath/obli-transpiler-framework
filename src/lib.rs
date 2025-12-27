// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! obli-transpiler: MiniObli → Rust (constant-time) transpiler
//!
//! A toy transpiler demonstrating oblivious program transformation.

pub mod ast;
pub mod emit;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod transform;

pub use ast::Expr;
pub use ir::ObliExpr;
pub use lexer::Lexer;
pub use parser::Parser;
pub use transform::to_oblivious;

/// Transpile MiniObli source code to oblivious Rust code.
pub fn transpile(source: &str) -> Result<String, String> {
    let lexer = Lexer::new(source);
    let tokens: Result<Vec<_>, _> = lexer.collect();
    let tokens = tokens.map_err(|e| e.to_string())?;

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    let obli_ir = to_oblivious(&ast);
    let rust_code = emit::emit_rust(&obli_ir);

    Ok(rust_code)
}
