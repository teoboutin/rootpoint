//! A session: one document, its undo history and a view state, changed by commands.

use std::fmt;
use std::path::{Path, PathBuf};

use std::collections::{BTreeMap, BTreeSet};

use rootpoint_model::{
    Asset, AssetId, AssetSource, Content, Document, FontRef, ImageContent, Node, NodeId, Op,
    SlideId, TextContent, TextStyle, apply, load_ron, save_ron,
};
use rootpoint_render::{FontSet, ImageSet, Precision, Report, render_slide};

use crate::command::{Command, StyleOption, parse_command};
use crate::files::Files;
use crate::save::{extension, inside, normalize, relative, with_suffix};
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
/// and the slide presenter mode shows, if it is on. With no slide shown, the editor shows the
/// first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct View {
    pub selected: Option<NodeId>,
    pub shown: Option<SlideId>,
    pub presenting: Option<SlideId>,
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

/// One undo step: the operations that revert one command, or every command of one group, applied
/// in order.
type Step = Vec<Op>;

/// A group opened by `begin`: the inverses of its edits, in the order they were applied, and the
/// redo stack as it was at the `begin`, which `cancel` restores.
struct Group {
    inverses: Vec<Op>,
    redo: Vec<Step>,
}

/// One document, its undo and redo stacks, and its view, changed only by commands.
pub struct Session {
    doc: Document,
    /// The directory of the presentation file, against which asset paths are read.
    dir: PathBuf,
    undo: Vec<Step>,
    redo: Vec<Step>,
    group: Option<Group>,
    view: View,
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
            dir: PathBuf::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            group: None,
            view: View::default(),
        }
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    /// The directory an asset's path is read against: the presentation's, or the working
    /// directory, the empty path, before a first save or load.
    pub fn directory(&self) -> &Path {
        &self.dir
    }

    /// Run one line: a command's reply, or `None` for a blank line or a comment, which starts
    /// with `#`, by the rule of `read_line`.
    pub fn execute(&mut self, line: &str, files: &mut dyn Files) -> Option<Reply> {
        Some(match read_line(line.as_bytes())? {
            Ok(command) => self.run(command, files),
            Err(reply) => reply,
        })
    }

    /// Whether a group opened by `begin` is open.
    pub fn group_open(&self) -> bool {
        self.group.is_some()
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
            Command::Save { path } => {
                self.save(&path, files)?;
                Ok(String::new())
            }
            Command::Sample => {
                let doc = crate::sample::sample()?;
                *self = Session::new();
                self.doc = doc;
                Ok(String::new())
            }
            Command::Load { path } => {
                let bytes = files.read(&path)?;
                let text = String::from_utf8(bytes).map_err(|_| format!("{path} is not UTF-8"))?;
                let doc = load_ron(&text).map_err(|e| format!("{path}: {e:?}"))?;
                *self = Session::new();
                self.doc = doc;
                self.dir = parent(&path);
                Ok(String::new())
            }
            Command::Tree { slide } => {
                let slides: Vec<String> = match slide {
                    Some(id) => {
                        let s = self.doc.slide(id).ok_or(format!("no slide {id}"))?;
                        vec![slide_tree(s)]
                    }
                    None => self.doc.slides.iter().map(slide_tree).collect(),
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
                        Content::Text(t) => {
                            if let FontRef::Asset(id) = t.style.font
                                && let Ok(bytes) = self.asset_bytes(id, files)
                            {
                                let _ = fonts.insert_bytes(id, bytes);
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
                self.end_group();
                let undo = matches!(command, Command::Undo);
                let from = if undo { &mut self.undo } else { &mut self.redo };
                let step = from.pop().ok_or(format!("nothing to {command}"))?;
                match self.apply_step(step.clone()) {
                    Ok(back) if undo => self.redo.push(back),
                    Ok(back) => self.undo.push(back),
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
                    inverses: Vec::new(),
                    redo: self.redo.clone(),
                });
                Ok(String::new())
            }
            Command::End => {
                self.group.as_ref().ok_or("no group is open")?;
                self.end_group();
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
                let step: Step = group.inverses.iter().rev().cloned().collect();
                self.apply_step(step)?;
                let group = self.group.take().expect("checked above");
                self.redo = group.redo;
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
            Command::View => {
                let show = |id: Option<String>| id.unwrap_or_else(|| "-".to_owned());
                Ok(format!(
                    "selected={} shown={} presenting={}",
                    show(self.view.selected.map(|n| n.to_string())),
                    show(self.view.shown.map(|s| s.to_string())),
                    show(self.view.presenting.map(|s| s.to_string()))
                ))
            }
            editing => {
                let op = self.operation(editing)?;
                let before = ids(&self.doc);
                let inverse = self.apply_op(op)?;
                match &mut self.group {
                    Some(group) => group.inverses.push(inverse.clone()),
                    None => self.undo.push(vec![inverse.clone()]),
                }
                self.redo.clear();
                Ok(self.created(&before, &inverse))
            }
        }
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
            Command::Text { area, text, style } => {
                // The grammar refuses these sizes; a command built through the API is held to the
                // same rule, so that it prints back as a line that parses.
                if style
                    .iter()
                    .any(|o| matches!(o, StyleOption::Size(s) if !(s.is_finite() && *s > 0.0)))
                {
                    return Err("a text size is a number above 0".to_owned());
                }
                for option in &style {
                    if let StyleOption::Font(Some(asset)) = option {
                        self.existing_asset(*asset)?;
                    }
                }
                let mut current = self.text_style(area);
                for option in style {
                    apply_style(&mut current, option);
                }
                Op::SetContent {
                    area,
                    content: Content::Text(TextContent {
                        text,
                        style: current,
                    }),
                }
            }
            Command::Image { area, asset, fit } => {
                self.existing_asset(asset)?;
                Op::SetContent {
                    area,
                    content: Content::Image(ImageContent { asset, fit }),
                }
            }
            Command::Clear { area } => Op::SetContent {
                area,
                content: Content::Text(TextContent {
                    text: String::new(),
                    style: self.text_style(area),
                }),
            },
            Command::Asset { path } => Op::AddAsset {
                asset: Asset {
                    source: AssetSource::Path(path),
                },
            },
            Command::RemoveAsset { asset } => Op::RemoveAsset { id: asset },
            other => return Err(format!("`{other}` applies no operation")),
        })
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

    /// The style of the text `area` holds, or the default style when it holds none.
    fn text_style(&self, area: NodeId) -> TextStyle {
        match self.doc.node(area) {
            Some(Node::Area(a)) => match &a.content {
                Content::Text(t) => t.style.clone(),
                Content::Image(_) => TextStyle::default(),
            },
            _ => TextStyle::default(),
        }
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

    /// Apply the operations of a step in order, and return the step that reverts it. A step that
    /// fails partway leaves the document and the view as they were.
    fn apply_step(&mut self, step: Step) -> Result<Step, String> {
        let (doc, view) = (self.doc.clone(), self.view.clone());
        let mut back = Vec::with_capacity(step.len());
        for op in step {
            match self.apply_op(op) {
                Ok(inverse) => back.push(inverse),
                Err(e) => {
                    (self.doc, self.view) = (doc, view);
                    return Err(e);
                }
            }
        }
        back.reverse();
        Ok(back)
    }

    /// Save the document to `path`, so that every asset is readable beside the saved file: an
    /// asset whose file lies in the saved file's directory keeps it, and any other is written
    /// there, under a name no other file holds. Every asset to write is read before anything is
    /// written, so a save that cannot read one writes nothing.
    fn save(&mut self, path: &str, files: &mut dyn Files) -> Result<(), String> {
        let target = normalize(&parent(path));
        let own = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        // The new source of each asset whose source changes, and the files to write.
        let mut sources: BTreeMap<AssetId, String> = BTreeMap::new();
        let mut writes: Vec<(String, Vec<u8>)> = Vec::new();
        let mut kept = BTreeSet::new();
        let mut copied = Vec::new();
        for (&id, asset) in &self.doc.assets {
            match &asset.source {
                AssetSource::Path(p) => {
                    let full = normalize(&self.dir.join(p));
                    if let Some(rel) = inside(&full, &target) {
                        if rel != *p {
                            sources.insert(id, rel.clone());
                        }
                        kept.insert(rel);
                        continue;
                    }
                    let bytes = files
                        .read(&full.to_string_lossy())
                        .map_err(|e| format!("{id}: {e}"))?;
                    let name = Path::new(p)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| format!("asset-{}{}", id.0, extension(&bytes)));
                    copied.push((id, name, bytes));
                }
                AssetSource::Embedded(bytes) => {
                    let name = format!("asset-{}{}", id.0, extension(bytes));
                    copied.push((id, name, bytes.clone()));
                }
            }
        }
        // The names this save gives, each with the bytes it holds once the save is done.
        let mut given: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        for (id, name, bytes) in copied {
            let mut n = 1;
            let chosen = loop {
                let candidate = if n == 1 {
                    name.clone()
                } else {
                    with_suffix(&name, n)
                };
                n += 1;
                if Some(&candidate) == own.as_ref() {
                    continue;
                }
                if let Some(held) = given.get(&candidate) {
                    if *held == bytes {
                        break candidate;
                    }
                    continue;
                }
                let file = target.join(&candidate).to_string_lossy().into_owned();
                if files.exists(&file).map_err(|e| format!("{id}: {e}"))? {
                    let held = files.read(&file).map_err(|e| format!("{id}: {e}"))?;
                    if held == bytes {
                        given.insert(candidate.clone(), bytes);
                        break candidate;
                    }
                    continue;
                }
                // A kept asset's file that is missing still holds its name.
                if kept.contains(&candidate) {
                    continue;
                }
                given.insert(candidate.clone(), bytes.clone());
                writes.push((file, bytes));
                break candidate;
            };
            if self.doc.assets[&id].source != AssetSource::Path(chosen.clone()) {
                sources.insert(id, chosen);
            }
        }
        let mut doc = self.doc.clone();
        for (&id, source) in &sources {
            let op = Op::SetAssetSource {
                id,
                source: AssetSource::Path(source.clone()),
            };
            apply(&mut doc, op).map_err(|e| format!("{e:?}"))?;
        }
        for (file, bytes) in &writes {
            files.write(file, bytes)?;
        }
        files.write(path, save_ron(&doc).as_bytes())?;
        // The new sources are no undo step: an undo to an old path, read against the new
        // directory, would name another file. The operations the stacks hold are given the same
        // sources instead, and an asset only they hold a path that names the same file.
        let old_dir = std::mem::replace(&mut self.dir, target);
        self.doc = doc;
        let group = self
            .group
            .iter_mut()
            .flat_map(|g| std::iter::once(&mut g.inverses).chain(g.redo.iter_mut()));
        let steps = self
            .undo
            .iter_mut()
            .chain(self.redo.iter_mut())
            .chain(group);
        for op in steps.flatten() {
            if let Op::RestoreAsset { id, asset } = op {
                if let Some(source) = sources.get(id) {
                    asset.source = AssetSource::Path(source.clone());
                } else if let AssetSource::Path(p) = &asset.source {
                    let full = normalize(&old_dir.join(p));
                    match relative(&full, &self.dir) {
                        Some(rel) => asset.source = AssetSource::Path(rel),
                        // No path names the file from the new directory; its bytes do.
                        None => {
                            if let Ok(bytes) = files.read(&full.to_string_lossy()) {
                                asset.source = AssetSource::Embedded(bytes);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Close the open group, if any: its edits become one undo step, and an empty group none.
    fn end_group(&mut self) {
        if let Some(group) = self.group.take()
            && !group.inverses.is_empty()
        {
            self.undo.push(group.inverses.into_iter().rev().collect());
        }
    }

    /// An asset's bytes: embedded in the document, or read from its path, relative to the
    /// presentation's directory. It is public so that a host decoding assets, as the app does,
    /// reads them by this one rule and never joins a path to a directory itself.
    pub fn asset_bytes(&self, id: AssetId, files: &mut dyn Files) -> Result<Vec<u8>, String> {
        let asset = self.doc.assets.get(&id).ok_or(format!("no asset {id}"))?;
        match &asset.source {
            AssetSource::Embedded(bytes) => Ok(bytes.clone()),
            AssetSource::Path(path) => {
                let full = normalize(&self.dir.join(path));
                files.read(&full.to_string_lossy())
            }
        }
    }

    /// The ids an operation created, as its reply states them: a new slide and its root; or the
    /// new area of a split, and the new split when the node was wrapped; or a new asset.
    fn created(&self, before: &BTreeSet<Id>, inverse: &Op) -> String {
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
                Id::Slide(_) => {}
            }
        }
        if let Op::RemoveAsset { id } = inverse {
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
}

fn ids(doc: &Document) -> BTreeSet<Id> {
    fn walk(node: &Node, out: &mut BTreeSet<Id>) {
        match node {
            Node::Area(a) => {
                out.insert(Id::Area(a.id));
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
/// assets with no image, and the text areas with a character no font has.
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
    format!(
        "fallback={} missing={} glyphs={}",
        list(&report.font_fallbacks),
        list(&report.missing_images),
        list(&report.missing_glyphs)
    )
}

fn apply_style(style: &mut TextStyle, option: StyleOption) {
    match option {
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

fn parent(path: &str) -> PathBuf {
    Path::new(path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}
