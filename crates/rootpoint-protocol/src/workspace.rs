//! What a native host adds to a workspace: the lock that keeps it to one process, the place of the
//! window's control socket, and the one function a host runs every command through. Native only:
//! the browser build opens no workspace. With the binaries and the client of the control socket,
//! it is the part of the crate that touches the file system; the session never calls it.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use crate::{Command, Files, Reply, Session};

/// The lock of a workspace, `design@rootpoint-protocol@workspace-lock`: `File::try_lock` held on
/// `.rootpoint/lock`, whose file holds the pid of
/// its holder, for the refusal's message. The kernel releases it when the process ends, so a crash
/// leaves no stale lock; it is released when the value is dropped.
pub struct WorkspaceLock {
    _file: File,
}

impl WorkspaceLock {
    /// Take the lock of the workspace at `dir`, creating its `.rootpoint/` and the `.gitignore`
    /// there, which ignores everything in it; refused while another holder has it, the reply
    /// naming that holder's pid.
    pub fn take(dir: &Path) -> Result<WorkspaceLock, String> {
        let machine = dir.join(".rootpoint");
        std::fs::create_dir_all(&machine).map_err(|e| format!("{}: {e}", machine.display()))?;
        let ignore = machine.join(".gitignore");
        if !ignore.exists() {
            std::fs::write(&ignore, "*\n").map_err(|e| format!("{}: {e}", ignore.display()))?;
        }
        let path = machine.join("lock");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                let mut pid = String::new();
                let _ = file.read_to_string(&mut pid);
                return Err(format!(
                    "{} is open in another process, pid {}",
                    dir.display(),
                    pid.trim()
                ));
            }
            Err(TryLockError::Error(e)) => return Err(format!("{}: {e}", path.display())),
        }
        file.set_len(0)
            .and_then(|()| file.rewind())
            .and_then(|()| write!(file, "{}", std::process::id()))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(WorkspaceLock { _file: file })
    }
}

/// Run one command as a native host runs every command: before `open <dir>` of a directory that
/// exists, and before `unpack <dir>` of a presentation held in memory, take the lock of `<dir>`,
/// or reply `err` naming the process that holds it; after a reply `ok` that changed the session's
/// root, hold the new lock and drop the old one. The lock of the workspace already open is kept,
/// since a second lock of one file is refused in one process as in another.
pub fn run(
    session: &mut Session,
    command: Command,
    files: &mut dyn Files,
    lock: &mut Option<WorkspaceLock>,
) -> Reply {
    let target = match &command {
        Command::Open { dir } if Path::new(dir).is_dir() => Some(dir.clone()),
        // A path that is a file gets the session's refusal, with no lock to take.
        Command::Unpack { dir } if session.root().is_none() && !Path::new(dir).is_file() => {
            Some(dir.clone())
        }
        _ => None,
    };
    let held = |dir: &str| {
        std::fs::canonicalize(dir)
            .ok()
            .is_some_and(|d| session.root().is_some_and(|root| Path::new(root) == d))
    };
    // The `.rootpoint/` a lock creates for a command that then fails leaves with it, so a refused
    // open or unpack leaves no trace in a directory that is no workspace.
    let mut made = None;
    let new = match target {
        Some(dir) if !held(&dir) => {
            let machine = Path::new(&dir).join(".rootpoint");
            if !machine.exists() {
                made = Some(machine);
            }
            match WorkspaceLock::take(Path::new(&dir)) {
                Ok(new) => Some(new),
                Err(reason) => {
                    if let Some(made) = made {
                        let _ = std::fs::remove_dir_all(made);
                    }
                    return Reply::Err(reason);
                }
            }
        }
        _ => None,
    };
    let before = session.root().map(str::to_owned);
    let reply = session.run(command, files);
    if matches!(reply, Reply::Ok(_)) && session.root() != before.as_deref() {
        *lock = new;
    } else if let Some(made) = made {
        drop(new);
        let _ = std::fs::remove_dir_all(made);
    }
    reply
}

/// The directory of the control sockets: `$XDG_RUNTIME_DIR/rootpoint/`, or `$TMPDIR/rootpoint/`,
/// then `/tmp/rootpoint-<uid>/`, where the variable is unset or relative. A host creates it with
/// mode 0700, `make_runtime_dir`, so no other user reaches a window through it.
pub fn runtime_dir() -> PathBuf {
    // A relative value would make the pointer file name a socket only one working directory finds.
    let var = |name| std::env::var_os(name).filter(|v| Path::new(v).is_absolute());
    if let Some(dir) = var("XDG_RUNTIME_DIR").or_else(|| var("TMPDIR")) {
        return PathBuf::from(dir).join("rootpoint");
    }
    #[cfg(unix)]
    // SAFETY: getuid has no precondition and cannot fail.
    let user = unsafe { libc::getuid() }.to_string();
    #[cfg(not(unix))]
    let user = "user";
    PathBuf::from(format!("/tmp/rootpoint-{user}"))
}

/// Create the runtime directory with mode 0700, or check the one there: it must be the user's
/// and reachable by no other user, since its sockets drive the windows. A directory another user
/// made first, as `/tmp/rootpoint-<uid>/` can be, is refused.
pub fn make_runtime_dir() -> std::io::Result<PathBuf> {
    let dir = runtime_dir();
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt};
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)?;
        let meta = std::fs::symlink_metadata(&dir)?;
        // SAFETY: getuid has no precondition and cannot fail.
        let uid = unsafe { libc::getuid() };
        if !meta.is_dir() || meta.uid() != uid || meta.mode() & 0o077 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "{} is not a directory of this user's alone, mode 0700",
                    dir.display()
                ),
            ));
        }
    }
    #[cfg(not(unix))]
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// The bytes a socket's path may hold: `sun_path` holds 108 on Linux and 104 on macOS, the closing
/// NUL included.
const SOCKET_PATH_MAX: usize = 103;

/// The file name of the window's control socket in the runtime directory `dir`: in a workspace,
/// `<name>-<hash>.sock`, the root's name truncated to 40 bytes, and further where the path would not
/// fit a socket's address, then the first 8 hexadecimal digits of the FNV-1a hash of the canonical
/// root, so two workspaces of one name have two sockets; in memory, `memory-<pid>.sock`.
pub fn socket_name(dir: &Path, root: Option<&str>) -> String {
    let Some(root) = root else {
        return format!("memory-{}.sock", std::process::id());
    };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in root.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    let hash = &format!("{hash:016x}")[..8];
    let name = Path::new(root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    // The directory, a `/`, the name, `-`, the hash and `.sock`.
    let room = SOCKET_PATH_MAX.saturating_sub(dir.as_os_str().len() + 1 + 1 + 8 + 5);
    let mut end = name.len().min(40).min(room);
    while !name.is_char_boundary(end) {
        end -= 1;
    }
    match &name[..end] {
        "" => format!("{hash}.sock"),
        name => format!("{name}-{hash}.sock"),
    }
}

/// The path of the window's control socket for a store: in the runtime directory, by
/// `socket_name`.
pub fn socket_path(root: Option<&str>) -> PathBuf {
    let dir = runtime_dir();
    dir.join(socket_name(&dir, root))
}

/// The pointer file of a workspace, `.rootpoint/socket`, which holds the path of the socket of the
/// window that holds it, so a client finds the window from the workspace.
pub fn pointer_file(root: &Path) -> PathBuf {
    root.join(".rootpoint").join("socket")
}
