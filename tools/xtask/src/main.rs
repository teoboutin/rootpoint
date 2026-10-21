use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use knowledge_architect::cli;
use knowledge_architect_gates::{Checker, GatesArgs, project_root, rust_project};

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
            knowledge_architect_gates::run(&root, &rust_project(checker, "origin/main"), &args)
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
