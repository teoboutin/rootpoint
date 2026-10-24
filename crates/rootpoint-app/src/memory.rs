//! Files held in memory, by path: the file system of the browser build, which holds the files of a
//! directory the author picked, the images and the packs picked since, by path, which `open-copy`,
//! `import` and `open-pack` copy into the session's map, per
//! `design@rootpoint-app@picked-files-by-path`, and the pack the session writes, which the page
//! downloads. It moves and removes no file, since the browser build holds no workspace,
//! `design@rootpoint-app@browser-pack-save`. It is built on every target, so the native tests check
//! the actions that read it.

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

    /// Hold these bytes at the path, or nothing, and give back what was held there before.
    pub(crate) fn replace_one(&self, path: &str, bytes: Option<Vec<u8>>) -> Option<Vec<u8>> {
        let mut files = self.0.borrow_mut();
        match bytes {
            Some(bytes) => files.insert(path.to_owned(), bytes),
            None => files.remove(path),
        }
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

    /// Keeps the bytes under the path: a pack the browser build writes there, which the page then
    /// downloads, `App::pack_for_download`.
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.0.borrow_mut().insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.0.borrow().contains_key(path))
    }

    /// Files held in memory have no symlinks: a path is its own canonical form.
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
    fn list(&mut self, dir: &str) -> Result<Vec<String>, String> {
        if self.0.borrow().contains_key(dir) {
            return Err(format!("{dir}: not a directory"));
        }
        let below = format!("{dir}/");
        Ok(self
            .0
            .borrow()
            .keys()
            .filter_map(|k| k.strip_prefix(&below).map(str::to_owned))
            .collect())
    }

    fn rename(&mut self, from: &str, _: &str) -> Result<(), String> {
        Err(format!("{from}: the browser build holds no workspace"))
    }

    fn remove(&mut self, path: &str) -> Result<(), String> {
        Err(format!("{path}: the browser build holds no workspace"))
    }
}
