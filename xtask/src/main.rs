//! Workspace task runner. Keeps build/dev orchestration in Cargo instead of a
//! Makefile so `cargo xtask <task>` is the single entry point for both the Rust
//! workspace and the `culinator-desktop` frontend.
//!
//! The `cargo xtask` alias is defined in `.cargo/config.toml`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

/// Frontend package directory, relative to the workspace root.
const FRONTEND_DIR: &str = "culinator-desktop";

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Build and dev tasks for the culinator workspace"
)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Install frontend dependencies (npm ci).
    Setup,
    /// Format Rust and frontend sources.
    Format,
    /// Verify formatting without writing changes.
    FormatCheck,
    /// Lint Rust (clippy, warnings denied) and the frontend.
    Lint,
    /// Run the full Rust test suite.
    Test,
    /// Type-check the frontend.
    Typecheck,
    /// Build the Rust workspace and the frontend.
    Build,
    /// Run format-check, lint, tests, typecheck, and the frontend build.
    Check,
    /// Launch the Tauri desktop app in dev mode.
    DesktopDev,
    /// Launch the development HTTP service with Vite.
    ServiceDev,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = project_root();

    match cli.task {
        Task::Setup => npm(&["ci"], &root),
        Task::Format => format(&root),
        Task::FormatCheck => format_check(&root),
        Task::Lint => lint(&root),
        Task::Test => test(&root),
        Task::Typecheck => typecheck(&root),
        Task::Build => build(&root),
        Task::Check => check(&root),
        Task::DesktopDev => npm(&["run", "tauri", "dev"], &root),
        Task::ServiceDev => npm(&["run", "dev:service"], &root),
    }
}

fn format(root: &Path) -> Result<()> {
    cargo(&["fmt", "--all"], root)?;
    npm(&["run", "format"], root)
}

fn format_check(root: &Path) -> Result<()> {
    cargo(&["fmt", "--all", "--", "--check"], root)?;
    npm(&["run", "format:check"], root)
}

fn lint(root: &Path) -> Result<()> {
    cargo(
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        root,
    )?;
    npm(&["run", "lint"], root)
}

fn test(root: &Path) -> Result<()> {
    cargo(&["test", "--workspace", "--all-targets"], root)
}

fn typecheck(root: &Path) -> Result<()> {
    npm(&["run", "typecheck"], root)
}

fn build(root: &Path) -> Result<()> {
    cargo(&["build", "--workspace"], root)?;
    npm(&["run", "build"], root)
}

fn check(root: &Path) -> Result<()> {
    format_check(root)?;
    lint(root)?;
    test(root)?;
    typecheck(root)?;
    npm(&["run", "build"], root)
}

/// Workspace root, derived from this crate's manifest directory (`<root>/xtask`).
fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask crate is nested under the workspace root")
        .to_path_buf()
}

fn cargo(args: &[&str], root: &Path) -> Result<()> {
    run("cargo", args, root)
}

fn npm(args: &[&str], root: &Path) -> Result<()> {
    run("npm", args, &root.join(FRONTEND_DIR))
}

fn run(program: &str, args: &[&str], dir: &Path) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("failed to spawn `{program}` (is it installed and on PATH?)"))?;
    if !status.success() {
        bail!("`{program} {}` failed ({status})", args.join(" "));
    }
    Ok(())
}
