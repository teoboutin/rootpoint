//! The autosave of the native window's workspace: a thread that writes `presentation.ron` after
//! every undo step, and its guard, which writes nothing over a file changed outside rootpoint. The
//! GUI thread pays only for a clone of the document; the thread serializes it, compares the file
//! with what it last wrote or read, and writes it through a temporary file and a rename, per
//! `design@rootpoint-app@workspace-autosave` and `design@rootpoint-app@ron-overwrite-guard`. It
//! reads no session and applies no change, so it is no action of
//! `design@rootpoint-app@actions-are-commands`; neither is the banner's "Keep mine", which hands it
//! the window's document and changes no session.
//! Native only: the browser build has no thread and no workspace.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use rootpoint_model::{Document, save_ron};

/// What the thread is asked to do next. Only the newest job waits: a newer clone replaces an older
/// one that was not written yet.
enum Job {
    /// Write the document, unless the guard finds the file changed.
    Write(Box<Document>),
    /// Write the document whatever the file holds, and resume: "Keep mine".
    Force(Box<Document>),
    /// Take the file as it is now as the baseline, resume, and write the document: after a
    /// `reload`, whose document is the file's. The file is read again when the job runs, so a
    /// write between the `reload`'s read and this one is not seen, a window as short as the one
    /// before a rename.
    Resume(Box<Document>),
}

#[derive(Default)]
struct State {
    job: Option<Job>,
    /// A job is being done: the slot is empty, and the file may still change.
    busy: bool,
    /// The tests keep the next job waiting.
    held: bool,
    /// The guard found the file changed outside rootpoint: writes are dropped until a `Force` or a
    /// `Resume`.
    stopped: bool,
    writes: u64,
    /// Why the last write failed, until one succeeds: the window's status line names it.
    failure: Option<String>,
    quit: bool,
}

/// The thread of one workspace's autosave, from the open of the workspace until the store leaves
/// it. Dropping it writes the job still waiting, then ends the thread.
pub struct Autosave {
    shared: Arc<(Mutex<State>, Condvar)>,
    thread: Option<JoinHandle<()>>,
}

impl Autosave {
    /// Start the autosave of the workspace at `root`, its baseline the bytes `presentation.ron`
    /// holds now, read on the caller's thread, or none when it has none. `wake` asks the window
    /// for a frame when the guard stops the writes or a write fails, so the banner or the label
    /// shows it.
    pub fn start(root: PathBuf, wake: Arc<dyn Fn() + Send + Sync>) -> Autosave {
        let file = root.join("presentation.ron");
        let mut baseline = std::fs::read(&file).ok();
        let shared = Arc::new((Mutex::new(State::default()), Condvar::new()));
        let theirs = shared.clone();
        let thread = std::thread::spawn(move || {
            let (lock, cond) = &*theirs;
            loop {
                let job = {
                    let mut state = lock.lock().unwrap();
                    while !state.quit && (state.held || state.job.is_none()) {
                        state = cond.wait(state).unwrap();
                    }
                    match state.job.take() {
                        Some(job) => {
                            state.busy = true;
                            job
                        }
                        None => return,
                    }
                };
                let done = run(&root, &file, job, &mut baseline, lock);
                let mut state = lock.lock().unwrap();
                state.busy = false;
                match done {
                    Done::Wrote => state.writes += 1,
                    Done::Stopped => {
                        state.stopped = true;
                        drop(state);
                        wake();
                        cond.notify_all();
                        continue;
                    }
                    Done::Failed => {
                        drop(state);
                        wake();
                        cond.notify_all();
                        continue;
                    }
                    Done::Nothing => {}
                }
                cond.notify_all();
            }
        });
        Autosave {
            shared,
            thread: Some(thread),
        }
    }

    fn put(&self, job: Job) {
        let (lock, cond) = &*self.shared;
        let mut state = lock.lock().unwrap();
        state.job = Some(job);
        cond.notify_all();
    }

    /// Write `doc` after an undo step, unless the guard has stopped the writes.
    pub fn save(&self, doc: Document) {
        self.put(Job::Write(Box::new(doc)));
    }

