//! A session: one document, its undo history and a view state, changed by commands.

use std::fmt;

use std::collections::{BTreeMap, BTreeSet};

use rootpoint_math::Formula;
use rootpoint_model::{
    Area, Asset, AssetId, BUILTIN_FAMILIES, Content, Document, EquationContent, EquationSize,
    FontRef, ImageContent, Node, NodeId, Op, Replacement, SlideId, Span, SpanChoice, SpanId,
    SpanStyle, StyleChoice, TextContent, TextItem, TextNode, TextNodeRef, TextStyle, apply,
    asset_path_is_confined, load_ron, save_ron, span_style_users,
};
use rootpoint_render::{FontSet, ImageSet, Precision, Report, render_slide};

use crate::command::{
    AssetFilter, Command, EquationOption, SpanField, SpanOption, SpanStyleCommand, StyleCommand,
    StyleOption, TextTarget, TreeItem, parse_command, quote, quote_always,
};
use crate::files::Files;
use crate::store::{PRESENTATION, TRASH, join, readable, unconfined, with_suffix, writable};
use crate::tree::slide_tree;

/// The reply to one command: `ok` with its result, or `err` with the reason. It prints as one
/// line: a new line or a carriage return in its text, as a path or a word an error repeats can
/// hold, prints as `\n` or `\r`.
#[derive(Clone, Debug, PartialEq)]
pub enum Reply {
    Ok(String),
    Err(String),
}

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (word, text) = match self {
            Reply::Ok(t) => ("ok", t),
            Reply::Err(t) => ("err", t),
        };
        if text.is_empty() {
            f.write_str(word)
        } else {
            let text = text.replace('\n', "\\n").replace('\r', "\\r");
            write!(f, "{word} {text}")
        }
    }
}

/// What the session shows besides the document: the selected node, the slide the editor shows,
/// the slide presenter mode shows, if it is on, and whether the editor shows the final result, the
/// preview. With no slide shown, the editor shows the first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct View {
    pub selected: Option<NodeId>,
    pub shown: Option<SlideId>,
    pub presenting: Option<SlideId>,
    /// The preview: the editor shows the slide with no mark, and no node is selected in it, so
    /// `preview on` deselects and `select n<id>` ends it.
    pub preview: bool,
}

/// The rule for one line of commands, as bytes, shared by every host that reads lines:
/// `None` for a blank line or a comment, which starts with `#`; an `err` reply for a line that is
/// not UTF-8 or does not parse; the command otherwise.
pub fn read_line(line: &[u8]) -> Option<Result<Command, Reply>> {
    let Ok(line) = std::str::from_utf8(line) else {
        return Some(Err(Reply::Err("the line is not UTF-8".to_owned())));
    };
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    Some(parse_command(line).map_err(|e| Reply::Err(e.0)))
}

/// A place of the store a file moves between: an asset path, or an entry of the trash, which is
/// `.rootpoint/trash/<name>` in a workspace and `Session::trash` in memory.
#[derive(Clone, Debug, PartialEq)]
enum Place {
    Asset(String),
    Trash(String),
}

/// A file moved from one place of the store to another, which holds no file.
#[derive(Clone, Debug, PartialEq)]
struct Move {
    from: Place,
    to: Place,
}

/// One undo step: the file moves and the operations that revert one command, or every command of
/// one group, each applied in order, the moves first. A move names its places by path, never by
/// the document, so the moves and the operations are independent of each other.
#[derive(Clone, Debug, Default)]
struct Step {
    ops: Vec<Op>,
    moves: Vec<Move>,
}

impl Step {
    fn ops(ops: Vec<Op>) -> Step {
        Step {
            ops,
            moves: Vec::new(),
        }
    }

    /// The step that applies `steps`' reverts in turn, the last first: the revert of a group from
    /// the reverts of its commands, in the order they were applied.
    fn undoing(steps: Vec<Step>) -> Step {
        let mut all = Step::default();
        for step in steps.into_iter().rev() {
            all.ops.extend(step.ops);
            all.moves.extend(step.moves);
        }
        all
    }
}

/// A group opened by `begin`: the reverts of its commands, in the order they were applied, and the
/// redo stack as it was at the `begin`, which `cancel` restores.
struct Group {
    reverts: Vec<Step>,
    redo: Vec<Step>,
}

/// Where the session finds the files of its presentation. An asset path is a key of the map in
/// memory, or relative to a workspace's root, which never changes while the workspace is open.
#[derive(Clone, Debug, PartialEq)]
pub enum Store {
    /// The presentation is held in memory: its assets' files are this map, by asset path, and no
    /// file of it is on disk.
    Memory(BTreeMap<String, Vec<u8>>),
    /// A workspace: `presentation.ron` and the assets' files are the host's, below this root, a
    /// directory written as `Files::canonical` gives it.
    Workspace(String),
}

/// One document, its undo and redo stacks, its view and its store, changed only by commands.
pub struct Session {
    doc: Document,
    store: Store,
    /// The undo steps pushed, undone or redone since the store was opened or last saved, or, in
    /// memory, written as a pack.
    unsaved: u64,
    /// The host path of the pack last opened or written in memory, which `save` writes again.
    pack: Option<String>,
    undo: Vec<Step>,
    redo: Vec<Step>,
    group: Option<Group>,
    view: View,
    /// The trash in memory: the bytes of removed and replaced assets, by entry name, which an undo
    /// or a redo moves back. A workspace's trash is `.rootpoint/trash/` below its root.
    trash: BTreeMap<String, Vec<u8>>,
    /// The entries the session put in the trash, which numbers the next one.
    trashed: u64,
    /// The files of the store the session moved, which `Session::files_moved` gives.
    moved: u64,
}

impl Default for Session {
    fn default() -> Self {
        Session::new()
    }
}

