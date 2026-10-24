//! The store's own files and folders, the workspace's lock under a race, relinks to a folder, and
//! the time `open-pack` takes over many files.

mod common;

use common::*;
use rootpoint_protocol::{Reply, Session};

/// An asset path whose first component is `mimetype` or `presentation.ron`, a file of a pack or of
/// a workspace, is refused at the import, so no pack holds a file and a folder of one name, which
/// `open-pack` would refuse.
#[test]
fn an_asset_below_a_file_of_the_store_s_own_is_refused_at_its_import() {
    let mut files = MemoryFiles::default();
    files.files.insert("logo.png".into(), two_colour_png());
    let mut s = Session::new();
    for to in ["mimetype/logo.png", "presentation.ron/x"] {
        let imported = s.execute(&format!("import logo.png {to}"), &mut files);
        assert!(
            matches!(imported, Some(Reply::Err(_))),
            "{to}: {imported:?}"
        );
    }
    ok(&mut s, &mut files, "import logo.png");
    ok(&mut s, &mut files, "pack s.rpk");
    let mut t = Session::new();
    let reply = t.execute("open-pack s.rpk", &mut files);
    assert_eq!(reply, Some(Reply::Ok(String::new())));
}

/// Two hosts open one directory that holds no `.rootpoint/` yet, at once: the one the other's lock
/// refuses removes none of the winner's files, so no third host takes the lock while the winner
/// holds it.
#[test]
fn a_host_refused_by_another_s_lock_removes_none_of_its_files() {
    use rootpoint_protocol::workspace::{WorkspaceLock, run};
    use std::sync::{Arc, Barrier};
    let base = std::fs::canonicalize(scratch("lock-race")).unwrap();
    let mut broken = 0;
    for round in 0..400 {
        let dir = base.join(format!("w{round}"));
        std::fs::create_dir_all(&dir).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let hosts: Vec<_> = (0..2)
            .map(|_| {
                let (barrier, dir) = (barrier.clone(), dir.to_str().unwrap().to_owned());
                std::thread::spawn(move || {
                    let mut files = RecordingFiles::default();
                    let mut session = Session::new();
                    let mut lock = None;
                    barrier.wait();
                    let reply = run(
                        &mut session,
                        rootpoint_protocol::Command::Open { dir },
                        &mut files,
                        &mut lock,
                    );
                    (reply, lock)
                })
            })
            .collect();
        let results: Vec<_> = hosts.into_iter().map(|h| h.join().unwrap()).collect();
        let winners = results
            .iter()
            .filter(|(r, l)| matches!(r, Reply::Ok(_)) && l.is_some())
            .count();
        // While a winner holds its lock, no third host may take it.
        if winners == 1 && WorkspaceLock::take(&dir).is_ok() {
            broken += 1;
        }
        drop(results);
    }
    assert_eq!(broken, 0, "rounds where a third host took a held lock");
}

/// The control of the race above: with one host, a held lock refuses a second taker.
#[test]
fn control_a_held_lock_refuses_a_second_taker() {
    use rootpoint_protocol::workspace::{WorkspaceLock, run};
    let dir = std::fs::canonicalize(scratch("lock-control")).unwrap();
    let mut files = RecordingFiles::default();
    let mut session = Session::new();
    let mut lock = None;
    let reply = run(
        &mut session,
        rootpoint_protocol::Command::Open {
            dir: dir.to_str().unwrap().to_owned(),
        },
        &mut files,
        &mut lock,
    );
    assert!(matches!(reply, Reply::Ok(_)) && lock.is_some());
    assert!(WorkspaceLock::take(&dir).is_err());
}

fn run_native(s: &mut Session, files: &mut RecordingFiles, line: &str) -> Reply {
    s.execute(line, files).unwrap()
}

