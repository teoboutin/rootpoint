//! The editor's actions, each the protocol commands that do what it does, run through `App::run`,
//! per `design@rootpoint-app@actions-are-commands`. A test calls each on an app with no window,
//! and runs the same commands on a second session.

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use eframe::egui::Pos2;
use parley::{Affinity, Selection};
use rootpoint_model::{AssetId, Axis, Content, ImageFit, Node, NodeId, SlideId, node_rects};
use rootpoint_model::{CaretPos, SpanId, TextNodeRef};
use rootpoint_model::{Content as C, Replacement, StyleChoice, TextContent};
use rootpoint_protocol::{Command, Reply, StyleCommand, StyleOption, TextTarget};

use crate::editing::{
    CaretMove, Editing, Item, empty_spans, from_byte, items, items_len, layout_point, left, parent,
    range_text, right, text_node,
};

use crate::hit::{Edge, Layout, MIN_PART, Preview, Zone};
use crate::menu::NodeMenu;
use crate::selection::Move;
use crate::{App, Controller, Gesture, MenuEntry, Removal};

impl App {
    /// Select a node: a click on an area, or on its row of the outline.
    pub fn select_node(&mut self, node: NodeId) {
        self.run(Command::Select { node });
    }

    /// Show a slide and select no node: a click on a slide in the list, or on the outline's row of
    /// the slide.
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
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        let doc = self.session.document();
        let at = self
            .shown_slide()
            .and_then(|s| doc.slides.iter().position(|x| x.id == s))
            .map_or(doc.slides.len(), |i| i + 1);
        self.run(Command::AddSlide { at: Some(at) });
    }

    /// Remove the shown slide.
    pub fn remove_slide(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let Some(slide) = self.shown_slide() {
            self.run(Command::RemoveSlide { slide });
        }
    }

    /// Insert a copy of the shown slide after it.
    pub fn duplicate_slide(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let Some(slide) = self.shown_slide() {
            self.run(Command::DuplicateSlide { slide });
        }
    }

    /// A drop of a slide on the row at `to` in the slide list.
    pub fn move_slide(&mut self, slide: SlideId, to: usize) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::MoveSlide { slide, to });
    }

    /// Whether the selection splits at its middle along `axis`: refused where a part would be less
    /// than `MIN_PART` slide units, the bound a divider drag keeps.
    pub fn can_split_middle(&self, axis: Axis) -> bool {
        if self.socket_holds() {
            return false;
        }
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
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
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
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Split {
            node: p.node,
            axis: p.axis,
            at: p.at,
            side: p.side,
        });
    }

    /// A secondary click on `node` at `at`: leave editing, select the node, so the menu and the
    /// side panel describe one node, and open the menu on it, `design@rootpoint-app@node-menu`.
    pub fn open_node_menu(&mut self, node: NodeId, at: Pos2) {
        // A secondary click inside the edited area ends editing too, before the menu opens, so
        // the keys, which go to the text while editing, act on the node the menu names,
        // `design@rootpoint-app@edit-entry`.
        if self.edit.is_some() {
            self.stop_editing();
        }
        self.select_node(node);
        if self.session.view().selected == Some(node) {
            self.menu_serial += 1;
            self.menu = Some(NodeMenu {
                node,
                at,
                serial: self.menu_serial,
            });
        }
    }

    /// A click on an entry of the open node menu: it runs the action of the side panel's
    /// equivalent and closes the menu. An entry of "Content" or "Style" that names what the node
    /// holds now sends nothing, as the side panel's does.
    pub fn menu_entry(&mut self, entry: MenuEntry) {
        if self.menu.take().is_none() {
            return;
        }
        let content = self.selected_content().cloned();
        match entry {
            MenuEntry::Select(node) => self.select_node(node),
            MenuEntry::Text => {
                if !matches!(content, Some(Content::Text(_))) {
                    self.set_text_content();
                }
            }
            MenuEntry::Image(asset) => match content {
                Some(Content::Image(i)) if i.asset != asset => self.set_image(asset, i.fit),
                Some(Content::Image(_)) => {}
                _ => self.set_image(asset, ImageFit::Fit),
            },
            MenuEntry::Style(choice) => {
                let style = match content {
                    Some(Content::Text(t)) => t.style,
                    _ => return,
                };
                match choice {
                    Some(name) if style != StyleChoice::Named(name.clone()) => {
                        self.name_style(name)
                    }
                    None if !matches!(style, StyleChoice::Custom(_)) => self.make_custom(),
                    _ => {}
                }
            }
            MenuEntry::Remove => self.remove_selection(),
        }
    }

    /// The button "Preview": enter the final view, which deselects, or leave it. It changes only
    /// the view, so it runs while the socket holds control; it ends the window's gestures first, as
    /// any action that does not join them.
    pub fn toggle_final_view(&mut self) {
        self.end_gestures();
        let on = !self.final_view();
        self.run(Command::Preview { on });
    }

    /// Delete: remove the selection.
    pub fn remove_selection(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let Some(node) = self.session.view().selected {
            self.run(Command::Remove { node });
        }
    }

    /// Open an undo group for a gesture, after closing any group still open, so that within one
    /// frame a group closes before another opens. `false` when the session refuses it.
    pub(crate) fn open_group(&mut self) -> bool {
        if self.socket_holds() {
            return false;
        }
        self.end_gestures();
        matches!(self.run(Command::Begin), Reply::Ok(_))
    }

    /// The button "Take control": the window takes the authority to edit back from the socket,
    /// closing with `end` a group the socket left open, so its edits are one undo step.
    pub fn take_control(&mut self) {
        if self.controller != Controller::Socket {
            return;
        }
        if self.session.group_open() {
            self.run(Command::End);
        }
        self.controller = Controller::TakenFromSocket;
    }

    /// Before an edit that is no part of a gesture: end the window's own gestures, a drag and a
    /// text edited in place, closing their group, so the edit is a step of its own. A group the
    /// control socket opened is closed by the socket or by the take-over, never here. A button of
    /// the slide list or of the side panel can be pressed while a text is edited.
    pub(crate) fn end_gestures(&mut self) {
        self.release_drag();
        self.stop_editing();
    }

    /// The press that grabs a divider opens an undo group, so the whole drag is one undo step.
    pub fn grab_divider(&mut self, split: NodeId, index: usize) {
        if self.socket_holds() {
            return;
        }
        let Some(pos) = self.divider(split, index) else {
            return;
        };
        if self.open_group() {
            self.gesture = Gesture::Divider {
                split,
                index,
                pos,
                left: false,
            };
        }
    }

    /// A frame of a divider drag that moves it to `pos`.
    pub fn drag_divider(&mut self, pos: f64) {
        if self.socket_holds() {
            return;
        }
        if let Gesture::Divider {
            split,
            index,
            pos: last,
            ..
        } = self.gesture
            && pos != last
        {
            self.run(Command::Divider { split, index, pos });
            self.gesture = Gesture::Divider {
                split,
                index,
                pos,
                left: true,
            };
        }
    }

    /// The press on the selected area's grip opens an undo group; the drop sends the commands.
    pub fn grab_grip(&mut self, area: NodeId) {
        if self.socket_holds() {
            return;
        }
        if self.open_group() {
            self.gesture = Gesture::Grip { area };
        }
    }

    /// The drop of a grip drag: on another area's swap zone, a `swap`; on a move zone, a split of
    /// the target on that edge at its middle, a swap of the dragged area with the new one and the
    /// removal of the dragged one; then the group ends. A drop on the dragged area itself, or
    /// outside any area, is `None` and cancels.
    pub fn drop_grip(&mut self, drop: Option<(NodeId, Zone)>) {
        if self.socket_holds() {
            return;
        }
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
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
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

    /// Set the selected area's content to an empty text, in the style a new area takes.
    pub fn set_text_content(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let Some(area) = self.selected_area() {
            self.run(Command::Text {
                area,
                root: Some(Vec::new()),
                style: Vec::new(),
            });
        }
    }

    /// Set one option of the selected text's style.
    pub fn set_style(&mut self, option: StyleOption) {
        self.send_text_option(option);
    }

    /// Make the selected text name the style `name`.
    pub fn name_style(&mut self, name: String) {
        self.send_text_option(StyleOption::Named(name));
    }

    /// Make the selected text hold the values it draws with as its own.
    pub fn make_custom(&mut self) {
        self.send_text_option(StyleOption::Custom);
    }

    /// Change the selected text's style by one option, its tree and its spans' ids kept, by `text`
    /// with no tree, `design@rootpoint-protocol@text-style-alone`.
    fn send_text_option(&mut self, option: StyleOption) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let (Some(area), Some(C::Text(_))) = (self.selected_area(), self.selected_content()) {
            self.run(Command::Text {
                area,
                root: None,
                style: vec![option],
            });
        }
    }

    /// The first name of the series "Style 1", "Style 2", … that the table does not hold.
    fn free_style_name(&self) -> String {
        let styles = &self.session.document().styles;
        (1..)
            .map(|n| format!("Style {n}"))
            .find(|name| !styles.contains_key(name))
            .expect("a table holds finitely many styles")
    }

    /// Add a style in the default values, under a free name, and select it in the Styles tab.
    pub fn add_style(&mut self) {
        self.add_style_with(Vec::new());
    }

    /// Add a copy of the style selected in the Styles tab, under a free name, and select it.
    pub fn duplicate_style(&mut self) {
        let Some(values) = self
            .styles_tab
            .selected
            .as_ref()
            .and_then(|name| self.session.document().styles.get(name))
        else {
            return;
        };
        let options = StyleOption::values_of(values);
        self.add_style_with(options);
    }

    fn add_style_with(&mut self, options: Vec<StyleOption>) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        let name = self.free_style_name();
        if let Reply::Ok(_) = self.run(Command::Style(StyleCommand::Add {
            name: name.clone(),
            options,
        })) {
            self.select_style(&name);
        }
    }

    /// Rename the style selected in the Styles tab, and keep it selected.
    pub fn rename_style(&mut self, to: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        let Some(from) = self.styles_tab.selected.clone() else {
            return;
        };
        if to.is_empty() || to == from {
            return;
        }
        if let Reply::Ok(_) = self.run(Command::Style(StyleCommand::Rename {
            from,
            to: to.clone(),
        })) {
            self.select_style(&to);
        }
    }

    /// Change one value of the style selected in the Styles tab, which every text naming it then
    /// draws with.
    pub fn set_selected_style(&mut self, option: StyleOption) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let Some(name) = self.styles_tab.selected.clone() {
            self.run(Command::Style(StyleCommand::Set {
                name,
                options: vec![option],
            }));
        }
    }

    /// Remove the style selected in the Styles tab: at once when no text names it, and otherwise
    /// by the dialog that asks what its texts take instead, a custom copy first.
    pub fn remove_style(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        let Some(name) = self.styles_tab.selected.clone() else {
            return;
        };
        if self.style_users(&name) == 0 {
            if let Reply::Ok(_) =
                self.run(Command::Style(StyleCommand::Remove { name, into: None }))
            {
                self.styles_tab.selected = None;
            }
        } else {
            self.styles_tab.removal = Some(Removal { name, into: None });
        }
    }

    /// Choose, in the dialog of a removal, what the texts naming the style take: another style, or
    /// `None` for a custom copy.
    pub fn choose_replacement(&mut self, into: Option<String>) {
        if let Some(removal) = &mut self.styles_tab.removal {
            removal.into = into;
        }
    }

    /// Remove the style of the open dialog, with the replacement chosen.
    pub fn confirm_removal(&mut self) {
        if self.socket_holds() {
            return;
        }
        let Some(removal) = self.styles_tab.removal.take() else {
            return;
        };
        self.end_gestures();
        let into = Some(match removal.into {
            Some(other) => Replacement::Named(other),
            None => Replacement::Custom,
        });
        if let Reply::Ok(_) = self.run(Command::Style(StyleCommand::Remove {
            name: removal.name,
            into,
        })) {
            self.styles_tab.selected = None;
        }
    }

    /// The styles the texts naming `name` may take instead, when it is removed: each other style a
    /// text may name, one whose font draws with no stand-in, since the command refuses any other,
    /// `design@rootpoint-protocol@no-new-breach`.
    pub fn replacements(&self, name: &str) -> Vec<String> {
        let doc = self.session.document();
        doc.styles
            .iter()
            .filter(|(other, style)| *other != name && doc.draws_as_asked(style))
            .map(|(other, _)| other.clone())
            .collect()
    }

    /// Close the dialog of a removal, removing nothing.
    pub fn cancel_removal(&mut self) {
        self.styles_tab.removal = None;
    }

    /// How many texts, over every slide, name the style `name`.
    pub(crate) fn style_users(&self, name: &str) -> usize {
        self.session
            .document()
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .filter(|a| {
                matches!(&a.content, C::Text(TextContent { style: StyleChoice::Named(n), .. }) if n == name)
            })
            .count()
    }

    /// Set the selected area's content to an image asset, fitted or filling the area.
    pub fn set_image(&mut self, asset: AssetId, fit: ImageFit) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        if let Some(area) = self.selected_area() {
            self.run(Command::Image { area, asset, fit });
        }
    }

    /// Add a file of the host's files as an asset, by `import`: at its path below a workspace's
    /// root, or copied into the store.
    pub fn add_asset(&mut self, path: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Import {
            from: path,
            to: None,
        });
    }

    /// The Assets tab's move: give the asset a new path, its file moved there, by `asset-move`.
    pub fn move_asset(&mut self, asset: AssetId, to: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::AssetMove { asset, to });
    }

    /// A drop of an asset's row on a folder's row: move it into that folder, `""` the root, under
    /// its file name.
    pub fn move_asset_to_folder(&mut self, asset: AssetId, folder: &str) {
        let path = self.asset_name(asset);
        let name = path.rsplit('/').next().unwrap_or(&path);
        let to = if folder.is_empty() {
            name.to_owned()
        } else {
            format!("{folder}/{name}")
        };
        if to != path {
            self.move_asset(asset, to);
        }
    }

    /// Relink a missing asset to a file the store holds, by `asset-relink`.
    pub fn relink_asset(&mut self, asset: AssetId, to: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::AssetRelink { asset, to });
    }

    /// Give an asset's file the bytes of a host file, by `asset-replace`.
    pub fn replace_asset(&mut self, asset: AssetId, from: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::AssetReplace { asset, from });
    }

    /// Replace an asset's bytes by a file the author picked, held in memory under its name, as
    /// `App::add_picked_image` holds a picked image; a refused replace gives the held files back.
    pub fn replace_with_picked(&mut self, asset: AssetId, name: String, bytes: Vec<u8>) {
        if self.socket_holds() {
            return;
        }
        let Some(held) = self.held.clone() else {
            self.error = Some("this editor holds no files in memory".to_owned());
            return;
        };
        let before = held.replace_one(&name, Some(bytes));
        self.replace_asset(asset, name.clone());
        if self.error.is_some() {
            held.replace_one(&name, before);
        }
    }

    /// Remove an asset nothing names, by `asset-remove`; its file goes to the trash.
    pub fn remove_asset(&mut self, asset: AssetId) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::AssetRemove { asset, into: None });
    }

    /// Remove an asset, what names it naming `into` instead, by `asset-remove … into=`.
    pub fn remove_asset_into(&mut self, asset: AssetId, into: AssetId) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::AssetRemove {
            asset,
            into: Some(into),
        });
    }

    /// Remove every asset nothing names, one undo step, by `asset-remove-unused`.
    pub fn remove_unused_assets(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::AssetRemoveUnused);
    }

    /// A use's button of the Assets tab: select its node, which shows its slide.
    pub fn show_use(&mut self, node: NodeId) {
        self.end_gestures();
        self.run(Command::Select { node });
    }

    /// The files dropped on the window this frame, each imported: natively by its path; in the
    /// browser its bytes are read first, then imported as a picked image, since a browser gives a
    /// dropped file's name for its path and its bytes only asynchronously, read in eframe 0.37.0's
    /// `path@elsewhere@eframe-0.37.0/src/web/dropped_file.rs`.
    pub(crate) fn take_dropped_files(&mut self, ctx: &eframe::egui::Context) {
        let dropped = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
        if dropped.is_empty() || self.socket_holds() {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        for file in dropped {
            if let Some(path) = self.utf8(file.path()) {
                self.add_asset(path);
            }
        }
        #[cfg(target_arch = "wasm32")]
        self.web.read_dropped(dropped);
    }

    /// Undo the last step: a drag or a stay in editing still open ends first, and is that
    /// step.
    pub fn undo(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Undo);
    }

    /// Redo the last step undone, after closing any group still open.
    pub fn redo(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Redo);
    }

    /// Open a directory as a workspace: its `presentation.ron`, which Save then writes, or an
    /// empty presentation.
    pub fn open_workspace(&mut self, dir: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Open { dir });
    }

    /// Open a pack in memory, its file then the one Save writes.
    pub fn open_pack(&mut self, file: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::OpenPack { file });
    }

    /// Write the presentation as a pack: in memory, Save then writes it again; a workspace stays
    /// as it is.
    pub fn pack_to(&mut self, file: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Pack { file });
    }

    /// Write a presentation held in memory as a workspace at an empty directory, which Save then
    /// writes.
    pub fn unpack_to(&mut self, dir: String) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Unpack { dir });
    }

    /// Write the presentation held in memory as a pack into the files held in memory, and give its
    /// name and its bytes, which the browser build hands to the page as a download: `save` when a
    /// pack is the session's target and `new_pack` is false, `pack` to that pack's name, or to
    /// `presentation.rpk`, otherwise. None while the control socket holds the authority to edit,
    /// and, with the status saying why, when the command fails or the app holds no files in
    /// memory. Built on every target, so the native tests check it.
    pub fn pack_for_download(&mut self, new_pack: bool) -> Option<(String, Vec<u8>)> {
        if self.socket_holds() {
            return None;
        }
        let Some(mut held) = self.held.clone() else {
            self.error = Some("this editor holds no files in memory".to_owned());
            return None;
        };
        self.end_gestures();
        let name = self.session.pack().unwrap_or("presentation.rpk").to_owned();
        let command = if new_pack || self.session.pack().is_none() {
            Command::Pack { file: name.clone() }
        } else {
            Command::Save
        };
        match self.run(command) {
            Reply::Ok(_) => rootpoint_protocol::Files::read(&mut held, &name)
                .ok()
                .map(|bytes| (name, bytes)),
            Reply::Err(_) => None,
        }
    }

    /// Open a pack the author picked, by its file name and its bytes: they are held in memory
    /// under the name, and `open-pack` opens them. A refused pack leaves the held files as they
    /// were. Only an app made by `App::in_memory` holds them; the browser build is one.
    pub fn open_picked_pack(&mut self, name: String, bytes: Vec<u8>) {
        if self.socket_holds() {
            return;
        }
        let Some(held) = self.held.clone() else {
            self.error = Some("this editor holds no files in memory".to_owned());
            return;
        };
        let before = held.replace_one(&name, Some(bytes));
        self.open_pack(name.clone());
        if self.error.is_some() {
            held.replace_one(&name, before);
        }
    }

    /// The banner's "Load the file": `reload` makes the file changed outside rootpoint the
    /// document, one undo step, and the autosave resumes from it; a file that does not load is
    /// named in the banner, which stays.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_outside_file(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        // The autosave resumes from the file read, as after every `reload` of the window.
        self.reload_error = match self.run(Command::Reload) {
            Reply::Ok(_) => None,
            Reply::Err(reason) => Some(reason),
        };
    }

    /// The banner's "Keep mine": the window's document is written over the file changed outside
    /// rootpoint, and the autosave resumes.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn keep_mine(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.reload_error = None;
        if let Some(autosave) = &self.autosave {
            autosave.keep_mine(self.session.document().clone());
        }
    }

    /// Open the built-in sample, in memory.
    pub fn open_sample(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.end_gestures();
        self.run(Command::Sample);
    }

    /// F5: present the shown slide, full screen.
    pub fn present(&mut self) {
        self.end_gestures();
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

    /// Open a copy of the presentation of a directory the author picked, given as each file's path
    /// under the picked directory, its first component that directory's name, and its bytes: the
    /// files become the ones the app holds in memory, under those paths, and `open-copy` copies the
    /// directory's `presentation.ron` and its assets into the session's map. A directory that opens
    /// no presentation leaves the open one and the files held as they were. Only an app made by
    /// `App::in_memory` holds them; the browser build is one.
    pub fn open_directory(&mut self, picked: Vec<(String, Vec<u8>)>) {
        if self.socket_holds() {
            return;
        }
        let Some(held) = self.held.clone() else {
            self.error = Some("this editor holds no files in memory".to_owned());
            return;
        };
        let Some(dir) = picked
            .first()
            .map(|(path, _)| path.split('/').next().unwrap_or(path).to_owned())
        else {
            self.error = Some("the picked directory holds no file".to_owned());
            return;
        };
        let before = held.replace(picked.into_iter().collect());
        self.end_gestures();
        if let Reply::Err(_) = self.run(Command::OpenCopy { dir }) {
            held.replace(before);
        }
    }

    /// Add an image the author picked, by its file name and its bytes, as an asset: it is held in
    /// memory under its name, and `import` copies it into the session's map, under its name, or
    /// the name with a number where the map holds other bytes under it.
    pub fn add_picked_image(&mut self, name: String, bytes: Vec<u8>) {
        if self.socket_holds() {
            return;
        }
        let Some(held) = self.held.clone() else {
            self.error = Some("this editor holds no files in memory".to_owned());
            return;
        };
        // The file held before under the name comes back when the import is refused, so a refused
        // action leaves the held files as they were.
        let before = held.replace_one(&name, Some(bytes));
        self.add_asset(name.clone());
        if self.error.is_some() {
            held.replace_one(&name, before);
        }
    }

    /// Ctrl+O: ask for a presentation directory, whose files the next frames read, then open it
    /// with `App::open_directory`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.web.pick_directory();
    }

    /// Ctrl+S: write the pack the session targets, or a new one with none, and hand its bytes to
    /// the page, which downloads them.
    #[cfg(target_arch = "wasm32")]
    pub fn save(&mut self) {
        if let Some((name, bytes)) = self.pack_for_download(false) {
            self.web.download(&name, &bytes);
        }
    }

    /// "Save as pack…": write a pack, and hand its bytes to the page, which downloads them,
    /// `design@rootpoint-app@browser-pack-save`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn pack_dialog(&mut self) {
        if let Some((name, bytes)) = self.pack_for_download(true) {
            self.web.download(&name, &bytes);
        }
    }

    /// "Open pack…": ask for a `.rpk` file, which the next frames read, then open it with
    /// `App::open_picked_pack`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open_pack_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.web.pick_pack();
    }

    /// Ask for an image or a font file, which the next frames read, then add it with
    /// `App::add_picked_image`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn add_asset_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        self.web.pick_image();
    }

    /// Ctrl+O: ask for a presentation directory, and open it as a workspace.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn open_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        let picked = rfd::FileDialog::new().pick_folder();
        if let Some(dir) = picked.and_then(|p| self.utf8(&p)) {
            self.open_workspace(dir);
        }
    }

    /// Ctrl+S: save the presentation to its store, `presentation.ron` at a workspace's root, or
    /// the pack a presentation held in memory was opened from or written to; with none, ask where
    /// to write a pack, as "Save as pack…" does.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self) {
        if self.socket_holds() {
            return;
        }
        if self.session.root().is_none() && self.session.pack().is_none() {
            self.pack_dialog();
            return;
        }
        self.end_gestures();
        self.run(Command::Save);
    }

    /// "Open pack…": ask for a `.rpk` file, and open it in memory.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn open_pack_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        let picked = rfd::FileDialog::new()
            .add_filter("rootpoint pack", &["rpk"])
            .pick_file();
        if let Some(file) = picked.and_then(|p| self.utf8(&p)) {
            self.open_pack(file);
        }
    }

    /// "Save as pack…" in memory, "Pack to…" in a workspace: ask for the `.rpk` file to write.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn pack_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        let picked = rfd::FileDialog::new()
            .add_filter("rootpoint pack", &["rpk"])
            .set_file_name("presentation.rpk")
            .save_file();
        if let Some(file) = picked.and_then(|p| self.utf8(&p)) {
            self.pack_to(file);
        }
    }

    /// "Save as workspace…", or "Unpack to…" once a pack is the target: ask for an empty
    /// directory, and write the presentation held in memory there as a workspace.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn unpack_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        let picked = rfd::FileDialog::new().pick_folder();
        if let Some(dir) = picked.and_then(|p| self.utf8(&p)) {
            self.unpack_to(dir);
        }
    }

    /// Ask for an image or a font file, and add it as an asset by `import`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn add_asset_dialog(&mut self) {
        if self.socket_holds() {
            return;
        }
        let Some(file) = rfd::FileDialog::new()
            .add_filter("image or font", &["png", "jpg", "jpeg", "ttf", "otf"])
            .pick_file()
        else {
            return;
        };
        if let Some(path) = self.utf8(&file) {
            self.add_asset(path);
        }
    }

    /// Ask for the file whose bytes replace an asset's, and replace them by `asset-replace`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn replace_asset_dialog(&mut self, asset: AssetId) {
        if self.socket_holds() {
            return;
        }
        let Some(file) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        if let Some(path) = self.utf8(&file) {
            self.replace_asset(asset, path);
        }
    }

    /// Ask for the file whose bytes replace an asset's, which the next frames read, then replace
    /// them with `App::replace_with_picked`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn replace_asset_dialog(&mut self, asset: AssetId) {
        if self.socket_holds() {
            return;
        }
        self.web.pick_replacement(asset);
    }

    /// A path as UTF-8, which the protocol's commands hold, or `None` with the status naming it.
    #[cfg(not(target_arch = "wasm32"))]
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

