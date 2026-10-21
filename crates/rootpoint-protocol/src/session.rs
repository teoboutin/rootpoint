//! A session: one document, its undo history and a view state, changed by commands.

use std::fmt;
use std::path::{Path, PathBuf};

use std::collections::BTreeSet;

use rootpoint_model::{
    Asset, AssetSource, Content, Document, FontRef, ImageContent, Node, NodeId, Op, SlideId,
    TextContent, TextStyle, apply, load_ron, save_ron,
};

use crate::command::{Command, StyleOption, parse_command};
use crate::files::Files;
use crate::tree::slide_tree;

/// The reply to one command: `ok` with its result, or `err` with the reason.
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
            write!(f, "{word} {text}")
        }
    }
}

/// What the session shows besides the document: the selected node, and the slide presenter mode
/// shows, if it is on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct View {
    pub selected: Option<NodeId>,
    pub presenting: Option<SlideId>,
}

/// One document, its undo and redo stacks, and its view, changed only by commands.
pub struct Session {
    doc: Document,
    /// The directory of the presentation file, against which asset paths are read.
    dir: PathBuf,
    undo: Vec<Op>,
    redo: Vec<Op>,
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
            view: View::default(),
        }
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    /// Run one line: a command's reply, or `None` for a blank line or a comment, which starts
    /// with `#`.
    pub fn execute(&mut self, line: &str, files: &mut dyn Files) -> Option<Reply> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        Some(match parse_command(line) {
            Ok(command) => self.run(command, files),
            Err(e) => Reply::Err(e.0),
        })
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
                files.write(&path, save_ron(&self.doc).as_bytes())?;
                self.dir = parent(&path);
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
            Command::Undo => {
                let inverse = self.undo.pop().ok_or("nothing to undo")?;
                let redo = self
                    .apply_op(inverse.clone())
                    .inspect_err(|_| self.undo.push(inverse))?;
                self.redo.push(redo);
                Ok(String::new())
            }
            Command::Redo => {
                let op = self.redo.pop().ok_or("nothing to redo")?;
                let undo = self
                    .apply_op(op.clone())
                    .inspect_err(|_| self.redo.push(op))?;
                self.undo.push(undo);
                Ok(String::new())
            }
            editing => {
                let op = self.operation(editing)?;
                let before = ids(&self.doc);
                let inverse = self.apply_op(op)?;
                self.undo.push(inverse.clone());
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
                let mut current = match self.doc.node(area) {
                    Some(Node::Area(a)) => match &a.content {
                        Content::Text(t) => t.style.clone(),
                        _ => TextStyle::default(),
                    },
                    _ => TextStyle::default(),
                };
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
            Command::Image { area, asset, fit } => Op::SetContent {
                area,
                content: Content::Image(ImageContent { asset, fit }),
            },
            Command::Clear { area } => Op::SetContent {
                area,
                content: Content::Empty,
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

    fn apply_op(&mut self, op: Op) -> Result<Op, String> {
        apply(&mut self.doc, op).map_err(|e| format!("{e:?}"))
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
