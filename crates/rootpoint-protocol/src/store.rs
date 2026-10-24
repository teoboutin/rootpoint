//! The paths of a store: an asset path joined to a root, the check that keeps every access inside
//! the root, and the name an import gives a file.

use rootpoint_model::asset_path_is_confined;

use crate::files::Files;

/// The file of a workspace that holds its presentation, at its root.
pub(crate) const PRESENTATION: &str = "presentation.ron";

/// `path` below the directory `dir` of the host: the path itself when `dir` is empty.
pub(crate) fn join(dir: &str, path: &str) -> String {
    if dir.is_empty() {
        path.to_owned()
    } else if dir.ends_with('/') {
        format!("{dir}{path}")
    } else {
        format!("{dir}/{path}")
    }
}

/// Whether `canonical` is `root` or lies below it, both canonical.
fn under(canonical: &str, root: &str) -> bool {
    canonical == root || canonical.starts_with(&join(root, ""))
}

/// The refusal of an asset path that fails `asset_path_is_confined`,
/// `design@rootpoint-protocol@store-confines-access`.
pub(crate) fn unconfined(path: &str) -> String {
    format!(
        "{} is not a path inside the root",
        crate::command::quote(path)
    )
}

/// The host path of the asset path `path` below `root`, for a read: refused when the path fails
/// `asset_path_is_confined`, or when its file, once every symlink is resolved, lies outside the
/// root. A path no file is at gives the host's own error, as a read would.
pub(crate) fn readable(root: &str, path: &str, files: &mut dyn Files) -> Result<String, String> {
    if !asset_path_is_confined(path) {
        return Err(unconfined(path));
    }
    let full = join(root, path);
    let canonical = files.canonical(&full)?;
    let root = files.canonical(root)?;
    if !under(&canonical, &root) || canonical == root {
        return Err(format!("{full} leads out of the root"));
    }
    Ok(full)
}

/// The host path of the asset path `path` below `root`, for a write: refused as for a read, the
/// file's own canonical path taken when it exists, and its nearest existing directory's otherwise.
pub(crate) fn writable(root: &str, path: &str, files: &mut dyn Files) -> Result<String, String> {
    if !asset_path_is_confined(path) {
        return Err(unconfined(path));
    }
    let full = join(root, path);
    let mut probe = full.clone();
    while !files.exists(&probe)? {
        match probe.rfind('/') {
            Some(i) if probe[..i].len() >= root.len() => probe.truncate(i),
            _ => {
                probe = root.to_owned();
                break;
            }
        }
    }
    let canonical = files.canonical(&probe)?;
    let root = files.canonical(root)?;
    if !under(&canonical, &root) || (probe == full && canonical == root) {
        return Err(format!("{full} leads out of the root"));
    }
    Ok(full)
}

/// The name with the suffix `-<n>` before the extension of its last component: `img/cat.png`
/// gives `img/cat-2.png`.
pub(crate) fn with_suffix(name: &str, n: u32) -> String {
    let start = name.rfind('/').map_or(0, |i| i + 1);
    match name[start..].rfind('.').filter(|&i| i > 0) {
        Some(i) => format!("{}-{n}{}", &name[..start + i], &name[start + i..]),
        None => format!("{name}-{n}"),
    }
}