/// The actions of a text edited in place, `design@rootpoint-app@edit-in-place`: each change of the
/// text is one or two small commands of `design@rootpoint-protocol@text-edit-commands`, in the
/// group the stay holds, `design@rootpoint-app@gesture-groups`; the caret is the editor's state,
/// `design@rootpoint-app@caret-editor-state`, which `path@rootpoint-app@src/editing.rs` holds.
impl App {
    /// Enter editing the text of `area` with the caret at the point `p` of the slide view: a click on
    /// the selected text area, so a first click selects and a second edits. The stay opens an undo
    /// group, after closing any group still open.
    pub fn edit_at(&mut self, area: NodeId, p: Pos2, layout: &Layout) {
        if self.socket_holds() || !self.is_text_area(area) {
            return;
        }
        let (Some(text), Some(content)) = (self.area_text(area), self.text_content(area)) else {
            return;
        };
        let (x, y) = layout_point(&text, layout, p);
        let cursor = Selection::from_point(&text.layout, x, y).focus();
        let caret = from_byte(&content, cursor.index(), cursor.affinity());
        if self.open_group() {
            self.edit = Some(Editing {
                area,
                caret,
                anchor: None,
                affinity: cursor.affinity(),
                column: None,
                preedit: None,
            });
        }
    }

