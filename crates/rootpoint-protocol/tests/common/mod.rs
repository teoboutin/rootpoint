//! Helpers of the protocol tests: files held in memory, and a session that runs a script.

#![allow(dead_code)]

use std::collections::BTreeMap;

use rootpoint_protocol::{Files, Reply, Session};

/// Files held in memory, by path.
#[derive(Default)]
pub struct MemoryFiles {
    pub files: BTreeMap<String, Vec<u8>>,
}

impl Files for MemoryFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.files.insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }
}

/// Run each line of a script, and return the replies, one per command.
pub fn run(session: &mut Session, files: &mut MemoryFiles, script: &str) -> Vec<Reply> {
    script
        .lines()
        .filter_map(|line| session.execute(line, files))
        .collect()
}

/// Run one line, which must succeed, and return its reply's text.
pub fn ok(session: &mut Session, files: &mut MemoryFiles, line: &str) -> String {
    match session.execute(line, files) {
        Some(Reply::Ok(text)) => text,
        other => panic!("`{line}` gave {other:?}"),
    }
}
