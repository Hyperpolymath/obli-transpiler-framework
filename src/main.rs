// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! obli - MiniObli to Rust (constant-time) transpiler CLI

use clap::{Parser, Subcommand};
use obli_transpiler::transpile;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "obli")]
#[command(author, version, about = "Oblivious program transpiler: MiniObli → Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transpile a MiniObli file to Rust
    Transpile {
        /// Input .mobli file
        #[arg(short, long)]
        input: PathBuf,

        /// Output .rs file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run a MiniObli expression directly
    Run {
        /// Expression to evaluate
        #[arg(short, long)]
        expr: String,
    },
    /// Check a MiniObli file for errors without transpiling
    Check {
        /// Input .mobli file
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transpile { input, output } => {
            let source = match fs::read_to_string(&input) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            match transpile(&source) {
                Ok(rust_code) => {
                    if let Some(output_path) = output {
                        if let Err(e) = fs::write(&output_path, &rust_code) {
                            eprintln!("Error writing {}: {}", output_path.display(), e);
                            std::process::exit(1);
                        }
                        eprintln!("Wrote {}", output_path.display());
                    } else {
                        println!("{}", rust_code);
                    }
                }
                Err(e) => {
                    eprintln!("Transpilation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Run { expr } => {
            match transpile(&expr) {
                Ok(rust_code) => {
                    println!("// Generated Rust code:\n{}", rust_code);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Check { input } => {
            let source = match fs::read_to_string(&input) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            match transpile(&source) {
                Ok(_) => {
                    println!("{}: OK", input.display());
                }
                Err(e) => {
                    eprintln!("{}: Error: {}", input.display(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}
