//! Files held in memory, by path: the file system of the browser build, which holds the files of a
//! directory the author picked and the images picked since, by path, which `open-copy` and `import`
//! copy into the session's map, per `design@rootpoint-app@picked-files-by-path`, and writes none,
//! per `design@rootpoint-app@browser-load-only`. It is built on every target, so the native tests
//! check the actions that read it.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use rootpoint_protocol::Files;

/// The files, shared between the session's reads and the actions that put picked files in.
#[derive(Clone, Default)]
pub struct MemoryFiles(Rc<RefCell<BTreeMap<String, Vec<u8>>>>);

impl MemoryFiles {
    /// Hold these files, and no other, and give back the files held before.
    pub(crate) fn replace(&self, files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
        std::mem::replace(&mut *self.0.borrow_mut(), files)
    }

    pub(crate) fn insert(&self, path: String, bytes: Vec<u8>) {
        self.0.borrow_mut().insert(path, bytes);
    }
}

impl Files for MemoryFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }

    fn write(&mut self, path: &str, _: &[u8]) -> Result<(), String> {
        Err(format!("{path}: the browser build saves no file"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.0.borrow().contains_key(path))
    }

    /// Files held in memory have no symlinks: a path is its own canonical form.
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
}