/// A relink names a file, never a folder: a relink to a folder is refused, so no later removal of
/// the asset moves the folder, and the file of another asset in it, to the trash.
#[test]
fn a_relink_to_a_folder_is_refused_and_no_removal_takes_another_asset_s_file() {
    let dir = std::fs::canonicalize(scratch("relink-folder")).unwrap();
    std::fs::create_dir_all(dir.join("img")).unwrap();
    std::fs::write(dir.join("img/a.png"), two_colour_png()).unwrap();
    std::fs::write(dir.join("b.png"), b"b").unwrap();
    let root = dir.to_str().unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    assert!(matches!(
        run_native(&mut s, &mut files, &format!("open {root}")),
        Reply::Ok(_)
    ));
    let a = match run_native(&mut s, &mut files, &format!("import {root}/img/a.png")) {
        Reply::Ok(id) => id,
        r => panic!("{r:?}"),
    };
    let b = match run_native(&mut s, &mut files, &format!("import {root}/b.png")) {
        Reply::Ok(id) => id,
        r => panic!("{r:?}"),
    };
    std::fs::remove_file(dir.join("b.png")).unwrap();
    let relink = run_native(&mut s, &mut files, &format!("asset-relink {b} img"));
    let remove = run_native(&mut s, &mut files, &format!("asset-remove {b}"));
    assert!(
        dir.join("img/a.png").exists(),
        "{a}'s file left its place: relink {relink:?}, remove {remove:?}"
    );
    assert!(
        matches!(relink, Reply::Err(_)),
        "a relink to a folder: {relink:?}"
    );
}

/// The same in memory: the relink to a folder is refused, and the asset is still removed.
#[test]
fn in_memory_a_relink_to_a_folder_is_refused() {
    let mut files = MemoryFiles::default();
    files.files.insert("a.png".into(), two_colour_png());
    files.files.insert("b.png".into(), b"b".to_vec());
    let mut first = Session::new();
    ok(&mut first, &mut files, "import a.png img/a.png");
    ok(&mut first, &mut files, "import b.png gone.png");
    write_presentation(&first, &mut files, "d");
    files.files.insert("d/img/a.png".into(), two_colour_png());
    let mut s = Session::new();
    ok(&mut s, &mut files, "open-copy d");
    let relink = s.execute("asset-relink a2 img", &mut files);
    let remove = s.execute("asset-remove a2", &mut files);
    assert!(
        matches!(remove, Some(Reply::Ok(_))),
        "the removal: {remove:?}"
    );
    assert!(
        matches!(relink, Some(Reply::Err(_))),
        "a relink to a folder: {relink:?}"
    );
}

/// The check that no file of a pack is another's folder reads each name's folders once: ten times
/// the entries take less than thirty times as long to open.
#[test]
fn opening_a_pack_of_many_files_takes_time_linear_in_its_entries() {
    use std::io::Write;
    let ron = {
        let mut files = MemoryFiles::default();
        let mut s = Session::new();
        ok(&mut s, &mut files, "new");
        rootpoint_model::save_ron(s.document())
    };
    let time = |n: usize| {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let stored = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/x-rootpoint").unwrap();
        zip.start_file("presentation.ron", stored).unwrap();
        zip.write_all(ron.as_bytes()).unwrap();
        for i in 0..n {
            zip.start_file(format!("img/{i:05}.png"), stored).unwrap();
            zip.write_all(b"x").unwrap();
        }
        let mut files = MemoryFiles::default();
        files
            .files
            .insert("p.rpk".into(), zip.finish().unwrap().into_inner());
        let mut s = Session::new();
        let start = std::time::Instant::now();
        ok(&mut s, &mut files, "open-pack p.rpk");
        start.elapsed()
    };
    let small = time(2000);
    let large = time(20000);
    eprintln!("2000 entries: {small:?}; 20000 entries: {large:?}");
    // Ten times the entries; linear work takes about ten times as long.
    assert!(
        large < small * 30,
        "2000 entries: {small:?}; 20000 entries: {large:?}"
    );
}