impl Session {
    /// An empty document, not yet saved.
    pub fn new() -> Self {
        Session {
            doc: Document::default(),
            store: Store::Memory(BTreeMap::new()),
            unsaved: 0,
            pack: None,
            undo: Vec::new(),
            redo: Vec::new(),
            group: None,
            view: View::default(),
            trash: BTreeMap::new(),
            trashed: 0,
            moved: 0,
        }
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The root of a workspace, against which its asset paths are read; none in memory, where the
    /// map's keys are the asset paths.
    pub fn root(&self) -> Option<&str> {
        match &self.store {
            Store::Memory(_) => None,
            Store::Workspace(root) => Some(root),
        }
    }

    /// The undo steps pushed, undone or redone since the store was opened or last saved: 0 when
    /// no edit was made since.
    pub fn unsaved(&self) -> u64 {
        self.unsaved
    }

    /// The host path of the pack a presentation held in memory was last opened from or written to,
    /// which `save` writes again; none in a workspace.
    pub fn pack(&self) -> Option<&str> {
        self.pack.as_deref()
    }

    /// Run one line: a command's reply, or `None` for a blank line or a comment, which starts
    /// with `#`, by the rule of `read_line`.
    pub fn execute(&mut self, line: &str, files: &mut dyn Files) -> Option<Reply> {
        Some(match read_line(line.as_bytes())? {
            Ok(command) => self.run(command, files),
            Err(reply) => reply,
        })
    }

    /// How many moves of the store's files the session tried, by an asset command or by the undo
    /// or the redo of one, a move that failed included: a host that keeps assets decoded by their paths decodes them again when it
    /// changes, since a path then holds other bytes.
    pub fn files_moved(&self) -> u64 {
        self.moved
    }

    /// Note that the host wrote the presentation to its store itself, as the window's autosave
    /// does at its `save`: the undo steps since the save count from 0 again.
    pub fn mark_saved(&mut self) {
        self.unsaved = 0;
    }

    /// Whether a group opened by `begin` is open.
    pub fn group_open(&self) -> bool {
        self.group.is_some()
    }

    /// Whether the open group holds an edit: a step `Session::unsaved` counts only once the group
    /// ends, as a text typed in place is until its editing ends.
    pub fn group_changed(&self) -> bool {
        self.group.as_ref().is_some_and(|g| !g.reverts.is_empty())
    }

    /// Run one command. A command that fails changes neither the document nor the view.
    pub fn run(&mut self, command: Command, files: &mut dyn Files) -> Reply {
        match self.run_inner(command, files) {
            Ok(text) => Reply::Ok(text),
            Err(reason) => Reply::Err(reason),
        }
    }

    fn run_inner(&mut self, command: Command, files: &mut dyn Files) -> Result<String, String> {
        match command {
            Command::New => {
                *self = Session::new();
                Ok(String::new())
            }
            Command::Save => match (&self.store, self.pack.clone()) {
                (Store::Workspace(root), _) => {
                    files.write(&join(root, PRESENTATION), save_ron(&self.doc).as_bytes())?;
                    self.unsaved = 0;
                    Ok(String::new())
                }
                (Store::Memory(_), Some(file)) => self.write_pack(&file, files),
                (Store::Memory(_), None) => Err("a presentation held in memory is saved by \
                    `pack <file>` or `unpack <dir>`"
                    .to_owned()),
            },
            Command::Pack { file } => self.write_pack(&file, files),
            Command::Reload => {
                let Store::Workspace(root) = &self.store else {
                    return Err("a presentation held in memory has no file to reload".to_owned());
                };
                let doc = read_presentation(&join(root, PRESENTATION), files)?;
                // The reply names no id: the file's ids are its own, and the operation creates
                // none.
                self.edit(
                    Op::ReplaceDocument {
                        document: Box::new(doc),
                    },
                    files,
                )?;
                Ok(String::new())
            }
            Command::OpenPack { file } => {
                let bytes = files.read(&file)?;
                let (text, map) =
                    crate::pack::read_pack(&bytes).map_err(|e| format!("{file}: {e}"))?;
                let doc = load_ron(&text).map_err(|e| format!("{file}: {e:?}"))?;
                *self = Session::new();
                self.doc = doc;
                self.store = Store::Memory(map);
                self.pack = Some(file);
                Ok(String::new())
            }
            Command::Unpack { dir } => {
                let Store::Memory(map) = &self.store else {
                    return Err(
                        "a workspace is unpacked already: `pack <file>` writes it as a pack"
                            .to_owned(),
                    );
                };
                // A directory that holds anything but `.rootpoint/`, which a host's lock creates
                // first, is refused, so an unpack overwrites no file; a path that is a file fails
                // the listing.
                if files.exists(&dir)?
                    && files
                        .list(&dir)?
                        .iter()
                        .any(|p| p.split('/').next() != Some(".rootpoint"))
                {
                    return Err(format!(
                        "{} holds files: unpack into an empty directory",
                        quote(&dir)
                    ));
                }
                for (path, bytes) in map {
                    files.write(&join(&dir, path), bytes)?;
                }
                files.write(&join(&dir, PRESENTATION), save_ron(&self.doc).as_bytes())?;
                // The trash goes with the map, so that the undo steps, which stay, find their
                // files in the workspace's trash.
                let root = files.canonical(&dir)?;
                for (entry, bytes) in &self.trash {
                    let path = trash_path(&root, entry, files)?;
                    files.write(&path, bytes)?;
                }
                self.trash.clear();
                self.store = Store::Workspace(root);
                self.pack = None;
                self.unsaved = 0;
                Ok(String::new())
            }
            Command::Sample => {
                let (doc, images) = crate::sample::sample()?;
                *self = Session::new();
                self.doc = doc;
                self.store = Store::Memory(images);
                Ok(String::new())
            }
            Command::Open { dir } => {
                let root = files.canonical(&dir)?;
                let file = join(&root, PRESENTATION);
                // A directory with no presentation opens an empty one, which the first save writes.
                let doc = if files.exists(&file)? {
                    read_presentation(&file, files)?
                } else {
                    Document::default()
                };
                // The trash of an earlier session is emptied, since no undo step of this one can
                // take its files back; an entry that cannot be removed stays, and the session
                // names its own entries around it.
                // A trash that a symlink places elsewhere is left alone, so the open removes no file
                // outside the root.
                if let Ok(trash) = trash_path(&root, "", files) {
                    for entry in files.list(&trash).unwrap_or_default() {
                        let _ = files.remove(&join(&trash, &entry));
                    }
                }
                *self = Session::new();
                self.doc = doc;
                self.store = Store::Workspace(root);
                Ok(String::new())
            }
            Command::OpenCopy { dir } => {
                let doc = read_presentation(&join(&dir, PRESENTATION), files)?;
                // Each asset is read through the check of a workspace at `dir`, so the copy reads
                // nothing outside it; an asset it refuses, or cannot read, is missing in the map.
                let mut map = BTreeMap::new();
                for asset in doc.assets.values() {
                    if let Ok(full) = readable(&dir, &asset.path, files)
                        && let Ok(bytes) = files.read(&full)
                    {
                        map.insert(asset.path.clone(), bytes);
                    }
                }
                *self = Session::new();
                self.doc = doc;
                self.store = Store::Memory(map);
                Ok(String::new())
            }
            Command::Import { from, to } => {
                let (path, bytes) = self.import(&from, to, files)?;
                let existing = self
                    .doc
                    .assets
                    .iter()
                    .find(|(_, a)| a.path == path)
                    .map(|(id, _)| *id);
                // A file the import writes enters the trash first and is moved to its path in the
                // step, so that the step's undo takes it back, `design@rootpoint-protocol@asset-trash`.
                let mut moves = Vec::new();
                let mut staged = None;
                if let Some(bytes) = bytes {
                    let entry = self.trash_entry("import", files)?;
                    self.put_in_trash(&entry, bytes, files)?;
                    moves.push(Move {
                        from: Place::Trash(entry.clone()),
                        to: Place::Asset(path.clone()),
                    });
                    staged = Some(entry);
                }
                let ops = match existing {
                    Some(_) => Vec::new(),
                    None => vec![Op::AddAsset {
                        asset: Asset { path: path.clone() },
                    }],
                };
                if (!ops.is_empty() || !moves.is_empty())
                    && let Err(e) = self.change(Step { ops, moves }, files)
                {
                    if let Some(entry) = staged {
                        self.drop_from_trash(&entry, files);
                    }
                    return Err(e);
                }
                let id = self
                    .doc
                    .assets
                    .iter()
                    .find(|(_, a)| a.path == path)
                    .map(|(id, _)| *id)
                    .expect("the asset is in the table");
                Ok(id.to_string())
            }
            Command::Store => {
                let unsaved = self.unsaved;
                Ok(match &self.store {
                    Store::Memory(_) => match &self.pack {
                        Some(file) => format!("memory unsaved={unsaved} pack={}", quote(file)),
                        None => format!("memory unsaved={unsaved}"),
                    },
                    Store::Workspace(root) => {
                        format!("workspace {} unsaved={unsaved}", quote(root))
                    }
                })
            }
            Command::Tree { slide } => {
                let numbers = self.doc.equation_numbers();
                let slides: Vec<String> = match slide {
                    Some(id) => {
                        let s = self.doc.slide(id).ok_or(format!("no slide {id}"))?;
                        vec![slide_tree(s, &numbers)]
                    }
                    None => self
                        .doc
                        .slides
                        .iter()
                        .map(|s| slide_tree(s, &numbers))
                        .collect(),
                };
                Ok(slides.join("; "))
            }
            Command::Render { slide, path, size } => {
                let (w, h) = size.unwrap_or((1920, 1080));
                let the_slide = self.doc.slide(slide).ok_or(format!("no slide {slide}"))?;
                let (mut fonts, mut images) = (FontSet::new(), ImageSet::new());
                for area in the_slide.root.areas() {
                    match &area.content {
                        Content::Image(i) => {
                            // An image that cannot be read or decoded is drawn grey, and the
                            // render's report names it.
                            if let Ok(bytes) = self.asset_bytes(i.asset, files) {
                                let _ = images.insert_bytes(i.asset, &bytes);
                            }
                        }
                        Content::Equation(_) => {}
                        Content::Text(t) => {
                            for font in self.doc.text_fonts(t) {
                                if let FontRef::Asset(id) = font
                                    && let Ok(bytes) = self.asset_bytes(id, files)
                                {
                                    let _ = fonts.insert_bytes(id, bytes);
                                }
                            }
                        }
                    }
                }
                // The portable precision, so that a headless image is the same on every
                // machine (`design@rootpoint-render@portable-precision`).
                let out = render_slide(
                    &self.doc,
                    slide,
                    w,
                    h,
                    &mut fonts,
                    &images,
                    Precision::Portable,
                )
                .map_err(|e| format!("{e:?}"))?;
                let report = report_words(&out.report);
                files.write(&path, &out.png())?;
                Ok(report)
            }
            Command::Undo | Command::Redo => {
                // An `undo` or a `redo` inside a group closes it first, as `end` does.
                self.end_group(files);
                let undo = matches!(command, Command::Undo);
                let from = if undo { &mut self.undo } else { &mut self.redo };
                let step = from.pop().ok_or(format!("nothing to {command}"))?;
                match self.apply_step(step.clone(), files) {
                    Ok(back) if undo => {
                        self.redo.push(back);
                        self.unsaved += 1;
                    }
                    Ok(back) => {
                        self.undo.push(back);
                        self.unsaved += 1;
                    }
                    Err(e) => {
                        let from = if undo { &mut self.undo } else { &mut self.redo };
                        from.push(step);
                        return Err(e);
                    }
                }
                Ok(String::new())
            }
            Command::Begin => {
                if self.group.is_some() {
                    return Err("a group is open".to_owned());
                }
                self.group = Some(Group {
                    reverts: Vec::new(),
                    redo: self.redo.clone(),
                });
                Ok(String::new())
            }
            Command::End => {
                self.group.as_ref().ok_or("no group is open")?;
                self.end_group(files);
                Ok(String::new())
            }
            // A session run headless has one controller, which holds the authority to edit; an
            // editor's host decides who holds it before the command reaches the session.
            Command::Control => Ok(String::new()),
            // Refused while a group is open, so that a script that holds the authority gives the
            // same replies in a window and headless, `design@rootpoint-protocol@control-commands`.
            Command::Release => match self.group {
                Some(_) => Err("a group is open".to_owned()),
                None => Ok(String::new()),
            },
            Command::Cancel => {
                let group = self.group.as_ref().ok_or("no group is open")?;
                self.apply_step(Step::undoing(group.reverts.clone()), files)?;
                let group = self.group.take().expect("checked above");
                let moved = group.reverts.iter().any(|s| !s.moves.is_empty());
                self.redo = group.redo;
                // The files the group put in the trash are reached by no step any more.
                if moved {
                    self.sweep_trash(files);
                }
                Ok(String::new())
            }
            Command::Select { node } => {
                let slide = self
                    .doc
                    .slides
                    .iter()
                    .find(|s| s.root.find(node).is_some())
                    .ok_or(format!("no node {node}"))?;
                self.view.shown = Some(slide.id);
                self.view.selected = Some(node);
                self.view.preview = false;
                Ok(String::new())
            }
            Command::SelectSlide { slide } => {
                self.doc.slide(slide).ok_or(format!("no slide {slide}"))?;
                self.view.shown = Some(slide);
                self.view.selected = None;
                Ok(String::new())
            }
            Command::Deselect => {
                self.view.selected = None;
                Ok(String::new())
            }
            Command::Preview { on } => {
                self.view.preview = on;
                if on {
                    self.view.selected = None;
                }
                Ok(String::new())
            }
            Command::Present { slide } => {
                let slide = match slide {
                    Some(id) => self.doc.slide(id).ok_or(format!("no slide {id}"))?.id,
                    None => self.doc.slides.first().ok_or("no slide to present")?.id,
                };
                self.view.presenting = Some(slide);
                Ok(slide.to_string())
            }
            Command::Next | Command::Prev => {
                let current = self.view.presenting.ok_or("not presenting")?;
                let i = self
                    .doc
                    .slides
                    .iter()
                    .position(|s| s.id == current)
                    .ok_or("not presenting")?;
                let target = if matches!(command, Command::Next) {
                    i.checked_add(1).filter(|&j| j < self.doc.slides.len())
                } else {
                    i.checked_sub(1)
                };
                let j = target.ok_or(if matches!(command, Command::Next) {
                    "at the last slide"
                } else {
                    "at the first slide"
                })?;
                let slide = self.doc.slides[j].id;
                self.view.presenting = Some(slide);
                Ok(slide.to_string())
            }
            Command::Stop => {
                self.view.presenting.ok_or("not presenting")?;
                self.view.presenting = None;
                Ok(String::new())
            }
            Command::Styles => Ok(self
                .doc
                .styles
                .iter()
                .map(|(name, style)| format!("{} {}", quote_always(name), style_options(style)))
                .collect::<Vec<_>>()
                .join("; ")),
            Command::View => {
                let show = |id: Option<String>| id.unwrap_or_else(|| "-".to_owned());
                Ok(format!(
                    "selected={} shown={} presenting={} preview={}",
                    show(self.view.selected.map(|n| n.to_string())),
                    show(self.view.shown.map(|s| s.to_string())),
                    show(self.view.presenting.map(|s| s.to_string())),
                    if self.view.preview { "on" } else { "off" }
                ))
            }
            Command::AssetMove { asset, to } => {
                let from = self.confined_asset(asset)?;
                self.unshared(asset, &from)?;
                self.free_path(asset, &to)?;
                if !self.file_at(&from, files)? {
                    return Err(format!(
                        "{asset} has no file at {}: `asset-relink` gives it one",
                        quote(&from)
                    ));
                }
                if self.taken_at(&to, files)? {
                    return Err(format!("{} holds a file", quote(&to)));
                }
                self.change(
                    Step {
                        ops: vec![Op::SetAssetPath {
                            id: asset,
                            path: to.clone(),
                        }],
                        moves: vec![Move {
                            from: Place::Asset(from),
                            to: Place::Asset(to),
                        }],
                    },
                    files,
                )?;
                Ok(String::new())
            }
            Command::AssetRelink { asset, to } => {
                self.doc
                    .assets
                    .get(&asset)
                    .ok_or(format!("no asset {asset}"))?;
                self.free_path(asset, &to)?;
                if !self.file_at(&to, files)? {
                    return Err(format!("{} holds no file", quote(&to)));
                }
                self.edit(
                    Op::SetAssetPath {
                        id: asset,
                        path: to,
                    },
                    files,
                )?;
                Ok(String::new())
            }
            Command::AssetReplace { asset, from } => {
                let path = self.confined_asset(asset)?;
                self.unshared(asset, &path)?;
                let held = self.file_at(&path, files)?;
                let bytes = files.read(&from)?;
                // The new bytes enter the trash first, so that every change of the store the step
                // makes is a move, which its undo runs back.
                let new = self.trash_entry(&asset.to_string(), files)?;
                self.put_in_trash(&new, bytes, files)?;
                let mut moves = Vec::new();
                if held {
                    let old = self.trash_entry(&asset.to_string(), files)?;
                    moves.push(Move {
                        from: Place::Asset(path.clone()),
                        to: Place::Trash(old),
                    });
                }
                moves.push(Move {
                    from: Place::Trash(new.clone()),
                    to: Place::Asset(path),
                });
                let step = Step {
                    ops: Vec::new(),
                    moves,
                };
                if let Err(e) = self.change(step, files) {
                    // The new bytes leave the trash with the step that failed.
                    self.drop_from_trash(&new, files);
                    return Err(e);
                }
                Ok(String::new())
            }
            Command::AssetRemove { asset, into } => {
                self.doc
                    .assets
                    .get(&asset)
                    .ok_or(format!("no asset {asset}"))?;
                let mut ops = match into {
                    None => {
                        let uses = self.asset_uses(asset);
                        if !uses.is_empty() {
                            return Err(format!("{asset} is used by {}", uses.join(" ")));
                        }
                        Vec::new()
                    }
                    Some(other) if other == asset => {
                        return Err(format!("{asset} cannot replace itself"));
                    }
                    Some(other) => {
                        self.existing_asset(other)?;
                        self.renaming_ops(asset, other)
                    }
                };
                ops.push(Op::RemoveAsset { id: asset });
                let moves = self.trash_moves(&[asset], files)?;
                self.change(Step { ops, moves }, files)?;
                Ok(String::new())
            }
            Command::AssetRemoveUnused => {
                let unused: Vec<AssetId> = self
                    .doc
                    .assets
                    .keys()
                    .copied()
                    .filter(|&id| self.asset_uses(id).is_empty())
                    .collect();
                // No asset unused, no step: an empty step would count as a change to save.
                if unused.is_empty() {
                    return Ok(String::new());
                }
                let ops = unused.iter().map(|&id| Op::RemoveAsset { id }).collect();
                let moves = self.trash_moves(&unused, files)?;
                self.change(Step { ops, moves }, files)?;
                Ok(unused
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(" "))
            }
            Command::Assets { filter } => self.list_assets(filter, files),
            Command::SpanStyles => Ok(self
                .doc
                .span_styles
                .iter()
                .map(|(name, style)| {
                    let mut words = vec![quote_always(name)];
                    words.extend(SpanOption::values_of(style).iter().map(|o| o.to_string()));
                    // A built-in family the binary does not carry, which only a file edited by
                    // hand holds, is written as its name, as `styles` writes it.
                    if let Some(FontRef::Builtin(family)) = &style.font
                        && family != "sans"
                    {
                        words.push(format!("font={}", quote(family)));
                    }
                    words.join(" ")
                })
                .collect::<Vec<_>>()
                .join("; ")),
            Command::Text { area, root, style } => {
                let ops = self.text_ops(area, root, style)?;
                self.edit_ops(ops, files)
            }
            editing => {
                let op = self.operation(editing)?;
                self.edit_ops(vec![op], files)
            }
        }
    }

    /// The path of an asset of the table, refused when it fails confinement: a command that moves
    /// or writes its file would reach outside the root,
    /// `design@rootpoint-protocol@store-confines-access`.
    fn confined_asset(&self, asset: AssetId) -> Result<String, String> {
        let path = &self
            .doc
            .assets
            .get(&asset)
            .ok_or(format!("no asset {asset}"))?
            .path;
        if !asset_path_is_confined(path) {
            return Err(unconfined(path));
        }
        Ok(path.clone())
    }

    /// Refuse a new path for `asset` that fails confinement, or that another asset names: two
    /// assets of one file would both change at a move or a replace of either.
    fn free_path(&self, asset: AssetId, path: &str) -> Result<(), String> {
        if !asset_path_is_confined(path) {
            return Err(unconfined(path));
        }
        match self
            .doc
            .assets
            .iter()
            .find(|(id, a)| **id != asset && a.path == path)
        {
            Some((other, _)) => Err(format!("{other} names {}", quote(path))),
            None => Ok(()),
        }
    }

    /// Refuse a move or a replace of a file another asset names too, which would change that
    /// asset with this one; only a file edited by hand names one path twice.
    fn unshared(&self, asset: AssetId, path: &str) -> Result<(), String> {
        match self
            .doc
            .assets
            .iter()
            .find(|(id, a)| **id != asset && a.path == path)
        {
            Some((other, _)) => Err(format!("{other} names {} too", quote(path))),
            None => Ok(()),
        }
    }

    /// Whether the store holds anything at the asset path, which is confined: a file, or, in
    /// memory as on disk, a folder of files below it or a file where a folder of it would be. A
    /// move goes to no such path.
    fn taken_at(&self, path: &str, files: &mut dyn Files) -> Result<bool, String> {
        match &self.store {
            Store::Memory(map) => Ok(held_in(map, path)),
            Store::Workspace(root) => {
                let full = writable(root, path, files)?;
                files.exists(&full)
            }
        }
    }

    /// Whether the store holds a file at the asset path, which is confined, and not a folder: what
    /// an asset's path names. A folder lists, where a file refuses to.
    fn file_at(&self, path: &str, files: &mut dyn Files) -> Result<bool, String> {
        match &self.store {
            Store::Memory(map) => Ok(map.contains_key(path)),
            Store::Workspace(root) => {
                let full = writable(root, path, files)?;
                Ok(files.exists(&full)? && files.list(&full).is_err())
            }
        }
    }

    /// The moves that take the files of the removed `assets` to the trash. An asset whose path
    /// fails confinement, whose file is missing or out of reach, or whose file another asset that
    /// stays names, moves nothing.
    fn trash_moves(
        &mut self,
        assets: &[AssetId],
        files: &mut dyn Files,
    ) -> Result<Vec<Move>, String> {
        let mut moves = Vec::new();
        for &id in assets {
            let path = self.doc.assets[&id].path.clone();
            let shared = self
                .doc
                .assets
                .iter()
                .any(|(other, a)| !assets.contains(other) && a.path == path);
            let moved = moves
                .iter()
                .any(|m: &Move| m.from == Place::Asset(path.clone()));
            // A file the store cannot reach, as one a symlink places outside the root, stays where
            // it is, so such an asset can still leave the table.
            if !asset_path_is_confined(&path)
                || shared
                || moved
                || !self.file_at(&path, files).unwrap_or(false)
            {
                continue;
            }
            moves.push(Move {
                from: Place::Asset(path),
                to: Place::Trash(self.trash_entry(&id.to_string(), files)?),
            });
        }
        Ok(moves)
    }

    /// What names the asset, as `assets` lists it: the areas whose image is it or whose text's
    /// own values or spans' own values set it as font, in the order of the slides and their trees,
    /// then the named styles, then the named span styles, whose font is it.
    fn asset_uses(&self, asset: AssetId) -> Vec<String> {
        let font = FontRef::Asset(asset);
        let mut uses: Vec<String> = self
            .doc
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .filter(|a| match &a.content {
                Content::Image(image) => image.asset == asset,
                Content::Text(text) => text.own_fonts().contains(&&font),
                Content::Equation(_) => false,
            })
            .map(|a| a.id.to_string())
            .collect();
        uses.extend(
            self.doc
                .styles
                .iter()
                .filter(|(_, s)| s.font == font)
                .map(|(name, _)| quote_always(name)),
        );
        uses.extend(
            self.doc
                .span_styles
                .iter()
                .filter(|(_, s)| s.font.as_ref() == Some(&font))
                .map(|(name, _)| format!("span:{}", quote_always(name))),
        );
        uses
    }

    /// The operations that make every image, text, named style and named span style naming `asset`
    /// name `other`.
    fn renaming_ops(&self, asset: AssetId, other: AssetId) -> Vec<Op> {
        let font = FontRef::Asset(asset);
        let mut ops = Vec::new();
        for area in self.doc.slides.iter().flat_map(|s| s.root.areas()) {
            let content = match &area.content {
                Content::Image(image) if image.asset == asset => Content::Image(ImageContent {
                    asset: other,
                    fit: image.fit,
                }),
                Content::Text(text) if text.own_fonts().contains(&&font) => {
                    Content::Text(text.with_font(&font, &FontRef::Asset(other)))
                }
                _ => continue,
            };
            ops.push(Op::SetContent {
                area: area.id,
                content,
            });
        }
        for (name, style) in &self.doc.styles {
            if style.font == font {
                ops.push(Op::SetStyle {
                    name: name.clone(),
                    style: TextStyle {
                        font: FontRef::Asset(other),
                        ..style.clone()
                    },
                });
            }
        }
        for (name, style) in &self.doc.span_styles {
            if style.font.as_ref() == Some(&font) {
                ops.push(Op::SetSpanStyleValues {
                    name: name.clone(),
                    style: rootpoint_model::SpanStyle {
                        font: Some(FontRef::Asset(other)),
                        ..style.clone()
                    },
                });
            }
        }
        ops
    }

    /// The reply of `assets`: each entry `a<id> <path> <bytes|missing> uses=<uses|->`, in the
    /// order of the ids, or, for `untracked`, the paths of the store's files no asset names that
    /// `listed_untracked` keeps; entries separated by `; `, as every list of a reply.
    fn list_assets(&self, filter: AssetFilter, files: &mut dyn Files) -> Result<String, String> {
        if filter == AssetFilter::Untracked {
            let named: BTreeSet<&str> = self.doc.assets.values().map(|a| a.path.as_str()).collect();
            let held = match &self.store {
                Store::Memory(map) => map.keys().cloned().collect(),
                Store::Workspace(root) => files.list(root)?,
            };
            return Ok(held
                .iter()
                .filter(|p| listed_untracked(p) && !named.contains(p.as_str()))
                .map(|p| quote(p))
                .collect::<Vec<_>>()
                .join("; "));
        }
        let mut entries = Vec::new();
        for (&id, asset) in &self.doc.assets {
            let uses = self.asset_uses(id);
            let size = self.asset_bytes(id, files).ok().map(|b| b.len());
            let keep = match filter {
                AssetFilter::All => true,
                AssetFilter::Unused => uses.is_empty(),
                AssetFilter::Missing => size.is_none(),
                AssetFilter::Untracked => unreachable!("handled above"),
            };
            if keep {
                entries.push(format!(
                    "{id} {} {} uses={}",
                    quote(&asset.path),
                    size.map_or("missing".to_owned(), |n| n.to_string()),
                    if uses.is_empty() {
                        "-".to_owned()
                    } else {
                        uses.join(",")
                    }
                ));
            }
        }
        Ok(entries.join("; "))
    }

    /// Apply an editing operation: its inverse joins the open group, or is an undo step of its
    /// own. The reply names the ids it created.
    fn edit(&mut self, op: Op, files: &mut dyn Files) -> Result<String, String> {
        self.edit_ops(vec![op], files)
    }

    /// Apply the operations of one command, all or none: their inverses join the open group, or
    /// are one undo step. The reply names the ids they created.
    fn edit_ops(&mut self, ops: Vec<Op>, files: &mut dyn Files) -> Result<String, String> {
        let before = ids(&self.doc);
        let revert = self.apply_step(Step::ops(ops), files)?;
        let last = revert.ops.first().cloned();
        self.record(revert, files);
        Ok(self.created(&before, last.as_ref()))
    }

    /// Apply a step of moves and operations, as one command, and record its revert.
    fn change(&mut self, step: Step, files: &mut dyn Files) -> Result<(), String> {
        let revert = self.apply_step(step, files)?;
        self.record(revert, files);
        Ok(())
    }

    /// Record the revert of a command: it joins the open group, or is an undo step of its own.
    /// The redo steps it clears take with them the trash's entries only they reach.
    fn record(&mut self, revert: Step, files: &mut dyn Files) {
        match &mut self.group {
            Some(group) => group.reverts.push(revert),
            None => {
                self.undo.push(revert);
                self.unsaved += 1;
            }
        }
        let moved = self.redo.iter().any(|s| !s.moves.is_empty());
        self.redo.clear();
        if moved {
            self.sweep_trash(files);
        }
    }

    /// Remove the trash's entries no step of the stacks reaches, so the trash holds a file only
    /// while an undo or a redo can bring it back, `design@rootpoint-protocol@asset-trash`.
    fn sweep_trash(&mut self, files: &mut dyn Files) {
        let group = self
            .group
            .iter()
            .flat_map(|g| g.reverts.iter().chain(&g.redo));
        let reachable: BTreeSet<&str> = self
            .undo
            .iter()
            .chain(&self.redo)
            .chain(group)
            .flat_map(|s| &s.moves)
            .flat_map(|m| [&m.from, &m.to])
            .filter_map(|p| match p {
                Place::Trash(entry) => Some(entry.as_str()),
                Place::Asset(_) => None,
            })
            .collect();
        match &self.store {
            Store::Memory(_) => {
                let reachable: BTreeSet<String> =
                    reachable.into_iter().map(str::to_owned).collect();
                self.trash.retain(|entry, _| reachable.contains(entry));
            }
            Store::Workspace(root) => {
                let Ok(trash) = trash_path(root, "", files) else {
                    return;
                };
                for entry in files.list(&trash).unwrap_or_default() {
                    if !reachable.contains(entry.as_str()) {
                        let _ = files.remove(&join(&trash, &entry));
                    }
                }
            }
        }
    }

    /// Put bytes in the trash under `entry`.
    fn put_in_trash(
        &mut self,
        entry: &str,
        bytes: Vec<u8>,
        files: &mut dyn Files,
    ) -> Result<(), String> {
        match &self.store {
            Store::Memory(_) => {
                self.trash.insert(entry.to_owned(), bytes);
                Ok(())
            }
            Store::Workspace(root) => {
                let path = trash_path(root, entry, files)?;
                files.write(&path, &bytes)
            }
        }
    }

    /// Remove an entry of the trash a failed step put there.
    fn drop_from_trash(&mut self, entry: &str, files: &mut dyn Files) {
        match &self.store {
            Store::Memory(_) => drop(self.trash.remove(entry)),
            Store::Workspace(root) => {
                if let Ok(path) = trash_path(root, entry, files) {
                    let _ = files.remove(&path);
                }
            }
        }
    }

    /// Write the presentation and its assets' files as a pack at `file`, every asset read by the
    /// store's check: an asset with no readable bytes is left out, and the reply names it after
    /// `missing=`. In memory the pack becomes the one `save` writes.
    fn write_pack(&mut self, file: &str, files: &mut dyn Files) -> Result<String, String> {
        let mut assets = Vec::new();
        let mut missing = Vec::new();
        for (&id, asset) in &self.doc.assets {
            match self.asset_bytes(id, files) {
                Ok(bytes) => assets.push((asset.path.clone(), bytes)),
                Err(_) => missing.push(id.to_string()),
            }
        }
        let bytes = crate::pack::write_pack(&save_ron(&self.doc), &assets)?;
        files.write(file, &bytes)?;
        if let Store::Memory(_) = self.store {
            self.pack = Some(file.to_owned());
            self.unsaved = 0;
        }
        Ok(if missing.is_empty() {
            "missing=-".to_owned()
        } else {
            format!("missing={}", missing.join(","))
        })
    }

    /// The asset path an import of the host's file `from` adds, and the bytes to write there when
    /// the store holds no file at it yet. In a workspace, a file below the root is taken at its
    /// path, with no copy, and `to`, when given, must be that path. Any other file goes at `to`, or
    /// at the root under its file name, with `-2`, `-3` before the extension where a file of other
    /// bytes, or a folder, holds the name, and the name itself where its file holds the same bytes.
    fn import(
        &mut self,
        from: &str,
        to: Option<String>,
        files: &mut dyn Files,
    ) -> Result<(String, Option<Vec<u8>>), String> {
        let bytes = files.read(from)?;
        if let Store::Workspace(root) = &self.store {
            let canonical = files.canonical(from)?;
            if let Some(below) = canonical.strip_prefix(&join(root, "")) {
                if to.as_ref().is_some_and(|to| to != below) {
                    return Err(format!(
                        "{} is below the root, at {}",
                        quote(from),
                        quote(below)
                    ));
                }
                if !asset_path_is_confined(below) {
                    return Err(unconfined(below));
                }
                return Ok((below.to_owned(), None));
            }
        }
        let name = match to {
            Some(to) => to,
            None => from.rsplit('/').next().unwrap_or(from).to_owned(),
        };
        if !asset_path_is_confined(&name) {
            return Err(unconfined(&name));
        }
        // A file where a folder of the name would be is met by every suffix of the name.
        if let Store::Memory(map) = &self.store
            && let Some(file) = map.keys().find(|k| name.starts_with(&format!("{k}/")))
        {
            return Err(format!("{} is a file, not a folder", quote(file)));
        }
        for n in 1.. {
            let candidate = if n == 1 {
                name.clone()
            } else {
                with_suffix(&name, n)
            };
            let held = match &self.store {
                // A folder at the name, or a file where its folder would be, is other bytes.
                Store::Memory(map) if !map.contains_key(&candidate) && held_in(map, &candidate) => {
                    continue;
                }
                Store::Memory(map) => map.get(&candidate).cloned(),
                Store::Workspace(root) => {
                    let full = writable(root, &candidate, files)?;
                    if !files.exists(&full)? {
                        None
                    } else {
                        match files.read(&full) {
                            Ok(bytes) => Some(bytes),
                            // A folder at the name is other bytes, as in memory.
                            Err(_) if files.list(&full).is_ok() => continue,
                            Err(e) => return Err(e),
                        }
                    }
                }
            };
            // A name no file holds is free, even where an asset names it: the file written there
            // is that asset's, which was drawn missing until now.
            match held {
                Some(held) if held == bytes => return Ok((candidate, None)),
                Some(_) => continue,
                None => return Ok((candidate, Some(bytes))),
            }
        }
        unreachable!("the suffixes do not run out")
    }

    /// The operation an editing command applies.
    fn operation(&self, command: Command) -> Result<Op, String> {
        Ok(match command {
            Command::AddSlide { at } => Op::AddSlide {
                at: at.unwrap_or(self.doc.slides.len()),
            },
            Command::RemoveSlide { slide } => Op::RemoveSlide { slide },
            Command::MoveSlide { slide, to } => Op::MoveSlide { slide, to },
            Command::DuplicateSlide { slide } => Op::DuplicateSlide { slide },
            Command::Split {
                node,
                axis,
                at,
                side,
            } => Op::Split {
                node,
                axis,
                at,
                side,
            },
            Command::Remove { node } => Op::Remove { node },
            Command::Divider { split, index, pos } => Op::MoveDivider { split, index, pos },
            Command::Swap { a, b } => Op::Swap { a, b },
            Command::Image { area, asset, fit } => {
                self.existing_asset(asset)?;
                Op::SetContent {
                    area,
                    content: Content::Image(ImageContent { asset, fit }),
                }
            }
            Command::Equation {
                area,
                formula,
                options,
            } => {
                let current = match self.doc.node(area) {
                    Some(Node::Area(Area {
                        content: Content::Equation(e),
                        ..
                    })) => e.clone(),
                    _ => self.doc.new_equation(Formula::empty()),
                };
                // A command built through the API is held to the grammar, so that it prints back
                // as a line that parses: a formula whose LaTeX reads back to it, and a size above 0.
                if Formula::parse(&formula.to_latex()).as_ref() != Ok(&formula) {
                    return Err("the formula's LaTeX does not read back to it".to_owned());
                }
                let mut equation = EquationContent { formula, ..current };
                for option in options {
                    match option {
                        EquationOption::Numbered(on) => equation.numbered = on,
                        EquationOption::Size(EquationSize::AtMost(size))
                            if !(size.is_finite() && size > 0.0) =>
                        {
                            return Err("an equation's size is a number above 0".to_owned());
                        }
                        EquationOption::Size(size) => equation.size = size,
                        EquationOption::Colour(colour) => equation.colour = colour,
                    }
                }
                Op::SetContent {
                    area,
                    content: Content::Equation(equation),
                }
            }
            Command::Clear { area } => Op::SetContent {
                area,
                content: Content::Text(TextContent {
                    root: rootpoint_model::TextNode::plain(String::new()),
                    style: self.text_style(area),
                }),
            },
            Command::Style(StyleCommand::Add { name, options }) => Op::AddStyle {
                style: self.style_values(TextStyle::default(), options)?,
                name,
            },
            Command::Style(StyleCommand::Set { name, options }) => {
                let current = self.doc.styles.get(&name).ok_or_else(|| no_style(&name))?;
                Op::SetStyle {
                    style: self.style_values(current.clone(), options)?,
                    name,
                }
            }
            Command::Style(StyleCommand::Rename { from, to }) => {
                // The operation refuses an area that names `to` too; the reply names the area.
                if let Some(area) = self.users(&to).first() {
                    return Err(format!("{area} names {}", quote_always(&to)));
                }
                Op::RenameStyle { from, to }
            }
            Command::Style(StyleCommand::Remove { name, into }) => {
                self.doc.styles.get(&name).ok_or_else(|| no_style(&name))?;
                match &into {
                    None => {
                        let users = self.users(&name);
                        if !users.is_empty() {
                            let users: Vec<String> = users.iter().map(|n| n.to_string()).collect();
                            return Err(format!(
                                "{} is used by {}",
                                quote_always(&name),
                                users.join(" ")
                            ));
                        }
                    }
                    Some(Replacement::Named(other)) if *other == name => {
                        return Err(format!("{} cannot replace itself", quote_always(&name)));
                    }
                    Some(Replacement::Named(other)) => self.usable_style(other)?,
                    Some(Replacement::Custom) => {}
                }
                Op::RemoveStyle { name, into }
            }
            Command::Insert { node, offset, text } => {
                let (area, node) = self.text_node(node)?;
                Op::InsertText {
                    area,
                    node,
                    offset,
                    text,
                }
            }
            Command::Delete { node, range } => {
                let (area, node) = self.text_node(node)?;
                Op::DeleteText { area, node, range }
            }
            Command::Wrap { node, range, style } => {
                let (area, node) = self.text_node(node)?;
                Op::WrapSpan {
                    area,
                    node,
                    range,
                    style: self.span_choice(SpanChoice::Custom(SpanStyle::default()), style)?,
                }
            }
            Command::Unwrap { span } => Op::UnwrapSpan {
                area: self.span_area(span)?.0,
                span,
            },
            Command::Span { span, style } => {
                // The grammar refuses a `span` with no option; a command built through the API is
                // held to the same rule, so that it prints back as a line that parses.
                if style.is_empty() {
                    return Err("missing a span's style option".to_owned());
                }
                let (area, current) = self.span_area(span)?;
                Op::SetSpanStyle {
                    area,
                    span,
                    style: self.span_choice(current, style)?,
                }
            }
            Command::SplitSpan { span, offset } => Op::SplitSpan {
                area: self.span_area(span)?.0,
                span,
                offset,
            },
            Command::SpanStyle(SpanStyleCommand::Add { name, options }) => Op::AddSpanStyle {
                style: self.span_values(SpanStyle::default(), options)?,
                name,
            },
            Command::SpanStyle(SpanStyleCommand::Set { name, options }) => {
                let current = self
                    .doc
                    .span_styles
                    .get(&name)
                    .ok_or_else(|| no_span_style(&name))?;
                Op::SetSpanStyleValues {
                    style: self.span_values(current.clone(), options)?,
                    name,
                }
            }
            Command::SpanStyle(SpanStyleCommand::Rename { from, to }) => {
                // The operation refuses a span that names `to` too; the reply names the span.
                if let Some((_, span)) = span_style_users(&self.doc, &to).first() {
                    return Err(format!("{span} names {}", quote_always(&to)));
                }
                Op::RenameSpanStyle { from, to }
            }
            Command::SpanStyle(SpanStyleCommand::Remove { name, into }) => {
                let removed = self
                    .doc
                    .span_styles
                    .get(&name)
                    .ok_or_else(|| no_span_style(&name))?;
                let users = span_style_users(&self.doc, &name);
                match &into {
                    None if !users.is_empty() => {
                        let users: Vec<String> = users.iter().map(|(_, t)| t.to_string()).collect();
                        return Err(format!(
                            "{} is used by {}",
                            quote_always(&name),
                            users.join(" ")
                        ));
                    }
                    None => {}
                    Some(Replacement::Named(other)) if *other == name => {
                        return Err(format!("{} cannot replace itself", quote_always(&name)));
                    }
                    Some(Replacement::Named(other)) => self.usable_span_style(other)?,
                    // A copy of values that set nothing would make each user a custom span that
                    // sets nothing.
                    Some(Replacement::Custom)
                        if *removed == SpanStyle::default() && !users.is_empty() =>
                    {
                        return Err(format!(
                            "{} sets no value, so its users cannot hold a copy: \
                             `into=\"<name>\"` names another",
                            quote_always(&name)
                        ));
                    }
                    Some(Replacement::Custom) => {}
                }
                Op::RemoveSpanStyle { name, into }
            }
            other => return Err(format!("`{other}` applies no operation")),
        })
    }

    /// The operations of `text`: with a tree, `Op::SetContent` gives the area its style and an
    /// empty text, then `Op::SetText` the tree, every span with a new id; with none, the area keeps
    /// its tree and its ids, and `Op::SetContent` gives it the style alone, refused on an image.
    fn text_ops(
        &self,
        area: NodeId,
        root: Option<Vec<TreeItem>>,
        style: Vec<StyleOption>,
    ) -> Result<Vec<Op>, String> {
        // The grammar refuses these sizes; a command built through the API is held to the same
        // rule, so that it prints back as a line that parses.
        if style
            .iter()
            .any(|o| matches!(o, StyleOption::Size(s) if !(s.is_finite() && *s > 0.0)))
        {
            return Err("a text size is a number above 0".to_owned());
        }
        if root.is_none() && style.is_empty() {
            return Err("`text` with no tree changes the style: give it an option".to_owned());
        }
        let mut choice = self.text_style(area);
        for option in style {
            choice = match option {
                StyleOption::Named(name) => {
                    self.usable_style(&name)?;
                    StyleChoice::Named(name)
                }
                StyleOption::Custom => StyleChoice::Custom(self.doc.resolve_style(&choice)),
                // Any other option changes the values drawn, so a named style forks to a custom
                // copy: no command but `style set` changes a shared style,
                // `design@rootpoint-model@named-styles`.
                value => {
                    self.usable_option(&value)?;
                    let mut values = self.doc.resolve_style(&choice);
                    apply_style(&mut values, value);
                    StyleChoice::Custom(values)
                }
            };
        }
        let content = |root: TextNode| Op::SetContent {
            area,
            content: Content::Text(TextContent {
                root,
                style: choice.clone(),
            }),
        };
        Ok(match root {
            Some(items) => vec![
                content(TextNode::default()),
                Op::SetText {
                    area,
                    root: self.text_tree(items)?,
                },
            ],
            None => {
                let current = match self.doc.node(area) {
                    Some(Node::Area(Area {
                        content: Content::Text(t),
                        ..
                    })) => t.root.clone(),
                    // An image has no text style to change, and no tree to keep.
                    Some(Node::Area(_)) => {
                        return Err(format!(
                            "{area} holds an image: `text` with a tree makes it a text"
                        ));
                    }
                    // A node that is no area is refused by `Op::SetContent`.
                    _ => TextNode::default(),
                };
                vec![content(current)]
            }
        })
    }

    /// The text node a tree's items give, each span's style refused as `wrap` refuses it. Its ids
    /// are placeholders, which `Op::SetText` replaces, and `apply` merges its adjacent runs.
    fn text_tree(&self, items: Vec<TreeItem>) -> Result<TextNode, String> {
        Ok(TextNode {
            items: items
                .into_iter()
                .map(|item| match item {
                    TreeItem::Chars(text) => Ok(TextItem::Chars(text)),
                    TreeItem::Span(span) => Ok(TextItem::Span(Span {
                        id: SpanId(0),
                        style: self
                            .span_choice(SpanChoice::Custom(SpanStyle::default()), span.style)?,
                        text: self.text_tree(span.items)?,
                    })),
                })
                .collect::<Result<_, String>>()?,
        })
    }

    /// The area and the text node a command names: an area's root, or a span found by its id,
    /// which is unique in the document.
    fn text_node(&self, target: TextTarget) -> Result<(NodeId, TextNodeRef), String> {
        Ok(match target {
            TextTarget::Root(area) => (area, TextNodeRef::Root),
            TextTarget::Span(span) => (self.span_area(span)?.0, TextNodeRef::Span(span)),
        })
    }

    /// The area whose text holds the span, and the span's style.
    fn span_area(&self, span: SpanId) -> Result<(NodeId, SpanChoice), String> {
        self.doc
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .find_map(|a| match &a.content {
                Content::Text(text) => text
                    .root
                    .spans()
                    .into_iter()
                    .find(|s| s.id == span)
                    .map(|s| (a.id, s.style.clone())),
                Content::Image(_) | Content::Equation(_) => None,
            })
            .ok_or(format!("no span {span}"))
    }

    /// The style `options` give a span whose style was `current`, applied in order as `text`'s
    /// are: a value set on a named span makes it custom, with the named values and that change,
    /// `design@rootpoint-model@named-span-styles`. A custom style left setting nothing is refused,
    /// since its span would change nothing, `design@rootpoint-model@text-tree-normal-form`.
    fn span_choice(
        &self,
        current: SpanChoice,
        options: Vec<SpanOption>,
    ) -> Result<SpanChoice, String> {
        let mut choice = current;
        for option in options {
            choice = match option {
                SpanOption::Named(name) => {
                    self.usable_span_style(&name)?;
                    SpanChoice::Named(name)
                }
                SpanOption::Custom => {
                    SpanChoice::Custom(self.doc.resolve_span(&choice).unwrap_or_default())
                }
                value => {
                    let mut values = self.doc.resolve_span(&choice).unwrap_or_default();
                    self.apply_span_value(&mut values, value)?;
                    SpanChoice::Custom(values)
                }
            };
        }
        if choice == SpanChoice::Custom(SpanStyle::default()) {
            return Err(
                "a custom span sets one value at least: `unwrap` puts a span's text in its parent"
                    .to_owned(),
            );
        }
        Ok(choice)
    }

    /// The values `options` give a span style whose values were `values`. `style=` and `custom`
    /// choose a span's style, not a value, and are refused, as the grammar refuses them.
    fn span_values(
        &self,
        mut values: SpanStyle,
        options: Vec<SpanOption>,
    ) -> Result<SpanStyle, String> {
        for option in options {
            if matches!(option, SpanOption::Named(_) | SpanOption::Custom) {
                return Err(format!(
                    "`{option}` chooses a span's style, not a value of a span style"
                ));
            }
            self.apply_span_value(&mut values, option)?;
        }
        Ok(values)
    }

    /// Apply a value option to a span style's values, refusing a font asset the table lacks and a
    /// size the grammar refuses. `style=` and `custom` choose a span's style, which the caller
    /// handles; they change no value.
    fn apply_span_value(&self, values: &mut SpanStyle, option: SpanOption) -> Result<(), String> {
        match option {
            SpanOption::Named(_) | SpanOption::Custom => {}
            SpanOption::Size(size) if !(size.is_finite() && size > 0.0) => {
                return Err("a text size is a number above 0".to_owned());
            }
            SpanOption::Size(size) => values.size = Some(size),
            SpanOption::Bold(on) => values.bold = Some(on),
            SpanOption::Italic(on) => values.italic = Some(on),
            SpanOption::Colour(c) => values.colour = Some(c),
            SpanOption::Font(None) => values.font = Some(FontRef::Builtin("sans".to_owned())),
            SpanOption::Font(Some(asset)) => {
                self.existing_asset(asset)?;
                values.font = Some(FontRef::Asset(asset));
            }
            SpanOption::Inherit(SpanField::Font) => values.font = None,
            SpanOption::Inherit(SpanField::Size) => values.size = None,
            SpanOption::Inherit(SpanField::Bold) => values.bold = None,
            SpanOption::Inherit(SpanField::Italic) => values.italic = None,
            SpanOption::Inherit(SpanField::Colour) => values.colour = None,
        }
        Ok(())
    }

    /// Refuse a span style a span would name and draw with a stand-in: one the table lacks, or
    /// whose font is an asset the table lacks or a family the binary does not carry,
    /// `design@rootpoint-protocol@no-new-breach`.
    fn usable_span_style(&self, name: &str) -> Result<(), String> {
        let style = self
            .doc
            .span_styles
            .get(name)
            .ok_or_else(|| no_span_style(name))?;
        match &style.font {
            Some(FontRef::Asset(asset)) => self.existing_asset(*asset),
            Some(FontRef::Builtin(family)) if !BUILTIN_FAMILIES.contains(&family.as_str()) => Err(
                format!("{} names no built-in family {family}", quote_always(name)),
            ),
            _ => Ok(()),
        }
    }

    /// Refuse a command that names an asset the table lacks, so that no command creates a cosmetic
    /// breach; the operation accepts one, since an undo may bring back what a file was loaded with,
    /// `design@rootpoint-protocol@no-new-breach`.
    fn existing_asset(&self, asset: AssetId) -> Result<(), String> {
        if self.doc.assets.contains_key(&asset) {
            Ok(())
        } else {
            Err(format!("no asset {asset}"))
        }
    }

    /// The style of the text `area` holds, or, when it holds none, the style a new area takes.
    fn text_style(&self, area: NodeId) -> StyleChoice {
        match self.doc.node(area) {
            Some(Node::Area(Area {
                content: Content::Text(t),
                ..
            })) => t.style.clone(),
            _ => self.doc.new_area_style(),
        }
    }

    /// Refuse a style a text would name and draw with a stand-in: one the table lacks, or whose
    /// font is an asset the table lacks or a family the binary does not carry,
    /// `design@rootpoint-protocol@no-new-breach`.
    fn usable_style(&self, name: &str) -> Result<(), String> {
        let style = self.doc.styles.get(name).ok_or_else(|| no_style(name))?;
        if self.doc.draws_as_asked(style) {
            return Ok(());
        }
        match &style.font {
            FontRef::Asset(asset) => Err(format!("no asset {asset}")),
            FontRef::Builtin(family) => Err(format!(
                "{} names no built-in family {family}",
                quote_always(name)
            )),
        }
    }

    /// Refuse a value option that names an asset the table lacks.
    fn usable_option(&self, option: &StyleOption) -> Result<(), String> {
        match option {
            StyleOption::Font(Some(asset)) => self.existing_asset(*asset),
            _ => Ok(()),
        }
    }

    /// The values `options` give a style whose values were `values`.
    fn style_values(
        &self,
        mut values: TextStyle,
        options: Vec<StyleOption>,
    ) -> Result<TextStyle, String> {
        for option in options {
            self.usable_option(&option)?;
            apply_style(&mut values, option);
        }
        Ok(values)
    }

    /// The text areas naming the style `name`, in the order of the slides and of their trees.
    fn users(&self, name: &str) -> Vec<NodeId> {
        self.doc
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .filter(|a| matches!(&a.content, Content::Text(TextContent { style: StyleChoice::Named(n), .. }) if n == name))
            .map(|a| a.id)
            .collect()
    }

    /// Apply an operation, then drop from the view a node or a slide it removed.
    fn apply_op(&mut self, op: Op) -> Result<Op, String> {
        let inverse = apply(&mut self.doc, op).map_err(|e| format!("{e:?}"))?;
        if let Some(node) = self.view.selected
            && self.doc.node(node).is_none()
        {
            self.view.selected = None;
        }
        if let Some(slide) = self.view.presenting
            && self.doc.slide(slide).is_none()
        {
            self.view.presenting = None;
        }
        if let Some(slide) = self.view.shown
            && self.doc.slide(slide).is_none()
        {
            self.view.shown = None;
        }
        Ok(inverse)
    }

    /// Apply the moves of a step, then its operations, each in order, and return the step that
    /// reverts it. A step that fails partway moves its files back and leaves the document and the
    /// view as they were, so a step applies whole or not at all, but where a move back fails too,
    /// which the reply names.
    fn apply_step(&mut self, step: Step, files: &mut dyn Files) -> Result<Step, String> {
        let mut moved: Vec<Move> = Vec::with_capacity(step.moves.len());
        let mut failed = None;
        for m in step.moves {
            match self.move_file(&m, files) {
                Ok(()) => moved.push(m),
                Err(e) => {
                    failed = Some(e);
                    break;
                }
            }
        }
        let (doc, view) = (self.doc.clone(), self.view.clone());
        let mut back = Vec::with_capacity(step.ops.len());
        if failed.is_none() {
            for op in step.ops {
                match self.apply_op(op) {
                    Ok(inverse) => back.push(inverse),
                    Err(e) => {
                        (self.doc, self.view) = (doc, view);
                        failed = Some(e);
                        break;
                    }
                }
            }
        }
        let back_moves: Vec<Move> = moved
            .into_iter()
            .rev()
            .map(|m| Move {
                from: m.to,
                to: m.from,
            })
            .collect();
        if let Some(e) = failed {
            // The moves made are taken back; a move back that fails leaves its file in the place
            // it reached, which the reply names with the first failure.
            for m in &back_moves {
                if let Err(back) = self.move_file(m, files) {
                    return Err(format!("{e}; and then {back}"));
                }
            }
            return Err(e);
        }
        back.reverse();
        Ok(Step {
            ops: back,
            moves: back_moves,
        })
    }

    /// Move one file of the store. The place it goes to must hold no file, since a native rename
    /// would replace it; in a workspace, an asset path is checked as every access is,
    /// `design@rootpoint-protocol@store-confines-access`.
    fn move_file(&mut self, m: &Move, files: &mut dyn Files) -> Result<(), String> {
        self.moved += 1;
        let name = |place: &Place| match place {
            Place::Asset(path) => quote(path),
            Place::Trash(entry) => format!("the trash's {entry}"),
        };
        match &mut self.store {
            Store::Memory(map) => {
                let taken = match &m.to {
                    Place::Asset(path) => held_in(map, path),
                    Place::Trash(entry) => self.trash.contains_key(entry),
                };
                if taken {
                    return Err(format!("{} holds a file", name(&m.to)));
                }
                let bytes = match &m.from {
                    Place::Asset(path) => map.remove(path),
                    Place::Trash(entry) => self.trash.remove(entry),
                }
                .ok_or_else(|| format!("{}: no such file", name(&m.from)))?;
                match &m.to {
                    Place::Asset(path) => map.insert(path.clone(), bytes),
                    Place::Trash(entry) => self.trash.insert(entry.clone(), bytes),
                };
                Ok(())
            }
            Store::Workspace(root) => {
                let root = root.clone();
                let from = match &m.from {
                    Place::Asset(path) => readable(&root, path, files)?,
                    Place::Trash(entry) => trash_path(&root, entry, files)?,
                };
                let to = match &m.to {
                    Place::Asset(path) => writable(&root, path, files)?,
                    Place::Trash(entry) => trash_path(&root, entry, files)?,
                };
                if files.exists(&to)? {
                    return Err(format!("{} holds a file", name(&m.to)));
                }
                files.rename(&from, &to)
            }
        }
    }

    /// A name for a new entry of the trash: `<prefix>-<n>`, the prefix an asset's id or `import`,
    /// numbered by the session's entries, and never one the trash holds, as an entry an open failed
    /// to empty.
    fn trash_entry(&mut self, prefix: &str, files: &mut dyn Files) -> Result<String, String> {
        loop {
            self.trashed += 1;
            let entry = format!("{prefix}-{}", self.trashed);
            let taken = match &self.store {
                Store::Memory(_) => self.trash.contains_key(&entry),
                Store::Workspace(root) => {
                    let path = trash_path(root, &entry, files)?;
                    files.exists(&path)?
                }
            };
            if !taken {
                return Ok(entry);
            }
        }
    }

    /// Close the open group, if any: its edits become one undo step, and an empty group none.
    /// The group's copy of the redo stack, which `cancel` would have restored, leaves with it, and
    /// the trash's entries only that copy reached with it.
    fn end_group(&mut self, files: &mut dyn Files) {
        if let Some(group) = self.group.take() {
            let moved = group.redo.iter().any(|s| !s.moves.is_empty());
            if !group.reverts.is_empty() {
                self.undo.push(Step::undoing(group.reverts));
                self.unsaved += 1;
            }
            if moved {
                self.sweep_trash(files);
            }
        }
    }

    /// An asset's bytes: its entry of the map in memory, or its file below a workspace's root.
    /// A path that fails confinement, or whose file a symlink places outside the root, is refused,
    /// whatever the file names, so no read reaches outside the root
    /// (`design@rootpoint-protocol@store-confines-access`). It is public so that a host decoding
    /// assets, as the app does, reads them by this one rule and never joins a path itself.
    pub fn asset_bytes(&self, id: AssetId, files: &mut dyn Files) -> Result<Vec<u8>, String> {
        let asset = self.doc.assets.get(&id).ok_or(format!("no asset {id}"))?;
        match &self.store {
            Store::Memory(map) => {
                if !asset_path_is_confined(&asset.path) {
                    return Err(unconfined(&asset.path));
                }
                map.get(&asset.path)
                    .cloned()
                    .ok_or(format!("{}: no such file", quote(&asset.path)))
            }
            Store::Workspace(root) => {
                let full = readable(root, &asset.path, files)?;
                files.read(&full)
            }
        }
    }

    /// The ids an operation created, as its reply states them: a new slide and its root; or the
    /// new area of a split, and the new split when the node was wrapped; or the new spans, in the
    /// order of the text; or a new asset.
    fn created(&self, before: &BTreeSet<Id>, inverse: Option<&Op>) -> String {
        let new: Vec<Id> = ids(&self.doc).difference(before).copied().collect();
        if let Some(slide) = new.iter().find_map(|i| match i {
            Id::Slide(s) => self.doc.slide(*s),
            _ => None,
        }) {
            return format!("{} root={}", slide.id, slide.root.id());
        }
        let mut words = Vec::new();
        for id in &new {
            match id {
                Id::Area(n) => words.insert(0, format!("new={n}")),
                Id::Split(n) => words.push(format!("split={n}")),
                Id::Span(t) => words.push(t.to_string()),
                Id::Slide(_) => {}
            }
        }
        if let Some(Op::RemoveAsset { id }) = inverse {
            words.push(id.to_string());
        }
        words.join(" ")
    }
}

/// The ids of a document, by kind, for the replies that name the ones a command created.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Id {
    Slide(SlideId),
    Split(NodeId),
    Area(NodeId),
    Span(SpanId),
}

