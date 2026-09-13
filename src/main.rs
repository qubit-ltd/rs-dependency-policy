// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Cargo subcommand entry point for dependency-policy operations.

use clap::Parser;
use qubit_dependency_policy::cli::Cli;

/// Parses command-line arguments and executes the selected policy operation.
fn main() {
    let mut arguments = std::env::args_os().collect::<Vec<_>>();
    if arguments
        .get(1)
        .is_some_and(|argument| argument == "dependency-policy")
    {
        arguments.remove(1);
    }
    let cli = Cli::parse_from(arguments);
    if let Err(error) = cli.execute() {
        eprintln!("{error}");
        std::process::exit(if matches!(error.code(), "DP001" | "DP101") {
            1
        } else {
            2
        });
    }
}
