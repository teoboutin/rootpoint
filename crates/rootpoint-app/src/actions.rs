//! The editor's actions, each the protocol commands that "UI actions to commands" of the slice's
//! spec maps it to, run through `App::run`. A test calls each on an app with no window, and runs the
//! same commands on a second session.

use std::path::{Component, Path, PathBuf};

use rootpoint_model::{AssetId, Axis, Content, ImageFit, Node, NodeId, SlideId, node_rects};
use rootpoint_protocol::{Command, Reply, StyleOption};

use crate::hit::{Edge, MIN_PART, Preview, Zone};
use crate::selection::Move;
use crate::{App, Gesture};

impl App {
    /// Select a node: a click on an area, or on a breadcrumb entry.
    pub fn select_node(&mut self, node: NodeId) {
        self.run(Command::Select { node });
    }

    /// Show a slide and select no node: a click on a slide in the list, or on the breadcrumb's
    /// first entry.
    pub fn show_slide(&mut self, slide: SlideId) {
        self.run(Command::SelectSlide { slide });
    }

    /// Alt and an arrow: select the parent, a child, or a sibling of the selection. A move with
    /// nowhere to go does nothing.
    pub fn move_selection(&mut self, m: Move) {
        let (Some(node), Some(slide)) = (self.session.view().selected, self.shown_slide()) else {
            return;
        };
        let target = self
            .session
            .document()
            .slide(slide)
            .and_then(|s| self.memory.target(s, node, m));
        if let Some(node) = target {
            self.select_node(node);
        }
    }

    /// Add a slide after the shown one, or at the end when none is shown.
    pub fn add_slide(&mut self) {
        let doc = self.session.document();
        let at = self
            .shown_slide()
            .and_then(|s| doc.slides.iter().position(|x| x.id == s))
            .map_or(doc.slides.len(), |i| i + 1);
        self.run(Command::AddSlide { at: Some(at) });
    }

    pub fn remove_slide(&mut self) {
        if let Some(slide) = self.shown_slide() {
            self.run(Command::RemoveSlide { slide });
        }
    }

    pub fn duplicate_slide(&mut self) {
        if let Some(slide) = self.shown_slide() {
            self.run(Command::DuplicateSlide { slide });
        }
    }

    /// A drop of a slide on the row at `to` in the slide list.
    pub fn move_slide(&mut self, slide: SlideId, to: usize) {
        self.run(Command::MoveSlide { slide, to });
    }

    /// Whether the selection splits at its middle along `axis`: refused where a part would be less
    /// than `MIN_PART` slide units, the bound a divider drag keeps.
    pub fn can_split_middle(&self, axis: Axis) -> bool {
        let (Some(node), Some(slide)) = (self.session.view().selected, self.shown_slide()) else {
            return false;
        };
        let doc = self.session.document();
        let Some(slide) = doc.slide(slide) else {
            return false;
        };
        node_rects(slide, doc.aspect)
            .iter()
            .find(|(id, _)| *id == node)
            .is_some_and(|(_, r)| {
                let extent = match axis {
                    Axis::LeftToRight => r.w,
                    Axis::TopToBottom => r.h,
                };
                extent * 0.5 >= MIN_PART
            })
    }

    /// A split button, or Shift and an arrow: split the selection at its middle, the new area on
    /// that edge.
    pub fn split_middle(&mut self, edge: Edge) {
        let (axis, side) = edge.split();
        if let Some(node) = self.session.view().selected
            && self.can_split_middle(axis)
        {
            self.run(Command::Split {
                node,
                axis,
                at: 0.5,
                side,
            });
        }
    }

    /// A click with Ctrl or Shift held: split the node at the preview line.
    pub fn split_at(&mut self, p: Preview) {
        self.run(Command::Split {
            node: p.node,
            axis: p.axis,
            at: p.at,
            side: p.side,
        });
    }

    /// Delete: remove the selection.
    pub fn remove_selection(&mut self) {
        if let Some(node) = self.session.view().selected {
            self.run(Command::Remove { node });
        }
    }

