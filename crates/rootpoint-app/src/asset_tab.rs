//! The Assets tab of the side panel: the tree of the asset paths, their folders built from the
//! paths' prefixes, with a search and the filters of `assets`; the selected asset's size, its
//! thumbnail and its uses; and the actions on it, each a command of `path@rootpoint-app@src/actions.rs`.
//! The tab reads the document and the store and runs no query through `App::run`, which would render
//! the slide again; what it reads from the files is kept until the next command, watcher event or
//! change of the store, so a frame reads no file.

use std::collections::{BTreeMap, BTreeSet};

use eframe::egui::{self, Color32, ColorImage, Sense, TextureHandle, TextureOptions, Ui};
use rootpoint_model::{AssetId, Content, FontRef, NodeId, StyleChoice, TextContent};
use rootpoint_protocol::{AssetFilter, Store};

use crate::App;

/// The widest a thumbnail is drawn, in points.
const THUMBNAIL_WIDTH: f32 = 128.0;

/// The longest side of a thumbnail's texture, in pixels: twice its width, for a screen of two
/// pixels per point, and far below the largest texture a painter takes, 8192 for egui-wgpu.
const THUMBNAIL_PIXELS: u32 = 256;

/// The image shrunk, by its nearest pixels, to fit `THUMBNAIL_PIXELS` on its longest side.
fn shrink(width: u16, height: u16, rgba: Vec<u8>) -> ([usize; 2], Vec<u8>) {
    let (w, h) = (u32::from(width), u32::from(height));
    let longest = w.max(h);
    if longest <= THUMBNAIL_PIXELS {
        return ([w as usize, h as usize], rgba);
    }
    let (tw, th) = (
        (w * THUMBNAIL_PIXELS / longest).max(1),
        (h * THUMBNAIL_PIXELS / longest).max(1),
    );
    let mut out = Vec::with_capacity((tw * th * 4) as usize);
    for y in 0..th {
        let sy = y * h / th;
        for x in 0..tw {
            let sx = x * w / tw;
            let at = ((sy * w + sx) * 4) as usize;
            out.extend_from_slice(&rgba[at..at + 4]);
        }
    }
    ([tw as usize, th as usize], out)
}

/// The Assets tab's own state, the window's and not the session's.
#[derive(Default)]
pub(crate) struct AssetTab {
    pub(crate) selected: Option<AssetId>,
    pub(crate) search: String,
    pub(crate) filter: Option<AssetFilter>,
    /// The move field's text while it is edited.
    pub(crate) path_edit: Option<String>,
    /// The relink field's text while it is edited.
    pub(crate) relink_edit: Option<String>,
    /// The asset a removal with a replacement names.
    pub(crate) replacement: Option<AssetId>,
    /// What the tab read from the files, dropped at each command and watcher event.
    pub(crate) listing: Option<Listing>,
    /// The selected image's texture, with what it was decoded from: the asset, its path and the
    /// files the session had moved.
    thumbnail: Option<(AssetId, String, u64, Option<TextureHandle>)>,
}

impl AssetTab {
    /// Drop the thumbnail, which the next frame decodes again: the store, or a file of it, may hold
    /// other bytes at its path.
    pub(crate) fn drop_thumbnail(&mut self) {
        self.thumbnail = None;
    }
}

/// What the tab read from the store's files.
pub(crate) struct Listing {
    /// Each asset's size in bytes, none for a missing one.
    sizes: BTreeMap<AssetId, Option<usize>>,
    /// The store's files that no asset names, as `rootpoint_protocol::listed_untracked` keeps them.
    pub(crate) untracked: Vec<String>,
}

/// What names an asset: an area, by its image or its own font, or a named style.
#[derive(Clone, Debug, PartialEq)]
pub enum AssetUse {
    Area(NodeId),
    Style(String),
}

/// A size in bytes as the tab writes it.
fn size_text(bytes: usize) -> String {
    match bytes {
        n if n < 1024 => format!("{n} B"),
        n if n < 1024 * 1024 => format!("{:.1} KB", n as f64 / 1024.0),
        n => format!("{:.1} MB", n as f64 / (1024.0 * 1024.0)),
    }
}

/// The folder of an asset path, `""` at the root.
fn folder_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(folder, _)| folder)
}

impl App {
    /// What names the asset, in the order of the slides and their trees, then the named styles.
    pub fn asset_uses(&self, asset: AssetId) -> Vec<AssetUse> {
        let doc = self.session.document();
        let font = FontRef::Asset(asset);
        let mut uses: Vec<AssetUse> = doc
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .filter(|a| match &a.content {
                Content::Image(image) => image.asset == asset,
                Content::Text(TextContent {
                    style: StyleChoice::Custom(style),
                    ..
                }) => style.font == font,
                Content::Text(_) => false,
            })
            .map(|a| AssetUse::Area(a.id))
            .collect();
        uses.extend(
            doc.styles
                .iter()
                .filter(|(_, s)| s.font == font)
                .map(|(name, _)| AssetUse::Style(name.clone())),
        );
        uses
    }

