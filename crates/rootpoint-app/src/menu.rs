//! The node menu: a secondary click on an area of the slide, or on a row of the outline, selects
//! that node and opens a menu on it, which selects one of its ancestors, sets its content and its
//! style, or removes it.

use eframe::egui::{self, Align, Context, Id, LayerId, Layout, Pos2, Rect};
use rootpoint_model::{AssetId, Axis, Content, ImageFit, Node, NodeId, StyleChoice};

use crate::App;
use crate::selection;

/// The node menu while it is open: the node it names and where the click was. It is the window's
/// own state, which no command sets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NodeMenu {
    node: NodeId,
    at: Pos2,
    /// Which opening this is: each opening is a popup of its own, so a secondary click while the
    /// menu is open opens the new one rather than closing the old.
    serial: u64,
}

/// An entry of the node menu, and what a click on it does.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuEntry {
    /// Select this node: the menu's node or one of its ancestors.
    Select(NodeId),
    /// Make the area hold a text, as the side panel's "text" does.
    Text,
    /// Make the area hold this image, as the side panel's "image" and its image list do.
    Image(AssetId),
    /// Name this style, or with `None` make the text custom, as the side panel's style row does.
    Style(Option<String>),
    /// Remove the node, as Delete does.
    Remove,
}

/// The sections of the node menu, in order, each with its heading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSection {
    Select,
    Content,
    Style,
    Remove,
}

/// An entry as the menu shows it: its section, its label, whether it names what the node holds
/// now, and whether it acts, all but "Select" being disabled while the socket holds the authority.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub section: MenuSection,
    pub entry: MenuEntry,
    pub label: String,
    pub current: bool,
    pub enabled: bool,
}

impl App {
    /// A secondary click on `node` at `at`: leave editing, select the node, so the menu and the
    /// side panel describe one node, and open the menu on it.
    pub fn open_node_menu(&mut self, node: NodeId, at: Pos2) {
        // A secondary click inside the edited area ends editing too, before the menu opens.
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

    /// The node the menu is open on, `None` when it is closed.
    pub fn menu_node(&self) -> Option<NodeId> {
        self.menu.map(|m| m.node)
    }

    /// Where the last frame put the entry of the open menu with this label.
    pub fn menu_entry_rect(&self, label: &str) -> Option<Rect> {
        self.view
            .menu_entries
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, r)| *r)
    }

    /// The entries of the open menu, in order: "Select" with the node's ancestors from the root
    /// down and the node itself; on an area, "Content" with text, image and, on an image area, each
    /// image asset; on a text area, "Style" with the table's styles and custom; and "Remove",
    /// except on the slide's root.
    pub fn menu_items(&self) -> Vec<MenuItem> {
        let Some(menu) = self.menu else {
            return Vec::new();
        };
        let doc = self.session.document();
        let Some(slide) = self.shown_slide().and_then(|s| doc.slide(s)) else {
            return Vec::new();
        };
        let Some(path) = selection::path(slide, menu.node) else {
            return Vec::new();
        };
        let editable = !self.socket_holds();
        let item = |section, entry, label: String, current, enabled| MenuItem {
            section,
            entry,
            label,
            current,
            enabled,
        };
        let mut items: Vec<MenuItem> = path
            .iter()
            .map(|n| {
                let kind = match n {
                    Node::Split(s) => match s.axis {
                        Axis::TopToBottom => "stacked",
                        Axis::LeftToRight => "beside",
                    },
                    Node::Area(a) => match a.content {
                        Content::Text(_) => "text",
                        Content::Image(_) => "image",
                    },
                };
                let id = n.id();
                let label = format!("{id} {kind}");
                item(
                    MenuSection::Select,
                    MenuEntry::Select(id),
                    label,
                    id == menu.node,
                    true,
                )
            })
            .collect();
        if let Some(Node::Area(area)) = path.last() {
            let images = self.image_assets();
            let (text, image) = match &area.content {
                Content::Text(_) => (true, None),
                Content::Image(i) => (false, Some(i.asset)),
            };
            items.push(item(
                MenuSection::Content,
                MenuEntry::Text,
                "text".to_owned(),
                text,
                editable,
            ));
            if let Some(first) = images.first() {
                items.push(item(
                    MenuSection::Content,
                    MenuEntry::Image(*first),
                    "image".to_owned(),
                    !text,
                    editable,
                ));
            }
            if image.is_some() {
                for id in images {
                    let label = format!("{id} {}", self.asset_name(id));
                    items.push(item(
                        MenuSection::Content,
                        MenuEntry::Image(id),
                        label,
                        Some(id) == image,
                        editable,
                    ));
                }
            }
            if let Content::Text(t) = &area.content {
                for name in doc.styles.keys() {
                    let on = t.style == StyleChoice::Named(name.clone());
                    items.push(item(
                        MenuSection::Style,
                        MenuEntry::Style(Some(name.clone())),
                        name.clone(),
                        on,
                        editable,
                    ));
                }
                let custom = matches!(t.style, StyleChoice::Custom(_));
                items.push(item(
                    MenuSection::Style,
                    MenuEntry::Style(None),
                    "custom".to_owned(),
                    custom,
                    editable,
                ));
            }
        }
        if path.len() > 1 {
            items.push(item(
                MenuSection::Remove,
                MenuEntry::Remove,
                format!("Remove {}", menu.node),
                false,
                editable,
            ));
        }
        items
    }

    /// A click on an entry of the open menu: it runs the action of the side panel's equivalent, of
    /// `path@rootpoint-app@src/actions.rs`, and closes the menu. An entry that names what the node
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

    /// Close the node menu.
    pub(crate) fn close_node_menu(&mut self) {
        self.menu = None;
    }

    /// The node menu, at the click that opened it. It closes when the selection is no longer its
    /// node, since its entries act on the selection; on a click of an entry; and on a click
    /// elsewhere or Esc, as egui's `PopupCloseBehavior::CloseOnClick` does.
    pub(crate) fn node_menu(&mut self, ctx: &Context) {
        self.view.menu_entries.clear();
        let Some(menu) = self.menu else {
            return;
        };
        if self.session.view().selected != Some(menu.node) {
            self.menu = None;
            return;
        }
        let items = self.menu_items();
        let mut open = true;
        let mut clicked = None;
        let mut places = Vec::new();
        egui::Popup::new(
            Id::unique(("node menu", menu.serial)),
            ctx.clone(),
            menu.at,
            LayerId::background(),
        )
        .kind(egui::PopupKind::Menu)
        .layout(Layout::top_down_justified(Align::Min))
        .open_bool(&mut open)
        .show(|ui| {
            let mut section = None;
            for item in &items {
                if section != Some(item.section) {
                    if section.is_some() {
                        ui.separator();
                    }
                    match item.section {
                        MenuSection::Select => _ = ui.label("Select"),
                        MenuSection::Content => _ = ui.label("Content"),
                        MenuSection::Style => _ = ui.label("Style"),
                        MenuSection::Remove => {}
                    }
                    section = Some(item.section);
                }
                let response = ui.add_enabled(
                    item.enabled,
                    egui::Button::selectable(item.current, item.label.as_str()),
                );
                places.push((item.label.clone(), response.rect));
                if response.clicked() {
                    clicked = Some(item.entry.clone());
                }
            }
        });
        self.view.menu_entries = places;
        if let Some(entry) = clicked {
            self.menu_entry(entry);
        } else if !open {
            self.menu = None;
        }
    }
}