    /// Leave editing, closing the stay's group, so its edits are one undo step; the area stays
    /// selected. A composition still open is dropped from the text first, and every empty span of
    /// the text is unwrapped, #text-tree-normal-form of
    /// `milestone@plans@rich-text`, inside the group.
    pub fn stop_editing(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        if edit.preedit.is_some() {
            self.set_preedit("");
        }
        // Each unwrap can leave the span that held the unwrapped one empty, so the spans are read
        // again after each.
        while let Some(span) = self
            .text_content(edit.area)
            .and_then(|t| empty_spans(&t).first().copied())
        {
            if !matches!(self.run(Command::Unwrap { span }), Reply::Ok(_)) {
                break;
            }
        }
        self.edit = None;
        self.run(Command::End);
    }

    /// Place the caret of the text being edited at the point `p` of the slide view.
    pub fn place_caret(&mut self, p: Pos2, layout: &Layout) {
        if let Some((caret, affinity)) = self.caret_at_point(p, layout) {
            self.put_caret(caret, None, affinity, None);
        }
    }

    /// Extend the selection to the point `p` of the slide view, as a drag inside the text, or a
    /// Shift+click, does.
    pub fn extend_to_point(&mut self, p: Pos2, layout: &Layout) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        if let Some((caret, affinity)) = self.caret_at_point(p, layout) {
            let anchor = edit.anchor.unwrap_or(edit.caret);
            self.put_caret(caret, Some(anchor), affinity, None);
        }
    }

    /// Extend the selection to the point `p` of the slide view by a Shift+click.
    pub fn shift_click(&mut self, p: Pos2, layout: &Layout) {
        self.extend_to_point(p, layout);
    }

    /// Move the caret, collapsing the selection, or, with `extend`, move the selection's caret end.
    /// Left and Right follow the milestone's table of caret moves through the tree, and with
    /// `extend` pass a span whole; the moves by words and lines are Parley's on the byte index,
    /// mapped back by `editing::from_byte`.
    pub fn move_caret(&mut self, m: CaretMove, extend: bool) {
        let (Some(edit), Some(content)) = (self.edit.clone(), self.edited_text()) else {
            return;
        };
        let Some(text) = self.area_text(edit.area) else {
            return;
        };
        let selected = edit.selected(&content);
        let focus = edit.focus(&content);
        let anchor = extend.then(|| edit.anchor.unwrap_or(edit.caret));
        let mut affinity = Affinity::Downstream;
        let mut column = None;
        let caret = match m {
            // Left or Right with a selection and no Shift collapses it to that end.
            CaretMove::Left | CaretMove::Right if !extend && selected.is_some() => {
                let (node, r) = selected.unwrap_or((focus.node, 0..0));
                let offset = if m == CaretMove::Left { r.start } else { r.end };
                CaretPos { node, offset }
            }
            CaretMove::Right => right(&content, &text.map, &text.layout, focus, extend),
            CaretMove::Left => left(&content, &text.map, &text.layout, focus, extend),
            CaretMove::TextStart => CaretPos {
                node: TextNodeRef::Root,
                offset: 0,
            },
            CaretMove::TextEnd => {
                affinity = Affinity::Upstream;
                CaretPos {
                    node: TextNodeRef::Root,
                    offset: items_len(&content.root),
                }
            }
            _ => {
                let Some(index) = text.map.byte_index(focus) else {
                    return;
                };
                let l = &text.layout;
                let here = edit
                    .column
                    .filter(|s| s.focus().index() == index)
                    .unwrap_or_else(|| Selection::from_byte_index(l, index, edit.affinity));
                let moved = match m {
                    CaretMove::WordLeft => here.previous_visual_word(l, false),
                    CaretMove::WordRight => here.next_visual_word(l, false),
                    CaretMove::Up => here.previous_line(l, false),
                    CaretMove::Down => here.next_line(l, false),
                    CaretMove::LineStart => here.line_start(l, false),
                    _ => here.line_end(l, false),
                };
                if matches!(m, CaretMove::Up | CaretMove::Down) {
                    column = Some(moved);
                }
                let cursor = moved.focus();
                affinity = cursor.affinity();
                from_byte(&content, cursor.index(), affinity)
            }
        };
        self.put_caret(caret, anchor, affinity, column);
    }

    /// Select the whole text: the root's every offset.
    pub fn select_all(&mut self) {
        let Some(content) = self.edited_text() else {
            return;
        };
        let start = CaretPos {
            node: TextNodeRef::Root,
            offset: 0,
        };
        let end = CaretPos {
            offset: items_len(&content.root),
            ..start
        };
        self.put_caret(end, Some(start), Affinity::Upstream, None);
    }

    /// Put the caret at a byte offset of the text, before the glyph there, as a test does to visit
    /// every offset.
    pub fn set_caret(&mut self, index: usize) {
        if let Some(content) = self.edited_text() {
            let caret = from_byte(&content, index, Affinity::Downstream);
            self.put_caret(caret, None, Affinity::Downstream, None);
        }
    }

    /// Put the caret at a caret position of the text, as a test does at each edge of a span.
    pub fn set_caret_pos(&mut self, caret: CaretPos) {
        self.put_caret(caret, None, Affinity::Downstream, None);
    }

    /// Type a string: it replaces the selection, and the caret follows it. Enter types a new line.
    pub fn type_str(&mut self, s: &str) {
        let (Some(edit), Some(content)) = (self.edit.clone(), self.edited_text()) else {
            return;
        };
        let (node, range) = match edit.selected(&content) {
            Some(selected) => selected,
            None => (edit.caret.node, edit.caret.offset..edit.caret.offset),
        };
        self.replace(node, range, s, None);
    }

    /// Backspace: delete the selection, or the character before the caret.
    pub fn delete_backward(&mut self) {
        self.delete(false);
    }

    /// Delete: delete the selection, or the character after the caret.
    pub fn delete_forward(&mut self) {
        self.delete(true);
    }

    /// Cut: the selected text, deleted, for the clipboard. A span in the selection goes whole, and
    /// the clipboard carries its characters alone, D3 of `milestone@plans@rich-text`.
    pub fn cut(&mut self) -> Option<String> {
        let edit = self.edit.clone()?;
        let content = self.edited_text()?;
        let (node, range) = edit.selected(&content)?;
        let text = range_text(text_node(&content, node)?, range.clone());
        self.replace(node, range, "", None);
        Some(text)
    }

    /// The IME's composing text: it replaces the previous one at the caret, or the selection when
    /// composing starts, so the render draws it where it will stand; an empty one removes it.
    pub fn set_preedit(&mut self, s: &str) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        // An empty preedit with no composition open changes nothing, per
        // `design@rootpoint-app@empty-preedit`.
        if s.is_empty() && edit.preedit.is_none() {
            return;
        }
        let (node, range) = self.composed_range(&edit);
        let start = range.start;
        let preedit = (!s.is_empty()).then(|| start..start + s.chars().count());
        self.replace(node, range, s, Some(preedit));
    }

    /// The IME's final text: it replaces the composing text, which ends.
    pub fn commit_ime(&mut self, s: &str) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let (node, range) = self.composed_range(&edit);
        self.replace(node, range, s, Some(None));
    }

    /// The offsets the IME's text replaces: the composing text, or, when composing starts, the
    /// selection or the caret.
    fn composed_range(&self, edit: &Editing) -> (TextNodeRef, std::ops::Range<usize>) {
        if let Some(p) = &edit.preedit {
            return (edit.caret.node, p.clone());
        }
        let selected = self.edited_text().and_then(|t| edit.selected(&t));
        selected.unwrap_or((edit.caret.node, edit.caret.offset..edit.caret.offset))
    }

    /// Replace the offsets `range` of the text node `node` by `s`, by a `delete` and an `insert`,
    /// either left out when it would change nothing, and put the caret after `s`. `preedit` sets
    /// the composing range when given, and keeps it otherwise, shifted by the change when it lies
    /// after the edit in the same text node.
    fn replace(
        &mut self,
        node: TextNodeRef,
        range: std::ops::Range<usize>,
        s: &str,
        preedit: Option<Option<std::ops::Range<usize>>>,
    ) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let target = text_target(edit.area, node);
        if !range.is_empty()
            && !matches!(
                self.run(Command::Delete {
                    node: target,
                    range: range.clone(),
                }),
                Reply::Ok(_)
            )
        {
            return;
        }
        if !s.is_empty() {
            self.run(Command::Insert {
                node: target,
                offset: range.start,
                text: s.to_owned(),
            });
        }
        let len = s.chars().count();
        let preedit = match preedit {
            Some(p) => p,
            None => edit
                .preedit
                .filter(|_| edit.caret.node == node)
                .and_then(|p| {
                    if p.end <= range.start {
                        Some(p)
                    } else if p.start >= range.end {
                        // After the edit: moved by the change of length.
                        let shift = |i: usize| i + len - range.len();
                        Some(shift(p.start)..shift(p.end))
                    } else {
                        None
                    }
                }),
        };
        if let Some(e) = &mut self.edit {
            e.preedit = preedit;
        }
        let caret = CaretPos {
            node,
            offset: range.start + len,
        };
        self.put_caret(caret, None, Affinity::Downstream, None);
    }

    /// Delete the selection, or, with none, what is after the caret or before it in its text node,
    /// as LyX does, D8 of `milestone@plans@rich-text`: a character; a span, which a first press
    /// selects whole and a second deletes, but an empty one at once; or, at the start of a span
    /// for Backspace and at its end for Delete, the span itself, unwrapped, its content kept. At
    /// the root's start or end the key does nothing. So every edit stays in the caret's text node,
    /// where typing goes. A character is found by the caret's logical clusters, as Parley's
    /// `PlainEditor` does: Backspace deletes one character of a cluster, or a whole line break or
    /// emoji, and Delete the cluster after the caret.
    fn delete(&mut self, forward: bool) {
        let (Some(edit), Some(content)) = (self.edit.clone(), self.edited_text()) else {
            return;
        };
        if let Some((node, range)) = edit.selected(&content) {
            self.replace(node, range, "", None);
            return;
        }
        let (Some(text), Some(node)) = (
            self.area_text(edit.area),
            text_node(&content, edit.caret.node),
        ) else {
            return;
        };
        let items = items(node);
        let caret = edit.caret;
        let i = caret.offset;
        let byte = |offset| text.map.byte_index(CaretPos { offset, ..caret });
        let next = if forward {
            items.get(i)
        } else {
            i.checked_sub(1).and_then(|j| items.get(j))
        };
        let range = match next {
            // At a span's edge, inside it: the span is unwrapped, and the caret stays where it was
            // drawn, now in the span's parent.
            None => {
                let TextNodeRef::Span(span) = caret.node else {
                    return;
                };
                let Some((up, at)) = parent(&content, span) else {
                    return;
                };
                if !matches!(self.run(Command::Unwrap { span }), Reply::Ok(_)) {
                    return;
                }
                let offset = if forward { at + items.len() } else { at };
                self.put_caret(CaretPos { node: up, offset }, None, edit.affinity, None);
                return;
            }
            // A span with items is selected whole, so that a second press deletes it.
            Some(Item::Span(s)) if !s.text.items.is_empty() => {
                let other = if forward { i + 1 } else { i - 1 };
                let anchor = Some(caret);
                self.put_caret(
                    CaretPos {
                        offset: other,
                        ..caret
                    },
                    anchor,
                    edit.affinity,
                    None,
                );
                return;
            }
            Some(Item::Span(_)) if forward => i..i + 1,
            Some(Item::Span(_)) => i - 1..i,
            Some(Item::Char(_)) => {
                let Some(index) = byte(i) else {
                    return;
                };
                let cursor = Selection::from_byte_index(&text.layout, index, edit.affinity).focus();
                let [before, after] = cursor.logical_clusters(&text.layout);
                if forward {
                    let end = after.map_or(index + 1, |c| c.text_range().end);
                    let mut j = i + 1;
                    while matches!(items.get(j), Some(Item::Char(_)))
                        && byte(j).is_some_and(|b| b < end)
                    {
                        j += 1;
                    }
                    i..j
                } else {
                    let mut j = i - 1;
                    if let Some(c) = before.filter(|c| c.is_hard_line_break() || c.is_emoji()) {
                        let start = c.text_range().start;
                        while j > 0
                            && matches!(items[j - 1], Item::Char(_))
                            && byte(j - 1).is_some_and(|b| b >= start)
                        {
                            j -= 1;
                        }
                    }
                    j..i
                }
            }
        };
        self.replace(caret.node, range, "", None);
    }

    /// The caret position under the point `p` of the slide view, and its affinity: in the deepest
    /// text node that owns the glyph under it, on the side of the glyph clicked.
    fn caret_at_point(&mut self, p: Pos2, layout: &Layout) -> Option<(CaretPos, Affinity)> {
        let area = self.editing()?;
        let text = self.area_text(area)?;
        let content = self.edited_text()?;
        let (x, y) = layout_point(&text, layout, p);
        let cursor = Selection::from_point(&text.layout, x, y).focus();
        Some((
            from_byte(&content, cursor.index(), cursor.affinity()),
            cursor.affinity(),
        ))
    }

    /// The one place the caret moves. Each empty span the caret was in and has left is unwrapped,
    /// the inner first, #text-tree-normal-form of `milestone@plans@rich-text`, and
    /// the caret and the anchor are moved back where they stood after it in its parent.
    fn put_caret(
        &mut self,
        mut caret: CaretPos,
        mut anchor: Option<CaretPos>,
        affinity: Affinity,
        column: Option<Selection>,
    ) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let content = self.edited_text();
        let left: Vec<SpanId> = match &content {
            Some(content) => {
                let inside = |pos: CaretPos| spans_holding(content, pos);
                let now = inside(caret);
                let mut was = inside(edit.caret);
                was.retain(|s| !now.contains(s));
                was
            }
            None => Vec::new(),
        };
        for span in left {
            let Some(content) = self.edited_text() else {
                break;
            };
            let empty =
                text_node(&content, TextNodeRef::Span(span)).is_some_and(|n| n.items.is_empty());
            let Some((up, at)) = parent(&content, span) else {
                continue;
            };
            if !empty || !matches!(self.run(Command::Unwrap { span }), Reply::Ok(_)) {
                continue;
            }
            let shift = |pos: &mut CaretPos| {
                if pos.node == up && pos.offset > at {
                    pos.offset -= 1;
                }
            };
            shift(&mut caret);
            if let Some(a) = &mut anchor {
                shift(a);
            }
        }
        if let Some(e) = &mut self.edit {
            e.caret = caret;
            e.anchor = anchor.filter(|&a| a != caret);
            e.affinity = affinity;
            e.column = column;
        }
    }
}

/// The text node a command names: the root of `area`'s text by the area's id, or a span by its.
fn text_target(area: NodeId, node: TextNodeRef) -> TextTarget {
    match node {
        TextNodeRef::Root => TextTarget::Root(area),
        TextNodeRef::Span(span) => TextTarget::Span(span),
    }
}

/// The spans that hold a caret position, from its own text node up.
fn spans_holding(text: &TextContent, pos: CaretPos) -> Vec<SpanId> {
    let mut out = Vec::new();
    let mut node = pos.node;
    while let TextNodeRef::Span(id) = node {
        out.push(id);
        match parent(text, id) {
            Some((up, _)) => node = up,
            None => break,
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_new_area_of_a_split_is_read_from_its_reply() {
        let reply = Reply::Ok("new=n12 split=n13".to_owned());
        assert_eq!(new_area(&reply), Some(NodeId(12)));
        assert_eq!(new_area(&Reply::Ok("split=n13".to_owned())), None);
    }
}