fn ids(doc: &Document) -> BTreeSet<Id> {
    fn walk(node: &Node, out: &mut BTreeSet<Id>) {
        match node {
            Node::Area(a) => {
                out.insert(Id::Area(a.id));
                if let Content::Text(text) = &a.content {
                    out.extend(text.root.spans().iter().map(|s| Id::Span(s.id)));
                }
            }
            Node::Split(s) => {
                out.insert(Id::Split(s.id));
                s.children.iter().for_each(|c| walk(c, out));
            }
        }
    }
    let mut out = BTreeSet::new();
    for slide in &doc.slides {
        out.insert(Id::Slide(slide.id));
        walk(&slide.root, &mut out);
    }
    out
}

/// A render's report, as its reply states it: the text areas whose font fell back, the image
/// assets with no image, the text areas with a character no font has, the text areas naming a
/// style the table lacks, and the spans naming a span style the table lacks.
fn report_words(report: &Report) -> String {
    fn list<T: fmt::Display>(items: &[T]) -> String {
        if items.is_empty() {
            "-".to_owned()
        } else {
            items
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",")
        }
    }
    let styles: Vec<String> = report
        .missing_styles
        .iter()
        .map(|(area, name)| format!("{area}:{}", quote(name)))
        .collect();
    let span_styles: Vec<String> = report
        .missing_span_styles
        .iter()
        .map(|(_, span, name)| format!("{span}:{}", quote(name)))
        .collect();
    format!(
        "fallback={} missing={} glyphs={} styles={} span-styles={} commands={}",
        list(&report.font_fallbacks),
        list(&report.missing_images),
        list(&report.missing_glyphs),
        list(&styles),
        list(&span_styles),
        list(&report.unknown_commands)
    )
}

