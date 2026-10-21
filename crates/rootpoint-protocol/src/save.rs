//! Where a save puts each asset, so that every asset is readable beside the saved file and no other
//! file is overwritten. Paths are compared lexically, after `.` and `..` components are removed.

use std::path::{Component, Path, PathBuf};

/// The path with its `.` components removed, and each `..` removed with the component before it,
/// where there is one.
pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut out: Vec<Component> = Vec::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => match out.last() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                // A `..` above the root is the root.
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => out.push(c),
            },
            c => out.push(c),
        }
    }
    out.iter().collect()
}

/// The path of `full` relative to `dir`, both normalized, written with `/`, when `full` lies inside
/// `dir` at any depth.
pub(crate) fn inside(full: &Path, dir: &Path) -> Option<String> {
    let rest = full.strip_prefix(dir).ok()?;
    let words: Option<Vec<&str>> = rest
        .components()
        .map(|c| match c {
            Component::Normal(w) => w.to_str(),
            _ => None,
        })
        .collect();
    let words = words?;
    (!words.is_empty()).then(|| words.join("/"))
}

/// The path of `full` relative to `dir`, both normalized, with `..` where it lies outside; `None`
/// where only the names of the directories above `dir` would tell, as for a `dir` that starts with
/// `..` that `full` does not share.
pub(crate) fn relative(full: &Path, dir: &Path) -> Option<String> {
    let full: Vec<Component> = full.components().collect();
    let dir: Vec<Component> = dir.components().collect();
    let common = full.iter().zip(&dir).take_while(|(a, b)| a == b).count();
    let mut words = Vec::new();
    for c in &dir[common..] {
        match c {
            Component::Normal(_) => words.push(".."),
            _ => return None,
        }
    }
    for c in &full[common..] {
        match c {
            Component::Normal(w) => words.push(w.to_str()?),
            Component::ParentDir => words.push(".."),
            _ => return None,
        }
    }
    (!words.is_empty()).then(|| words.join("/"))
}

/// The extension that an embedded asset's bytes name: an image or a font the renderer reads, or
/// none.
pub(crate) fn extension(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        ".png"
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        ".jpg"
    } else if bytes.starts_with(&[0, 1, 0, 0]) || bytes.starts_with(b"true") {
        ".ttf"
    } else if bytes.starts_with(b"OTTO") {
        ".otf"
    } else {
        ""
    }
}

/// The name with the suffix `-<n>` before its extension: `cat.png` gives `cat-2.png`.
pub(crate) fn with_suffix(name: &str, n: u32) -> String {
    match name.rfind('.').filter(|&i| i > 0) {
        Some(i) => format!("{}-{n}{}", &name[..i], &name[i..]),
        None => format!("{name}-{n}"),
    }
}