    /// Open an undo group for a gesture, after closing any group still open, so that within one
    /// frame a group closes before another opens. `false` when the session refuses it.
    fn open_group(&mut self) -> bool {
        self.release_drag();
        self.blur_text();
        matches!(self.run(Command::Begin), Reply::Ok(_))
    }

    /// The press that grabs a divider opens an undo group, so the whole drag is one undo step.
    pub fn grab_divider(&mut self, split: NodeId, index: usize) {
        let Some(pos) = self.divider(split, index) else {
            return;
        };
        if self.open_group() {
            self.gesture = Gesture::Divider { split, index, pos };
        }
    }

    /// A frame of a divider drag that moves it to `pos`.
    pub fn drag_divider(&mut self, pos: f64) {
        if let Gesture::Divider {
            split,
            index,
            pos: last,
        } = self.gesture
            && pos != last
        {
            self.run(Command::Divider { split, index, pos });
            self.gesture = Gesture::Divider { split, index, pos };
        }
    }

    /// The press on the selected area's grip opens an undo group; the drop sends the commands.
    pub fn grab_grip(&mut self, area: NodeId) {
        if self.open_group() {
            self.gesture = Gesture::Grip { area };
        }
    }

    /// The drop of a grip drag: on another area's swap zone, a `swap`; on a move zone, a split of
    /// the target on that edge at its middle, a swap of the dragged area with the new one and the
    /// removal of the dragged one; then the group ends. A drop on the dragged area itself, or
    /// outside any area, is `None` and cancels.
    pub fn drop_grip(&mut self, drop: Option<(NodeId, Zone)>) {
        let Gesture::Grip { area } = self.gesture else {
            return;
        };
        self.gesture = Gesture::Idle;
        let Some((target, zone)) = drop else {
            self.run(Command::Cancel);
            return;
        };
        match zone {
            Zone::Swap => {
                self.run(Command::Swap { a: area, b: target });
            }
            Zone::Move(edge) => {
                let (axis, side) = edge.split();
                let reply = self.run(Command::Split {
                    node: target,
                    axis,
                    at: 0.5,
                    side,
                });
                if let Some(new) = new_area(&reply) {
                    self.run(Command::Swap { a: area, b: new });
                    self.run(Command::Remove { node: area });
                }
            }
        }
        self.run(Command::End);
    }

    /// The release that ends a drag closes its group; a grip released here, with no drop
    /// computed, is cancelled.
    pub fn release_drag(&mut self) {
        match self.gesture {
            Gesture::Divider { .. } => {
                self.run(Command::End);
            }
            Gesture::Grip { .. } => {
                self.run(Command::Cancel);
            }
            Gesture::Idle | Gesture::Cancelled => {}
        }
        self.gesture = Gesture::Idle;
    }

    /// Esc during a drag: its edits are reverted, and the group closes with no step.
    pub fn cancel_drag(&mut self) {
        if let Gesture::Divider { .. } | Gesture::Grip { .. } = self.gesture {
            self.run(Command::Cancel);
            self.gesture = Gesture::Cancelled;
        }
    }

    /// A double-click on a divider sets it halfway between its neighbours, the neighbouring
    /// dividers or the split's edges.
    pub fn reset_divider(&mut self, split: NodeId, index: usize) {
        let Some(Node::Split(s)) = self.node(split) else {
            return;
        };
        let pos = (s.child_extent(index).0 + s.child_extent(index + 1).1) / 2.0;
        if s.dividers.get(index) != Some(&pos) {
            self.run(Command::Divider { split, index, pos });
        }
    }

    /// The selected area, when the selection is an area.
    pub fn selected_area(&self) -> Option<NodeId> {
        let node = self.session.view().selected?;
        matches!(self.node(node)?, Node::Area(_)).then_some(node)
    }

    /// The content of the selected area.
    pub fn selected_content(&self) -> Option<&Content> {
        match self.node(self.selected_area()?)? {
            Node::Area(a) => Some(&a.content),
            Node::Split(_) => None,
        }
    }

