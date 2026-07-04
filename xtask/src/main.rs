//! Developer task runner (action A2).
//!
//! Promotes the traceability gate and the CI step sequence behind `cargo xtask`,
//! so contributors and CI share one entry point. The traceability gate itself
//! stays language-agnostic: `scripts/trace-gate.py` remains the single source of
//! truth for R3/R4 and is invoked here rather than reimplemented.
//!
//! Usage:
//!   cargo xtask trace-gate   # requirement -> test coverage gate (R3/R4)
//!   cargo xtask ci           # trace-gate + fmt check + clippy + tests

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let ok = match task.as_str() {
        "trace-gate" => trace_gate(),
        "ci" => trace_gate() && fmt_check() && clippy() && test(),
        other => {
            eprintln!("unknown task {other:?}\n\nUsage: cargo xtask <trace-gate|ci>");
            false
        }
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Run a command, echoing it first, and report whether it succeeded.
fn run(program: &str, args: &[&str]) -> bool {
    eprintln!("[xtask] {program} {}", args.join(" "));
    Command::new(program)
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}

fn trace_gate() -> bool {
    run("python3", &["scripts/trace-gate.py"])
}

fn fmt_check() -> bool {
    run("cargo", &["fmt", "--all", "--", "--check"])
}

/// Clippy at pedantic (via workspace lints) promoted to errors (NFR-MAINT-01).
fn clippy() -> bool {
    run(
        "cargo",
        &["clippy", "--all-targets", "--", "-D", "warnings"],
    )
}

fn test() -> bool {
    run("cargo", &["test", "--workspace"])
}
