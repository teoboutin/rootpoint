//! The watcher of a workspace's assets, per `design@rootpoint-app@asset-watch`: the directories that hold an asset of the table, each
//! watched alone, not recursively, so that a change of `.git/` or of a build tree is never seen and
//! the watches stay within inotify's limit of watches per user. A change of an asset's file wakes
//! the window, which decodes its assets again. Native only: in memory, and in the browser, no file
//! on disk is the presentation's.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use notify::event::ModifyKind;
use notify::{EventKind, RecursiveMode, Watcher};

/// The watches of one workspace, and the paths their events named since the window last took them.
pub struct Watch {
    watcher: notify::RecommendedWatcher,
    watched: BTreeSet<PathBuf>,
    changed: Arc<Mutex<Vec<PathBuf>>>,
}

impl Watch {
    /// A watcher with no watch yet, which wakes the window by `wake` at each event.
    pub fn new(wake: Arc<dyn Fn() + Send + Sync>) -> notify::Result<Watch> {
        let changed = Arc::new(Mutex::new(Vec::new()));
        let theirs = changed.clone();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            // Only an event that changes a file's bytes or its name counts: a read of the file,
            // as `git add` makes, or a change of its metadata, decodes nothing.
            let Ok(event) = event else { return };
            let changes = match event.kind {
                EventKind::Create(_) | EventKind::Remove(_) => true,
                EventKind::Modify(ModifyKind::Metadata(_)) => false,
                EventKind::Modify(_) => true,
                _ => false,
            };
            if changes {
                theirs.lock().unwrap().extend(event.paths);
                wake();
            }
        })?;
        Ok(Watch {
            watcher,
            watched: BTreeSet::new(),
            changed,
        })
    }

    /// Watch exactly these directories: the ones no longer named stop, and each one named is
    /// watched again, since a directory removed and made again lost its watch with the old one. A
    /// directory that cannot be watched, as one that does not exist, is left out.
    pub fn set(&mut self, dirs: BTreeSet<PathBuf>) {
        for gone in self.watched.difference(&dirs) {
            let _ = self.watcher.unwatch(gone);
        }
        self.watched = dirs
            .into_iter()
            .filter(|dir| self.watcher.watch(dir, RecursiveMode::NonRecursive).is_ok())
            .collect();
    }

    /// The paths the events named since the last call.
    pub fn take(&self) -> Vec<PathBuf> {
        std::mem::take(&mut *self.changed.lock().unwrap())
    }
}
