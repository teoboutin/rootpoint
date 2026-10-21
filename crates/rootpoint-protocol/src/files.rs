//! The files a session reads and writes, through its host: the session opens no file itself,
//! since the browser build links it and `std::fs` fails there at run time.

/// Reads and writes whole files by path. The headless binary implements it on the native file
/// system; another host, such as the browser build, implements it its own way.
pub trait Files {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String>;
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String>;
}
