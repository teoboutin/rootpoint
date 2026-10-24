//! A session: one document, its undo history and a view state, changed by commands.

use std::fmt;

use std::collections::{BTreeMap, BTreeSet};

use rootpoint_model::{
    Area, Asset, AssetId, Content, Document, FontRef, ImageContent, Node, NodeId, Op, Replacement,
    SlideId, StyleChoice, TextContent, TextStyle, apply, asset_path_is_confined, load_ron,
    save_ron,
};
use rootpoint_render::{FontSet, ImageSet, Precision, Report, render_slide};

use crate::command::{Command, StyleCommand, StyleOption, parse_command, quote, quote_always};
use crate::files::Files;
use crate::store::{PRESENTATION, join, readable, unconfined, with_suffix, writable};
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

/// One undo step: the operations that revert one command, or every command of one group, applied
/// in order.
type Step = Vec<Op>;

/// A group opened by `begin`: the inverses of its edits, in the order they were applied, and the
/// redo stack as it was at the `begin`, which `cancel` restores.
struct Group {
    inverses: Vec<Op>,
    redo: Vec<Step>,
}

/// Where the session finds the files of its presentation. Every asset path is relative to the
/// store's root, which never changes while the store is open, so no command rewrites a path.
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
    /// The undo steps pushed, undone or redone since the store was opened or last saved.
    unsaved: u64,
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
            store: Store::Memory(BTreeMap::new()),
            unsaved: 0,
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
            Command::Save => {
                let Store::Workspace(root) = &self.store else {
                    return Err(
                        "a presentation held in memory is saved by `pack <file>` or \
                        `unpack <dir>`"
                            .to_owned(),
                    );
                };
                files.write(&join(root, PRESENTATION), save_ron(&self.doc).as_bytes())?;
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
                let path = self.import(&from, to, files)?;
                match self.doc.assets.iter().find(|(_, a)| a.path == path) {
                    Some((id, _)) => Ok(id.to_string()),
                    None => self.edit(Op::AddAsset {
                        asset: Asset { path },
                    }),
                }
            }
            Command::Store => {
                let unsaved = self.unsaved;
                Ok(match &self.store {
                    Store::Memory(_) => format!("memory unsaved={unsaved}"),
                    Store::Workspace(root) => {
                        format!("workspace {} unsaved={unsaved}", quote(root))
                    }
                })
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
                            if let FontRef::Asset(id) = self.doc.resolve_style(&t.style).font
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
            editing => {
                let op = self.operation(editing)?;
                self.edit(op)
            }
        }
    }

    /// Apply an editing operation: its inverse joins the open group, or is an undo step of its
    /// own. The reply names the ids it created.
    fn edit(&mut self, op: Op) -> Result<String, String> {
        let before = ids(&self.doc);
        let inverse = self.apply_op(op)?;
        match &mut self.group {
            Some(group) => group.inverses.push(inverse.clone()),
            None => {
                self.undo.push(vec![inverse.clone()]);
                self.unsaved += 1;
            }
        }
        self.redo.clear();
        Ok(self.created(&before, &inverse))
    }

    /// The asset path an import of the host's file `from` adds, after writing the file into the
    /// store when it is not there yet. In a workspace, a file below the root is taken at its path,
    /// with no copy, and `to`, when given, must be that path. Any other file is written at `to`, or
    /// at the root under its file name, with `-2`, `-3` before the extension where a file of other
    /// bytes holds the name, and the name itself where its file holds the same bytes.
    fn import(
        &mut self,
        from: &str,
        to: Option<String>,
        files: &mut dyn Files,
    ) -> Result<String, String> {
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
                return Ok(below.to_owned());
            }
        }
        let name = match to {
            Some(to) => to,
            None => from.rsplit('/').next().unwrap_or(from).to_owned(),
        };
        if !asset_path_is_confined(&name) {
            return Err(unconfined(&name));
        }
        for n in 1.. {
            let candidate = if n == 1 {
                name.clone()
            } else {
                with_suffix(&name, n)
            };
            let held = match &self.store {
                Store::Memory(map) => map.get(&candidate).cloned(),
                Store::Workspace(root) => {
                    let full = writable(root, &candidate, files)?;
                    if files.exists(&full)? {
                        Some(files.read(&full)?)
                    } else {
                        None
                    }
                }
            };
            // A name no file holds is free, even where an asset names it: the file written there
            // is that asset's, which was drawn missing until now.
            match held {
                Some(held) if held == bytes => return Ok(candidate),
                Some(_) => continue,
                None => {}
            }
            match &mut self.store {
                Store::Memory(map) => {
                    map.insert(candidate.clone(), bytes);
                }
                Store::Workspace(root) => {
                    let full = writable(root, &candidate, files)?;
                    files.write(&full, &bytes)?;
                }
            }
            return Ok(candidate);
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
            Command::Text { area, text, style } => {
                // The grammar refuses these sizes; a command built through the API is held to the
                // same rule, so that it prints back as a line that parses.
                if style
                    .iter()
                    .any(|o| matches!(o, StyleOption::Size(s) if !(s.is_finite() && *s > 0.0)))
                {
                    return Err("a text size is a number above 0".to_owned());
                }
                let mut choice = self.text_style(area);
                for option in style {
                    choice = match option {
                        StyleOption::Named(name) => {
                            self.usable_style(&name)?;
                            StyleChoice::Named(name)
                        }
                        StyleOption::Custom => StyleChoice::Custom(self.doc.resolve_style(&choice)),
                        // Any other option changes the values drawn, so a named style forks to a
                        // custom copy: no command but `style set` changes a shared style,
                        // `design@rootpoint-model@named-styles`.
                        value => {
                            self.usable_option(&value)?;
                            let mut values = self.doc.resolve_style(&choice);
                            apply_style(&mut values, value);
                            StyleChoice::Custom(values)
                        }
                    };
                }
                Op::SetContent {
                    area,
                    content: Content::Text(TextContent {
                        text,
                        style: choice,
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
            Command::RemoveAsset { asset } => Op::RemoveAsset { id: asset },
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

    /// Close the open group, if any: its edits become one undo step, and an empty group none.
    fn end_group(&mut self) {
        if let Some(group) = self.group.take()
            && !group.inverses.is_empty()
        {
            self.undo.push(group.inverses.into_iter().rev().collect());
            self.unsaved += 1;
        }
    }

    /// An asset's bytes: its entry of the map in memory, or its file below a workspace's root.
    /// A path that leads out of the root is refused, whatever the file names, so no read reaches
    /// outside it (`design@rootpoint-protocol@store-confines-access`). It is public so that a host decoding
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
    let styles: Vec<String> = report
        .missing_styles
        .iter()
        .map(|(area, name)| format!("{area}:{}", quote(name)))
        .collect();
    format!(
        "fallback={} missing={} glyphs={} styles={}",
        list(&report.font_fallbacks),
        list(&report.missing_images),
        list(&report.missing_glyphs),
        list(&styles)
    )
}

/// The refusal of a style the table lacks.
fn no_style(name: &str) -> String {
    format!("no style {}", quote_always(name))
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