/// The refusal of a style the table lacks.
fn no_style(name: &str) -> String {
    format!("no style {}", quote_always(name))
}

/// The refusal of a span style the table lacks.
fn no_span_style(name: &str) -> String {
    format!("no span style {}", quote_always(name))
}

/// A style's every value, as the options `style add` takes, in a fixed order. A built-in family the
/// binary does not carry, which only a file edited by hand holds, is written as its name.
fn style_options(style: &TextStyle) -> String {
    let mut words: Vec<String> = StyleOption::values_of(style)
        .iter()
        .map(|o| o.to_string())
        .collect();
    if let FontRef::Builtin(name) = &style.font
        && name != "sans"
    {
        words.push(format!("font={}", quote(name)));
    }
    words.join(" ")
}

/// Apply a value option to a style's values. `style=` and `custom` choose a text's style, which
/// the caller handles; they change no value.
fn apply_style(style: &mut TextStyle, option: StyleOption) {
    match option {
        StyleOption::Named(_) | StyleOption::Custom => {}
        StyleOption::Size(size) => style.size = size,
        StyleOption::Bold(on) => style.bold = on,
        StyleOption::Italic(on) => style.italic = on,
        StyleOption::Colour(c) => style.colour = c,
        StyleOption::Align(a) => style.h_align = a,
        StyleOption::VAlign(v) => style.v_align = v,
        StyleOption::Font(None) => style.font = FontRef::Builtin("sans".to_owned()),
        StyleOption::Font(Some(a)) => style.font = FontRef::Asset(a),
    }
}