    /// The asset selected in the Assets tab.
    pub fn selected_asset(&self) -> Option<AssetId> {
        self.asset_tab.selected
    }

    /// Select an asset in the Assets tab, the window's own state. A path typed for the asset
    /// selected before is sent first, as the field's loss of the keyboard would send it.
    pub fn select_asset(&mut self, asset: AssetId) {
        if let (Some(before), Some(text)) =
            (self.asset_tab.selected, self.asset_tab.path_edit.take())
            && self
                .session
                .document()
                .assets
                .get(&before)
                .is_some_and(|a| a.path != text)
        {
            self.move_asset(before, text);
        }
        self.asset_tab.selected = Some(asset);
        self.asset_tab.relink_edit = None;
    }

    /// What the tab reads from the files, read once until the next command.
    pub(crate) fn listing(&mut self) -> &Listing {
        if self.asset_tab.listing.is_none() {
            let doc = self.session.document();
            let mut sizes = BTreeMap::new();
            for &id in doc.assets.keys() {
                let size = self
                    .session
                    .asset_bytes(id, &mut *self.files)
                    .ok()
                    .map(|b| b.len());
                sizes.insert(id, size);
            }
            let named: BTreeSet<&str> = doc.assets.values().map(|a| a.path.as_str()).collect();
            let held: Vec<String> = match self.session.store() {
                Store::Memory(map) => map.keys().cloned().collect(),
                Store::Workspace(root) => self.files.list(root).unwrap_or_default(),
            };
            let untracked = held
                .into_iter()
                .filter(|p| rootpoint_protocol::listed_untracked(p) && !named.contains(p.as_str()))
                .collect();
            self.asset_tab.listing = Some(Listing { sizes, untracked });
        }
        self.asset_tab.listing.as_ref().expect("built above")
    }

    /// The untracked files a relink of the asset offers, those of its file's name first, the
    /// default D11 of `milestone@plans@asset-storage`, awaiting the owner.
    pub fn relink_candidates(&mut self, asset: AssetId) -> Vec<String> {
        let name = self.asset_name(asset);
        let name = name.rsplit('/').next().unwrap_or("").to_owned();
        let mut candidates = self.listing().untracked.clone();
        candidates.sort_by_key(|p| p.rsplit('/').next() != Some(name.as_str()));
        candidates
    }