    /// Set the selected area's content to empty.
    pub fn clear_content(&mut self) {
        if let Some(area) = self.selected_area() {
            self.run(Command::Clear { area });
        }
    }

    /// Set the selected area's content to an empty text, in the default style.
    pub fn set_text_content(&mut self) {
        if let Some(area) = self.selected_area() {
            self.run(Command::Text {
                area,
                text: String::new(),
                style: Vec::new(),
            });
        }
    }

    /// The text field gains focus: its edits until it loses focus are one undo step.
    pub fn focus_text(&mut self) {
        let Some(area) = self.selected_area() else {
            return;
        };
        let text = match self.selected_content() {
            Some(Content::Text(t)) => t.text.clone(),
            _ => return,
        };
        if self.open_group() {
            self.editing = Some((area, text));
        }
    }

    /// A change of the text field: the area's whole text.
    pub fn type_text(&mut self, text: String) {
        if let Some((area, _)) = self.editing {
            self.run(Command::Text {
                area,
                text: text.clone(),
                style: Vec::new(),
            });
            self.editing = Some((area, text));
        }
    }

    /// The text field loses focus, or the window does: the group closes.
    pub fn blur_text(&mut self) {
        if self.editing.take().is_some() {
            self.run(Command::End);
        }
    }

    /// Set one option of the selected text's style.
    pub fn set_style(&mut self, option: StyleOption) {
        if let (Some(area), Some(Content::Text(t))) =
            (self.selected_area(), self.selected_content())
        {
            let text = t.text.clone();
            self.run(Command::Text {
                area,
                text,
                style: vec![option],
            });
        }
    }

    /// Set the selected area's content to an image asset, fitted or filling the area.
    pub fn set_image(&mut self, asset: AssetId, fit: ImageFit) {
        if let Some(area) = self.selected_area() {
            self.run(Command::Image { area, asset, fit });
        }
    }

    /// Add an image asset, by its path relative to the session's directory.
    pub fn add_asset(&mut self, path: String) {
        self.run(Command::Asset { path });
    }

    pub fn undo(&mut self) {
        self.blur_text();
        self.run(Command::Undo);
    }

    pub fn redo(&mut self) {
        self.blur_text();
        self.run(Command::Redo);
    }

    /// Load a presentation file; Save then writes to it.
    pub fn open_file(&mut self, path: String) {
        self.blur_text();
        if let Reply::Ok(_) = self.run(Command::Load { path: path.clone() }) {
            self.path = Some(path);
        }
    }

    /// Save the presentation to a file; Save then writes to it again.
    pub fn save_to(&mut self, path: String) {
        self.blur_text();
        if let Reply::Ok(_) = self.run(Command::Save { path: path.clone() }) {
            self.path = Some(path);
        }
    }

    /// Open the built-in sample, which has no file of its own.
    pub fn open_sample(&mut self) {
        self.blur_text();
        if let Reply::Ok(_) = self.run(Command::Sample) {
            self.path = None;
        }
    }

    /// F5: present the shown slide, full screen.
    pub fn present(&mut self) {
        self.blur_text();
        self.release_drag();
        self.run(Command::Present {
            slide: self.shown_slide(),
        });
    }

    /// →, Space or Page Down in presenter mode. At the last slide it stays there.
    pub fn next_slide(&mut self) {
        self.run(Command::Next);
    }

    /// ← or Page Up in presenter mode. At the first slide it stays there.
    pub fn previous_slide(&mut self) {
        self.run(Command::Prev);
    }

    /// Esc in presenter mode: back to the editor.
    pub fn stop_presenting(&mut self) {
        self.run(Command::Stop);
    }

    /// Ctrl+O: ask for a presentation file, and load it.
    pub(crate) fn open_dialog(&mut self) {
        let picked = rfd::FileDialog::new()
            .add_filter("rootpoint presentation", &["ron"])
            .pick_file();
        if let Some(path) = picked.and_then(|p| self.utf8(&p)) {
            self.open_file(path);
        }
    }