/// The presentation of a file of the host.
fn read_presentation(file: &str, files: &mut dyn Files) -> Result<Document, String> {
    let bytes = files.read(file)?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{file} is not UTF-8"))?;
    load_ron(&text).map_err(|e| format!("{file}: {e:?}"))
}

/// Whether a map of files holds anything at a path: a file, a file below it, or a file where one
/// of its folders would be, as a directory of files on disk would.
fn held_in(map: &BTreeMap<String, Vec<u8>>, path: &str) -> bool {
    map.contains_key(path)
        || map
            .keys()
            .any(|k| k.starts_with(&format!("{path}/")) || path.starts_with(&format!("{k}/")))
}

/// The host path of an entry of a workspace's trash, or of the trash itself for an empty entry,
/// refused when `.rootpoint/` or its `trash/` is not where its path says, as a symlink would place
/// it, so that no move, write or removal of the trash reaches outside the root,
/// `design@rootpoint-protocol@store-confines-access`. The root is canonical.
fn trash_path(root: &str, entry: &str, files: &mut dyn Files) -> Result<String, String> {
    for dir in [".rootpoint", TRASH] {
        let full = join(root, dir);
        if files.exists(&full)? && files.canonical(&full)? != full {
            return Err(format!("{full} leads out of the root"));
        }
    }
    Ok(if entry.is_empty() {
        join(root, TRASH)
    } else {
        join(root, &format!("{TRASH}/{entry}"))
    })
}

/// Whether `assets untracked` lists a file of the store that no asset names: one at a confined
/// path, none of whose components starts with `.`, so that the files of `.git/` and a
/// `.gitignore` are left out with the store's own, `design@rootpoint-protocol@table-of-added-assets`.
pub fn listed_untracked(path: &str) -> bool {
    asset_path_is_confined(path) && !path.split('/').any(|c| c.starts_with('.'))
}
