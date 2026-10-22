//! Files held in memory, by path: the file system of the browser build, which holds the files of a
//! directory the author picked and the images added since, and writes none, since a browser
//! cannot write the files beside a presentation. It is built on every target, so the native tests
//! check the actions that read it.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use rootpoint_protocol::Files;

/// The files, shared between the session's reads and the actions that put picked files in.
#[derive(Clone, Default)]
pub struct MemoryFiles(Rc<RefCell<BTreeMap<String, Vec<u8>>>>);

impl MemoryFiles {
    /// Hold these files, and no other.
    pub(crate) fn replace(&self, files: BTreeMap<String, Vec<u8>>) {
        *self.0.borrow_mut() = files;
    }

    pub(crate) fn insert(&self, path: String, bytes: Vec<u8>) {
        self.0.borrow_mut().insert(path, bytes);
    }

    pub(crate) fn get(&self, path: &str) -> Option<Vec<u8>> {
        self.0.borrow().get(path).cloned()
    }
}

impl Files for MemoryFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.get(path).ok_or(format!("{path}: no such file"))
    }

    fn write(&mut self, path: &str, _: &[u8]) -> Result<(), String> {
        Err(format!("{path}: the browser build saves no file"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.0.borrow().contains_key(path))
    }
}