    /// Ctrl+S: save to the presentation's file, or ask for one when it has none.
    pub(crate) fn save(&mut self) {
        match self.path.clone() {
            Some(path) => self.save_to(path),
            None => self.save_dialog(),
        }
    }

    /// Ask for the file to save to.
    pub(crate) fn save_dialog(&mut self) {
        let picked = rfd::FileDialog::new()
            .add_filter("rootpoint presentation", &["ron"])
            .set_file_name("presentation.ron")
            .save_file();
        if let Some(path) = picked.and_then(|p| self.utf8(&p)) {
            self.save_to(path);
        }
    }

    /// Ask for an image file, and add it as an asset, its path relative to the session's
    /// directory, as every asset's path is.
    pub(crate) fn add_asset_dialog(&mut self) {
        let Some(file) = rfd::FileDialog::new()
            .add_filter("image", &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return;
        };
        let directory = match std::env::current_dir() {
            Ok(cwd) => cwd.join(self.session.directory()),
            Err(e) => {
                self.error = Some(format!("the working directory: {e}"));
                return;
            }
        };
        match relative(&file, &directory).and_then(|p| self.utf8(&p)) {
            Some(path) => self.add_asset(path),
            None if self.error.is_none() => {
                self.error = Some(format!(
                    "{} has no path from the presentation",
                    file.display()
                ));
            }
            None => {}
        }
    }

    /// A path as UTF-8, which the protocol's commands hold, or `None` with the status naming it.
    fn utf8(&mut self, path: &Path) -> Option<String> {
        let text = path.to_str().map(str::to_owned);
        if text.is_none() {
            self.error = Some(format!("{} is not UTF-8", path.display()));
        }
        text
    }

    /// A node of the shown slide.
    pub(crate) fn node(&self, node: NodeId) -> Option<&Node> {
        let slide = self.session.document().slide(self.shown_slide()?)?;
        slide.root.find(node)
    }

    /// The position of a divider of the shown slide.
    fn divider(&self, split: NodeId, index: usize) -> Option<f64> {
        match self.node(split)? {
            Node::Split(s) => s.dividers.get(index).copied(),
            Node::Area(_) => None,
        }
    }
}

/// The new area a `split` reply names, as `new=n12`.
fn new_area(reply: &Reply) -> Option<NodeId> {
    let Reply::Ok(text) = reply else {
        return None;
    };
    text.split(' ')
        .find_map(|w| w.strip_prefix("new=n"))
        .and_then(|n| n.parse().ok())
        .map(NodeId)
}

/// The path of the absolute `file` from the absolute directory `dir`, with `..` where it lies
/// outside; `None` where the two share no root, as on two drives.
fn relative(file: &Path, dir: &Path) -> Option<PathBuf> {
    let file: Vec<Component> = file.components().collect();
    let dir: Vec<Component> = dir
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect();
    if file.first() != dir.first() {
        return None;
    }
    let common = file.iter().zip(&dir).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in &dir[common..] {
        out.push("..");
    }
    for c in &file[common..] {
        out.push(c.as_os_str());
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_made_relative_to_the_presentations_directory() {
        let rel = |f: &str, d: &str| relative(Path::new(f), Path::new(d)).unwrap();
        assert_eq!(rel("/a/b/cat.png", "/a/b"), Path::new("cat.png"));
        assert_eq!(rel("/a/b/img/cat.png", "/a/b"), Path::new("img/cat.png"));
        assert_eq!(rel("/a/cat.png", "/a/b/c"), Path::new("../../cat.png"));
        assert_eq!(rel("/a/b/cat.png", "/a/b/."), Path::new("cat.png"));
    }

    #[test]
    fn the_new_area_of_a_split_is_read_from_its_reply() {
        let reply = Reply::Ok("new=n12 split=n13".to_owned());
        assert_eq!(new_area(&reply), Some(NodeId(12)));
        assert_eq!(new_area(&Reply::Ok("split=n13".to_owned())), None);
    }
}