    /// "Keep mine": write `doc` over the file, and resume.
    pub fn keep_mine(&self, doc: Document) {
        self.put(Job::Force(Box::new(doc)));
    }

    /// After "Load the file": take the file as the baseline, resume, and write `doc`, the
    /// document loaded from it.
    pub fn resume(&self, doc: Document) {
        self.put(Job::Resume(Box::new(doc)));
    }

    /// Whether the guard found the file changed outside rootpoint, and writes nothing until the
    /// author answers.
    pub fn stopped(&self) -> bool {
        self.shared.0.lock().unwrap().stopped
    }

    /// Why the last write failed, if it did and none succeeded since.
    pub fn failure(&self) -> Option<String> {
        self.shared.0.lock().unwrap().failure.clone()
    }

    /// How many times the thread wrote the file.
    pub fn writes(&self) -> u64 {
        self.shared.0.lock().unwrap().writes
    }

    /// Keep the next job waiting, or let it run: a test holds the thread to send a burst of steps.
    pub fn hold(&self, held: bool) {
        let (lock, cond) = &*self.shared;
        lock.lock().unwrap().held = held;
        cond.notify_all();
    }

    /// Wait until no job waits and none runs.
    pub fn wait_idle(&self) {
        let (lock, cond) = &*self.shared;
        let mut state = lock.lock().unwrap();
        while state.busy || (state.job.is_some() && !state.held) {
            state = cond.wait(state).unwrap();
        }
    }
}

impl Drop for Autosave {
    fn drop(&mut self) {
        {
            let (lock, cond) = &*self.shared;
            let mut state = lock.lock().unwrap();
            state.held = false;
            state.quit = true;
            cond.notify_all();
        }
        // The job still waiting is written first: the last edit before a close is not lost.
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

enum Done {
    Wrote,
    Stopped,
    Failed,
    Nothing,
}

/// Do one job: the guard reads the file and compares it with the bytes last written or read; a
/// difference stops the writes, unless the job forces one.
fn run(
    root: &std::path::Path,
    file: &std::path::Path,
    job: Job,
    baseline: &mut Option<Vec<u8>>,
    lock: &Mutex<State>,
) -> Done {
    let (doc, force) = match job {
        Job::Resume(doc) => {
            *baseline = std::fs::read(file).ok();
            lock.lock().unwrap().stopped = false;
            (doc, false)
        }
        Job::Write(doc) if lock.lock().unwrap().stopped => {
            drop(doc);
            return Done::Nothing;
        }
        Job::Write(doc) => (doc, false),
        Job::Force(doc) => (doc, true),
    };
    if !force && std::fs::read(file).ok() != *baseline {
        return Done::Stopped;
    }
    let text = save_ron(&doc);
    // A write between the comparison and the rename is not seen, a window of one write's time,
    // which `design@rootpoint-app@ron-overwrite-guard` accepts.
    match write_through_rename(root, file, text.as_bytes()) {
        Ok(()) => {
            *baseline = Some(text.into_bytes());
            let mut state = lock.lock().unwrap();
            state.failure = None;
            if force {
                state.stopped = false;
            }
            Done::Wrote
        }
        Err(e) => {
            lock.lock().unwrap().failure = Some(e);
            Done::Failed
        }
    }
}

/// Write the file whole: to `.rootpoint/presentation.ron.new`, inside the root and on its file
/// system, flushed to the disk, then renamed over the file, so a reader never sees half of it.
fn write_through_rename(
    root: &std::path::Path,
    file: &std::path::Path,
    bytes: &[u8],
) -> Result<(), String> {
    let dir = root.join(".rootpoint");
    let temporary = dir.join("presentation.ron.new");
    let at = |path: &std::path::Path| {
        let path = path.display().to_string();
        move |e: std::io::Error| format!("{path}: {e}")
    };
    std::fs::create_dir_all(&dir).map_err(at(&dir))?;
    let mut out = File::create(&temporary).map_err(at(&temporary))?;
    out.write_all(bytes).map_err(at(&temporary))?;
    out.sync_all().map_err(at(&temporary))?;
    std::fs::rename(&temporary, file).map_err(at(file))
}