    /// The Assets tab: the search and the filter, the tree, the selected asset and its actions,
    /// and the total size. Its edits are disabled while the socket holds control, as every edit is.
    pub(crate) fn assets_tab(&mut self, ui: &mut Ui) {
        let editable = !self.socket_holds();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.asset_tab.search)
                    .hint_text("search")
                    .desired_width(110.0),
            );
            let filter = self.asset_tab.filter.unwrap_or(AssetFilter::All);
            let label = |f: AssetFilter| match f {
                AssetFilter::All => "all",
                AssetFilter::Unused => "unused",
                AssetFilter::Missing => "missing",
                AssetFilter::Untracked => "untracked",
            };
            egui::ComboBox::from_id_salt("asset filter")
                .selected_text(label(filter))
                .show_ui(ui, |ui| {
                    for f in [
                        AssetFilter::All,
                        AssetFilter::Unused,
                        AssetFilter::Missing,
                        AssetFilter::Untracked,
                    ] {
                        let response = ui.selectable_label(filter == f, label(f));
                        self.view
                            .buttons
                            .push((format!("filter {}", label(f)), response.rect));
                        if response.clicked() {
                            self.asset_tab.filter = Some(f);
                        }
                    }
                });
        });
        ui.horizontal_wrapped(|ui| {
            if self.button(ui, "Import…", editable) {
                self.add_asset_dialog();
            }
            let unused = self
                .session
                .document()
                .assets
                .keys()
                .any(|&id| self.asset_uses(id).is_empty());
            if self.button(ui, "Remove unused", editable && unused) {
                self.remove_unused_assets();
            }
        });
        ui.separator();
        self.asset_tree(ui, editable);
        ui.separator();
        self.selected_asset_view(ui, editable);
        let listing = self.listing();
        let total: usize = listing.sizes.values().flatten().sum();
        let count = listing.sizes.len();
        ui.separator();
        ui.label(format!(
            "{count} asset{}, {}",
            if count == 1 { "" } else { "s" },
            size_text(total)
        ));
    }

    /// The tree of the assets the filter and the search keep, a row per folder, which a dragged
    /// asset is dropped on to move it there, and a row per asset under its folder.
    fn asset_tree(&mut self, ui: &mut Ui, editable: bool) {
        let filter = self.asset_tab.filter.unwrap_or(AssetFilter::All);
        let search = self.asset_tab.search.to_lowercase();
        if filter == AssetFilter::Untracked {
            let untracked = self.listing().untracked.clone();
            for path in untracked
                .iter()
                .filter(|p| p.to_lowercase().contains(&search))
            {
                ui.add(egui::Label::new(path).truncate());
            }
            return;
        }
        let sizes = self.listing().sizes.clone();
        let mut rows: Vec<(AssetId, String)> = self
            .session
            .document()
            .assets
            .iter()
            .filter(|(_, a)| a.path.to_lowercase().contains(&search))
            .filter(|(id, _)| match filter {
                AssetFilter::Unused => self.asset_uses(**id).is_empty(),
                AssetFilter::Missing => sizes.get(id).is_some_and(Option::is_none),
                _ => true,
            })
            .map(|(id, a)| (*id, a.path.clone()))
            .collect();
        rows.sort_by(|a, b| (folder_of(&a.1), &a.1).cmp(&(folder_of(&b.1), &b.1)));
        // Every folder of every asset, so that an asset can be dropped on any of them, the root's
        // included, whatever the search keeps.
        let mut folders: BTreeSet<String> = BTreeSet::from([String::new()]);
        for asset in self.session.document().assets.values() {
            let mut folder = folder_of(&asset.path);
            while !folder.is_empty() {
                folders.insert(folder.to_owned());
                folder = folder_of(folder);
            }
        }
        for folder in folders {
            let depth = if folder.is_empty() {
                0
            } else {
                folder.matches('/').count() + 1
            };
            let label = if folder.is_empty() {
                "/".to_owned()
            } else {
                format!("{}/", folder.rsplit('/').next().unwrap_or(&folder))
            };
            let (response, dropped) = ui.dnd_drop_zone::<AssetId, _>(egui::Frame::new(), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(12.0 * depth as f32);
                    ui.add(egui::Label::new(egui::RichText::new(label).strong()).truncate());
                });
            });
            self.view
                .buttons
                .push((format!("folder {folder}"), response.response.rect));
            if let Some(asset) = dropped
                && editable
            {
                self.move_asset_to_folder(*asset, &folder);
            }
            for (id, path) in rows.iter().filter(|(_, p)| folder_of(p) == folder) {
                let name = path.rsplit('/').next().unwrap_or(path);
                let size = match sizes.get(id).copied().flatten() {
                    Some(n) => size_text(n),
                    None => "missing".to_owned(),
                };
                let on = self.asset_tab.selected == Some(*id);
                let response = ui
                    .horizontal(|ui| {
                        ui.add_space(12.0 * (depth + 1) as f32);
                        ui.add(
                            egui::Button::selectable(on, format!("{name}   {size}"))
                                .truncate()
                                .sense(Sense::click_and_drag()),
                        )
                    })
                    .inner;
                if editable {
                    response.dnd_set_drag_payload(*id);
                }
                self.view
                    .buttons
                    .push((format!("asset {id}"), response.rect));
                if response.clicked() {
                    self.select_asset(*id);
                }
            }
        }
    }

    /// The selected asset: its path, size and thumbnail, its uses, each a button to its node or
    /// its style, and the actions on it.
    fn selected_asset_view(&mut self, ui: &mut Ui, editable: bool) {
        let Some(asset) = self.asset_tab.selected else {
            return;
        };
        let Some(path) = self
            .session
            .document()
            .assets
            .get(&asset)
            .map(|a| a.path.clone())
        else {
            self.asset_tab.selected = None;
            return;
        };
        let size = self.listing().sizes.get(&asset).copied().flatten();
        ui.add(
            egui::Label::new(format!(
                "{asset}  {}  {path}",
                size.map_or("missing".to_owned(), size_text)
            ))
            .truncate(),
        );
        self.thumbnail(ui, asset, &path);
        let uses = self.asset_uses(asset);
        ui.horizontal_wrapped(|ui| {
            ui.label("used by");
            if uses.is_empty() {
                ui.label("nothing");
            }
            for use_ in &uses {
                match use_ {
                    AssetUse::Area(node) => {
                        if self.button(ui, &format!("use {node}"), true) {
                            self.show_use(*node);
                        }
                    }
                    AssetUse::Style(name) => {
                        if self.tab_button(ui, &format!("use style {name}"), true) {
                            self.show_tab(crate::SideTab::Styles);
                            self.select_style(name);
                        }
                    }
                }
            }
        });
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal(|ui| {
                ui.label("path");
                let mut text = self
                    .asset_tab
                    .path_edit
                    .clone()
                    .unwrap_or_else(|| path.clone());
                let response = ui.add(egui::TextEdit::singleline(&mut text).desired_width(160.0));
                if response.changed() {
                    self.asset_tab.path_edit = Some(text.clone());
                }
                // The move is sent once, when the field loses the keyboard or Enter is pressed, so
                // that typing a path is one command and one undo step; Esc leaves it unsent.
                if response.lost_focus() {
                    if text != path && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.move_asset(asset, text);
                    }
                    self.asset_tab.path_edit = None;
                }
            });
            ui.horizontal_wrapped(|ui| {
                if self.button(ui, "Replace…", editable) {
                    self.replace_asset_dialog(asset);
                }
                if self.button(ui, "Remove asset", editable && uses.is_empty()) {
                    self.remove_asset(asset);
                }
            });
            let others: Vec<(AssetId, String)> = self
                .session
                .document()
                .assets
                .iter()
                .filter(|(id, _)| **id != asset)
                .map(|(id, a)| (*id, a.path.clone()))
                .collect();
            if !uses.is_empty() && !others.is_empty() {
                ui.horizontal(|ui| {
                    let chosen = self
                        .asset_tab
                        .replacement
                        .filter(|r| others.iter().any(|(id, _)| id == r));
                    let text = chosen
                        .and_then(|r| others.iter().find(|(id, _)| *id == r))
                        .map_or("replacement…".to_owned(), |(_, p)| p.clone());
                    egui::ComboBox::from_id_salt("asset replacement")
                        .selected_text(text)
                        .show_ui(ui, |ui| {
                            for (id, p) in &others {
                                if ui.selectable_label(chosen == Some(*id), p).clicked() {
                                    self.asset_tab.replacement = Some(*id);
                                }
                            }
                        });
                    if self.button(ui, "Remove into", editable && chosen.is_some())
                        && let Some(into) = chosen
                    {
                        self.remove_asset_into(asset, into);
                    }
                });
            }
            if size.is_none() {
                ui.label("relink to");
                for candidate in self.relink_candidates(asset) {
                    if self.tab_button(ui, &format!("relink {candidate}"), editable) {
                        self.relink_asset(asset, candidate);
                    }
                }
                // A path typed, for a file the list does not offer: sent once, when the field loses
                // the keyboard or Enter is pressed.
                let mut text = self.asset_tab.relink_edit.clone().unwrap_or_default();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .hint_text("a path in the store")
                        .desired_width(160.0),
                );
                self.view
                    .buttons
                    .push(("relink path".to_owned(), response.rect));
                if response.changed() {
                    self.asset_tab.relink_edit = Some(text.clone());
                }
                if response.lost_focus() {
                    if !text.is_empty() && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.relink_asset(asset, text);
                    }
                    self.asset_tab.relink_edit = None;
                }
            }
        });
    }

    /// A button of the tab, cut to the panel's width, whose place the frame records by its label.
    fn tab_button(&mut self, ui: &mut Ui, label: &str, enabled: bool) -> bool {
        let response = ui.add_enabled(enabled, egui::Button::new(label).truncate());
        self.view.buttons.push((label.to_owned(), response.rect));
        response.clicked()
    }

    /// The selected asset's thumbnail, decoded once per asset, path and move of the store's files;
    /// a file that is no image, as a font, shows none.
    fn thumbnail(&mut self, ui: &mut Ui, asset: AssetId, path: &str) {
        let moved = self.session.files_moved();
        let fresh = matches!(&self.asset_tab.thumbnail,
            Some((a, p, m, _)) if *a == asset && p == path && *m == moved);
        if !fresh {
            let texture = self
                .session
                .asset_bytes(asset, &mut *self.files)
                .ok()
                .and_then(|bytes| rootpoint_render::decode_rgba(&bytes).ok())
                .map(|(w, h, rgba)| {
                    let (size, rgba) = shrink(w, h, rgba);
                    let image = ColorImage::from_rgba_unmultiplied(size, &rgba);
                    ui.ctx().load_texture(
                        format!("thumbnail {asset}"),
                        image,
                        TextureOptions::LINEAR,
                    )
                });
            self.asset_tab.thumbnail = Some((asset, path.to_owned(), moved, texture));
        }
        if let Some((_, _, _, Some(texture))) = &self.asset_tab.thumbnail {
            let [w, h] = texture.size();
            let scale = (THUMBNAIL_WIDTH / w as f32).min(1.0);
            let size = egui::vec2(w as f32 * scale, h as f32 * scale);
            ui.add(egui::Image::new((texture.id(), size)).bg_fill(Color32::TRANSPARENT));
        }
    }
}
