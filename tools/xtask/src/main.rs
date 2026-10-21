use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use knowledge_architect::cli;
use knowledge_architect_gates::{Checker, Distiller, Gate, GatesArgs, project_root, rust_project};

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run every gate a branch must pass before it merges.
    Gates(GatesArgs),
    /// The knowledge-architect checker, at the version this crate pins.
    Klarch {
        #[command(subcommand)]
        command: cli::Command,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Gates(args) => {
            let cwd = std::env::current_dir().expect("a working directory");
            let Some(root) = project_root(&cwd, knowledge_architect::MANIFEST_NAME) else {
                eprintln!("xtask: not inside the project");
                return ExitCode::FAILURE;
            };
            let checker = Checker {
                package: "xtask",
                prefix: &["klarch"],
            };
            let mut gates = rust_project(checker, "origin/main");
            gates.push(wasm_gate());
            knowledge_architect_gates::run(&root, &gates, &args)
        }
        Command::Klarch { command } => {
            let own = Path::new(env!("CARGO_MANIFEST_DIR"));
            let outcome = cli::locate().and_then(|manifest| {
                cli::refuse_a_foreign_build(
                    manifest.root(),
                    &[
                        cli::this_library(),
                        cli::Library {
                            crate_dir: own.to_path_buf(),
                            package: env!("CARGO_PKG_NAME"),
                        },
                    ],
                    &[env!("CARGO_PKG_NAME")],
                )?;
                cli::refuse_another_version(&manifest, &[own])?;
                cli::run(command, &manifest, &[own], &mut [])
            });
            outcome.unwrap_or_else(|e| {
                eprintln!("error: {e}");
                ExitCode::from(2)
            })
        }
    }
}

/// The crates the browser build links, each built for the wasm target. A crate the browser build
/// links is listed here when it is created, so a dependency that cannot build for the browser fails
/// the gates at once rather than at the browser build.
const WASM_CRATES: &[&str] = &["rootpoint-model", "rootpoint-protocol", "rootpoint-render"];

fn wasm_gate() -> Gate {
    let mut args = vec![
        "build".to_owned(),
        "--target".to_owned(),
        "wasm32-unknown-unknown".to_owned(),
    ];
    for krate in WASM_CRATES {
        args.push("-p".to_owned());
        args.push((*krate).to_owned());
    }
    Gate {
        name: "wasm".to_owned(),
        program: "cargo".to_owned(),
        args,
        envs: Vec::new(),
        resolves: true,
        rebased_only: false,
        distiller: Distiller::WithoutProgress,
    }
}
