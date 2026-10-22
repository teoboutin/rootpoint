use std::path::{Path, PathBuf};
use std::process::{Command as Process, ExitCode};

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
    /// Build the browser build of the editor into a static directory, `web/` under the target
    /// directory by default,
    /// which any static file server serves.
    Web {
        /// The directory to write, emptied first.
        #[arg(long)]
        out: Option<PathBuf>,
    },
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
            gates.push(wasm_test_gate());
            knowledge_architect_gates::run(&root, &gates, &args)
        }
        Command::Web { out } => {
            let cwd = std::env::current_dir().expect("a working directory");
            let Some(root) = project_root(&cwd, knowledge_architect::MANIFEST_NAME) else {
                eprintln!("xtask: not inside the project");
                return ExitCode::FAILURE;
            };
            match build_web(&root, out) {
                Ok(dir) => {
                    println!("{}", dir.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("xtask web: {e}");
                    ExitCode::FAILURE
                }
            }
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

/// The crates the browser build links, each built for the wasm target, so a dependency that cannot
/// build for the browser fails the gates at once rather than at the browser build.
const WASM_CRATES: &[&str] = &[
    "rootpoint-app",
    "rootpoint-model",
    "rootpoint-protocol",
    "rootpoint-render",
];

fn wasm_gate() -> Gate {
    let mut args = vec![
        "build".to_owned(),
        // The libraries, and the editor's binary, which is the browser build's entry point; the
        // other binaries, such as the headless runner of the protocol, are native programs, not
        // part of the browser build.
        "--lib".to_owned(),
        "--bin".to_owned(),
        "rootpoint".to_owned(),
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

/// The app's tests that run in a browser, for the browser build: its actions, the layout of a slide
/// against the native golden image, and F5 kept from the browser, which only a browser can refute,
/// as `goal@rootpoint-app@browser-window` asks the same actions there. `wasm-bindgen-test-runner`
/// runs them in a headless Firefox, through geckodriver, both found on the `PATH`.
fn wasm_test_gate() -> Gate {
    Gate {
        name: "wasm-test".to_owned(),
        program: "cargo".to_owned(),
        args: [
            "test",
            "--target",
            "wasm32-unknown-unknown",
            "-p",
            "rootpoint-app",
            "--test",
            "actions",
            "--test",
            "web",
        ]
        .map(str::to_owned)
        .to_vec(),
        envs: Vec::new(),
        resolves: true,
        rebased_only: false,
        distiller: Distiller::Tests,
    }
}

/// Build the editor's binary for the wasm target, bind it with `wasm-bindgen` for a page that
/// loads it as a module, and write it with the app's page into `out`. The `wasm-bindgen` program
/// must be the version of the `wasm-bindgen` crate in `Cargo.lock`, since the two share a format.
fn build_web(root: &Path, out: Option<PathBuf>) -> Result<PathBuf, String> {
    let target = target_directory(root)?;
    let out = out.unwrap_or_else(|| target.join("web"));
    let locked = locked_version(root, "wasm-bindgen")?;
    let found = Process::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|e| {
            format!(
                "wasm-bindgen: {e}; install it with `cargo install wasm-bindgen-cli --version {locked} --locked`"
            )
        })?;
    let found = String::from_utf8_lossy(&found.stdout);
    if found.split_whitespace().nth(1) != Some(locked.as_str()) {
        return Err(format!(
            "wasm-bindgen is {}, and Cargo.lock has {locked}; install it with `cargo install wasm-bindgen-cli --version {locked} --locked`",
            found.trim()
        ));
    }
    run(Process::new(env!("CARGO")).current_dir(root).args([
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "rootpoint-app",
        "--bin",
        "rootpoint",
    ]))?;
    let module = target.join("wasm32-unknown-unknown/release/rootpoint.wasm");
    if !module.exists() {
        return Err(format!("the build wrote no {}", module.display()));
    }
    if out.exists() {
        std::fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    run(Process::new("wasm-bindgen")
        .args(["--target", "web", "--no-typescript", "--out-dir"])
        .arg(&out)
        .arg(&module))?;
    let page = root.join("crates/rootpoint-app/web/index.html");
    std::fs::copy(&page, out.join("index.html")).map_err(|e| format!("{}: {e}", page.display()))?;
    Ok(out)
}

/// The directory cargo builds into, as `cargo metadata` gives it from the root: `CARGO_TARGET_DIR`,
/// `CARGO_BUILD_TARGET_DIR` and `build.target-dir` can each move it, and a relative one is read
/// against the directory cargo runs in, so a guess could take a stale module from another one.
fn target_directory(root: &Path) -> Result<PathBuf, String> {
    let output = Process::new(env!("CARGO"))
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed: {}", output.status));
    }
    let json = String::from_utf8_lossy(&output.stdout);
    // A JSON string: the path ends at the first quote no backslash escapes.
    let key = "\"target_directory\":\"";
    let start = json
        .find(key)
        .map(|i| i + key.len())
        .ok_or("cargo metadata gave no target_directory")?;
    let mut path = String::new();
    let mut chars = json[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Ok(PathBuf::from(path)),
            '\\' => path.extend(chars.next()),
            c => path.push(c),
        }
    }
    Err("cargo metadata's target_directory has no end".to_owned())
}

/// The version of a package in `Cargo.lock`, which holds it once.
fn locked_version(root: &Path, package: &str) -> Result<String, String> {
    let lock =
        std::fs::read_to_string(root.join("Cargo.lock")).map_err(|e| format!("Cargo.lock: {e}"))?;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == format!("name = \"{package}\"")
            && let Some(version) = lines
                .next()
                .and_then(|l| l.strip_prefix("version = \""))
                .and_then(|l| l.strip_suffix('"'))
        {
            return Ok(version.to_owned());
        }
    }
    Err(format!("Cargo.lock holds no {package}"))
}

fn run(command: &mut Process) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|e| format!("{:?}: {e}", command.get_program()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{:?} failed: {status}", command.get_program()))
    }
}
