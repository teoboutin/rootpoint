//! The files a session reads and writes, through its host: the session opens no file itself,
//! since the browser build links it and `std::fs` fails there at run time
//! (`design@rootpoint-protocol@session-files`). A workspace's files are the host's, under its root;
//! a presentation held in memory has its files in the session's own map, and reads the host's
//! only to import a file or to copy a directory in.

/// Reads and writes whole files by path. The headless binary implements it on the native file
/// system; another host, such as the browser build, implements it its own way.
pub trait Files {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String>;
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String>;
    /// Whether anything is at the path, a file, a directory or a symlink, whatever its target: an
    /// error when the host cannot tell. A write to a symlink would follow it, so the store's check
    /// takes a symlink whose target is missing for one there, which `canonical` then refuses.
    fn exists(&mut self, path: &str) -> Result<bool, String>;
    /// The path of an existing file or directory with every symlink resolved, absolute natively;
    /// a host with no symlinks gives the path itself. The store compares it with its root, so that
    /// no access reaches outside the root, and stores a workspace's root in this form.
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Err(format!("{path}: unsupported"))
    }
    /// Every file below the directory, at any depth, as a path relative to it written with `/`, a
    /// symlink included as a file; none for a directory that does not exist.
    fn list(&mut self, dir: &str) -> Result<Vec<String>, String> {
        Err(format!("{dir}: unsupported"))
    }
}
