//! The files a session reads and writes, through its host: the session opens no file itself,
//! since the browser build links it and `std::fs` fails there at run time
//! (`design@rootpoint-protocol@session-files`).

/// Reads and writes whole files by path. The headless binary implements it on the native file
/// system; another host, such as the browser build, implements it its own way.
pub trait Files {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String>;
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String>;
    /// Whether a file is at the path: an error when the host cannot tell, so that a save never
    /// takes a file it cannot read for a missing one, and writes over it.
    fn exists(&mut self, path: &str) -> Result<bool, String>;
}
