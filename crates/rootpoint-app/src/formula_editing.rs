//! The editing of a formula in the window: the caret in a math item of a text, or in an equation
//! area's formula, its moves and the edits of its keys, LyX's, by the math crate's editing of a
//! formula at a `CellPath`. Each change sends the whole formula, by `math` for a math item and
//! `equation` for an equation area, `design@rootpoint-protocol@math-edit-commands`, inside the
//! stay's undo group, `design@rootpoint-app@gesture-groups`.

use eframe::egui::{Pos2, Rect};
use parley::Affinity;
use rootpoint_math::{
    Atom, CellPath, Deletion, Formula, FormulaLayout, MatrixEdit, ampersand, caret_box, clamp,
    command_atom, complete, delete, delete_atom, edit_matrix, end, holding_atoms, insert,
    leave_cell, left, new_row, next_cell, path_at, right, script, start, typed_atom, vertical_move,
};
use rootpoint_model::{CaretPos, Content, Node, NodeId, TextItem, TextNodeRef};
use rootpoint_protocol::{Command, EquationOption, Reply, TextTarget};
use rootpoint_render::area_formula;

use crate::App;
use crate::editing::{CARET_WIDTH, CaretMove, Editing, MathCaret, items, layout_point, text_node};
use crate::hit::Layout;

/// Where the formula being edited stands: a math item of a text, by the area, the text node and
/// the item's offset, or an equation area.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Place {
    Item {
        area: NodeId,
        node: TextNodeRef,
        offset: usize,
    },
    Equation(NodeId),
}

/// A formula's layout as the slide draws it: the layout, the point of the slide, in slide units,
/// where its origin is drawn, and the scale it is drawn at.
pub(crate) struct FormulaView {
    pub layout: FormulaLayout,
    pub origin: (f64, f64),
    pub scale: f64,
}

impl FormulaView {
    /// A point of the layout, in slide units of the slide.
    fn to_slide(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.origin.0 + self.scale * x,
            self.origin.1 + self.scale * y,
        )
    }

    /// A point of the slide view, in the layout's units.
    fn view_to_layout(&self, layout: &Layout, p: Pos2) -> (f64, f64) {
        let (u, v) = layout.to_units(p);
        (
            (u - self.origin.0) / self.scale,
            (v - self.origin.1) / self.scale,
        )
    }

    /// A rectangle of the layout, in points of the slide view.
    pub(crate) fn rect(&self, layout: &Layout, x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
        let (a, b) = (self.to_slide(x0, y0), self.to_slide(x1, y1));
        Rect::from_min_max(layout.to_points(a.0, a.1), layout.to_points(b.0, b.1))
    }
}

/// The overlay's marks over the formula being edited, `App::math_marks`.
pub struct MathMarks {
    pub empty_cells: Vec<Rect>,
    /// The frames of the atoms that hold the caret, inner first.
    pub frames: Vec<Rect>,
    pub selected: Option<Rect>,
    pub command: Option<String>,
}

impl App {
    /// Whether `node` is an area of the shown slide whose content the window edits in place: a text
    /// or an equation.
    pub fn is_editable_area(&self, node: NodeId) -> bool {
        matches!(
            self.node(node),
            Some(Node::Area(a)) if matches!(a.content, Content::Text(_) | Content::Equation(_))
        )
    }

    /// Whether `node` is an area of the shown slide that holds an equation.
    pub(crate) fn is_equation_area(&self, node: NodeId) -> bool {
        matches!(
            self.node(node),
            Some(Node::Area(a)) if matches!(a.content, Content::Equation(_))
        )
    }

    /// The caret in the formula being edited, if the caret is in one.
    pub fn math_caret(&self) -> Option<CellPath> {
        Some(self.edit.as_ref()?.math.as_ref()?.path.clone())
    }

    /// The atom a first press of Backspace or Delete selected in the formula being edited.
    pub fn selected_atom(&self) -> Option<CellPath> {
        self.edit.as_ref()?.math.as_ref()?.selected.clone()
    }

    /// The command being typed after `\` in the formula being edited, by its name.
    pub fn typed_command(&self) -> Option<String> {
        self.edit.as_ref()?.math.as_ref()?.command.clone()
    }

    fn place(&self) -> Option<Place> {
        let edit = self.edit.as_ref()?;
        edit.math.as_ref()?;
        if self.is_equation_area(edit.area) {
            return Some(Place::Equation(edit.area));
        }
        Some(Place::Item {
            area: edit.area,
            node: edit.caret.node,
            offset: edit.caret.offset,
        })
    }

    /// The formula being edited, as the document holds it.
    pub fn edited_formula(&self) -> Option<Formula> {
        match self.place()? {
            Place::Equation(area) => match self.node(area)? {
                Node::Area(a) => match &a.content {
                    Content::Equation(e) => Some(e.formula.clone()),
                    _ => None,
                },
                Node::Split(_) => None,
            },
            Place::Item { area, node, offset } => {
                let content = self.text_content(area)?;
                match items(text_node(&content, node)?).get(offset)? {
                    crate::editing::Item::Object(TextItem::Math(f)) => Some(f.clone()),
                    _ => None,
                }
            }
        }
    }

    /// Send a formula for the one being edited, the whole formula replaced; whether it was taken.
    fn send_formula(&mut self, formula: Formula) -> bool {
        let command = match self.place() {
            Some(Place::Equation(area)) => Command::Equation {
                area,
                formula,
                options: Vec::new(),
            },
            Some(Place::Item { area, node, offset }) => Command::Math {
                node: target(area, node),
                offset,
                formula,
            },
            None => return false,
        };
        matches!(self.run(command), Reply::Ok(_))
    }

    /// The layout of the formula being edited, as the slide draws it.
    pub(crate) fn formula_view(&mut self) -> Option<FormulaView> {
        let slide = self.shown_slide()?;
        match self.place()? {
            Place::Equation(area) => {
                self.assets.refresh(&self.session, &mut *self.files);
                let (fonts, _) = self.assets.sets();
                let f = area_formula(self.session.document(), slide, area, fonts)?;
                Some(FormulaView {
                    layout: f.layout,
                    origin: f.origin,
                    scale: f.scale,
                })
            }
            Place::Item { area, node, offset } => {
                let text = self.area_text(area)?;
                let index = text.map.byte_index(CaretPos { node, offset })?;
                let f = text.formulas.into_iter().find(|f| f.range.start == index)?;
                Some(FormulaView {
                    layout: f.layout,
                    origin: (text.origin.0 + f.origin.0, text.origin.1 + f.origin.1),
                    scale: 1.0,
                })
            }
        }
    }

    /// The caret's rectangle in a formula, in points of the slide view, `CARET_WIDTH` wide and as
    /// tall as the cell it stands in.
    pub fn math_caret_rect(&mut self, layout: &Layout) -> Option<Rect> {
        let path = self.math_caret()?;
        let view = self.formula_view()?;
        let b = caret_box(&view.layout, &path)?;
        let r = view.rect(layout, b.x, b.y - b.ascent, b.x, b.y + b.descent.max(0.0));
        // A cell of no height, as an empty group's, gets the height of the formula's axis.
        let r = if r.height() < 2.0 {
            let axis = view.layout.axis.max(1.0);
            view.rect(layout, b.x, b.y - 2.0 * axis, b.x, b.y)
        } else {
            r
        };
        Some(Rect::from_min_max(
            r.min,
            Pos2::new(r.min.x + CARET_WIDTH, r.max.y),
        ))
    }

    /// The marks the overlay draws over the formula being edited, in points of the slide view:
    /// the outline of each empty cell, on the placeholder box the layout reserves; the frame of
    /// each atom that holds the caret, inner first; the atom a first Backspace or Delete selected;
    /// and the command being typed, `\` and its name, which the overlay writes after the caret.
    pub fn math_marks(&mut self, layout: &Layout) -> Option<MathMarks> {
        let path = self.math_caret()?;
        let selected = self.selected_atom();
        let command = self.typed_command().map(|name| format!("\\{name}"));
        let view = self.formula_view()?;
        let rect =
            |x: f64, y: f64, w: f64, a: f64, d: f64| view.rect(layout, x, y - a, x + w, y + d);
        let empty_cells = view
            .layout
            .cells
            .iter()
            .filter(|c| c.empty)
            .map(|c| rect(c.x, c.y, c.width, c.ascent, c.descent))
            .collect();
        let mut frames: Vec<Rect> = holding_atoms(&view.layout, &path)
            .into_iter()
            .map(|a| rect(a.x, a.y, a.width, a.ascent, a.descent))
            .collect();
        frames.reverse();
        let selected = selected.and_then(|at| {
            let a = view.layout.atoms.iter().find(|a| a.path == at)?;
            Some(rect(a.x, a.y, a.width, a.ascent, a.descent))
        });
        Some(MathMarks {
            empty_cells,
            frames,
            selected,
            command,
        })
    }

    /// Set the caret in the formula, closing any selection of an atom.
    pub(crate) fn set_math_caret(&mut self, path: CellPath) {
        if let Some(Editing { math: Some(m), .. }) = &mut self.edit {
            m.path = path;
            m.selected = None;
        }
    }

    /// Enter the math item after the text's caret, at its first position, or its last (`at_end`).
    pub(crate) fn enter_formula(&mut self, at_end: bool) {
        let Some(edit) = &mut self.edit else {
            return;
        };
        edit.anchor = None;
        edit.math = Some(MathCaret {
            path: start(),
            selected: None,
            command: None,
        });
        if at_end && let Some(f) = self.edited_formula() {
            self.set_math_caret(end(&f));
        }
    }

    /// Leave the formula being edited, closing the command being typed: in a text, the caret
    /// stands before the math item, or after it (`after`), and an empty item is removed, as an
    /// empty span is unwrapped; an equation area keeps its formula, empty or not, and the caret
    /// stays in it.
    pub(crate) fn leave_formula(&mut self, after: bool) {
        self.close_math_command();
        let Some(Place::Item { area, node, offset }) = self.place() else {
            return;
        };
        let empty = self.edited_formula().is_some_and(|f| f.is_empty());
        if let Some(edit) = &mut self.edit {
            edit.math = None;
            edit.anchor = None;
        }
        let offset = if empty {
            self.run(Command::Delete {
                node: target(area, node),
                range: offset..offset + 1,
            });
            offset
        } else {
            offset + usize::from(after)
        };
        if let Some(edit) = &mut self.edit {
            edit.caret = CaretPos { node, offset };
            edit.affinity = Affinity::Downstream;
        }
    }

    /// Close the command being typed after `\`: its atom goes at the caret, as LyX turns a typed
    /// command into its symbol or structure, and the caret goes after it, or into its first cell. A
    /// name the subset reads as no atom is an unknown command, drawn red; one that its LaTeX would
    /// not write back, as `\left` alone, is dropped.
    pub(crate) fn close_math_command(&mut self) {
        let Some(name) = self
            .edit
            .as_mut()
            .and_then(|e| e.math.as_mut())
            .and_then(|m| m.command.take())
        else {
            return;
        };
        if !name.is_empty() {
            let atom = command_atom(&name).unwrap_or(Atom::Unknown(name));
            self.insert_math_atom(atom);
        }
    }

    /// A move of the caret in a formula, by LyX's keys: Right and Left through every position, out
    /// of the formula at its ends; Up and Down between the cells above and below; the starts and
    /// ends of the formula for the others.
    pub(crate) fn math_move(&mut self, m: CaretMove) {
        self.close_math_command();
        let (Some(f), Some(path)) = (self.edited_formula(), self.math_caret()) else {
            return;
        };
        let moved = match m {
            CaretMove::Right => right(&f, &path),
            CaretMove::Left => left(&f, &path),
            CaretMove::Up => Some(vertical_move(&f, &path, true).unwrap_or(path.clone())),
            CaretMove::Down => Some(vertical_move(&f, &path, false).unwrap_or(path.clone())),
            CaretMove::LineStart | CaretMove::TextStart | CaretMove::WordLeft => Some(start()),
            CaretMove::LineEnd | CaretMove::TextEnd | CaretMove::WordRight => Some(end(&f)),
        };
        match moved {
            Some(p) => self.set_math_caret(p),
            None if matches!(self.place(), Some(Place::Item { .. })) => {
                self.leave_formula(m == CaretMove::Right)
            }
            None => {}
        }
    }

    /// Tab in a formula: the next cell, or, while a command is typed, its completion to the
    /// longest name the table's names that start with it share.
    pub fn math_tab(&mut self) {
        if let Some(name) = self.typed_command() {
            let completed = complete(&name);
            if completed != name {
                self.rename_command(completed);
            }
            return;
        }
        let (Some(f), Some(path)) = (self.edited_formula(), self.math_caret()) else {
            return;
        };
        if let Some(p) = next_cell(&f, &path) {
            self.set_math_caret(p);
        }
    }

    /// Ctrl+Return in a formula: a new row, of the formula or of the matrix that holds the caret.
    pub fn math_new_row(&mut self) {
        self.edit_formula(new_row);
    }

    /// An edit of the formula at the caret: the edit gives the caret's new place, and the formula
    /// changed is sent, read back as the parser reads its LaTeX, the caret kept inside it.
    fn edit_formula(&mut self, edit: impl FnOnce(&mut Formula, &CellPath) -> Option<CellPath>) {
        self.close_math_command();
        let (Some(mut f), Some(path)) = (self.edited_formula(), self.math_caret()) else {
            return;
        };
        let Some(path) = edit(&mut f, &path) else {
            return;
        };
        // A formula the parser reads otherwise than the edit left it, as a script after a group,
        // is sent as it reads, so the session takes it, and the caret is brought into it.
        let f = Formula::parse(&f.to_latex()).unwrap_or(f);
        let path = clamp(&f, &path);
        if self.send_formula(f) {
            self.set_math_caret(path);
        }
    }

    /// Insert an atom at the caret in a formula, as a typed character or the palette does.
    pub fn insert_math_atom(&mut self, atom: Atom) {
        self.edit_formula(|f, p| insert(f, p, atom));
    }

    /// Edit the rows (`rows`) or the columns of the matrix that holds the caret.
    pub fn edit_math_matrix(&mut self, rows: bool, edit: MatrixEdit) {
        self.edit_formula(|f, p| edit_matrix(f, p, rows, edit));
    }

    /// Make the atom before the caret a script's nucleus, the caret in its superscript (`sup`) or
    /// subscript.
    pub fn math_script(&mut self, sup: bool) {
        self.edit_formula(|f, p| script(f, p, sup));
    }

    /// Give the command being typed a new name. The formula is not changed until the command
    /// closes: a name being typed, as `\fra` on the way to `\frac`, can be one the subset does not
    /// read back, which the session refuses.
    fn rename_command(&mut self, name: String) {
        if let Some(m) = self.edit.as_mut().and_then(|e| e.math.as_mut())
            && m.command.is_some()
        {
            m.command = Some(name);
        }
    }

    /// A character typed in a formula, as LyX's `InsetMathNest::interpretChar` takes it, by the
    /// mapping table "A key in a formula, to what it does" of the milestone on math: a letter, a
    /// digit or an operator inserts its symbol; `^` and `_` make a script; `{` opens a group, `}`
    /// leaves it; `&` sets an alignment point or moves to a matrix's next cell; `\` opens a command
    /// name, which letters extend and any other character closes, a space being swallowed; a space
    /// leaves the cell, then the formula.
    pub fn math_type(&mut self, ch: char) {
        if let Some(name) = self.typed_command() {
            if ch.is_ascii_alphabetic() {
                self.rename_command(format!("{name}{ch}"));
                return;
            }
            if name.is_empty() {
                // `\` then a character that is no letter: the command of that character, as `\,`
                // or `\{`, or nothing.
                if let Some(m) = self.edit.as_mut().and_then(|e| e.math.as_mut()) {
                    m.command = None;
                }
                if let Some(atom) =
                    command_atom(&ch.to_string()).filter(|a| !matches!(a, Atom::Unknown(_)))
                {
                    self.insert_math_atom(atom);
                }
                return;
            }
            self.close_math_command();
            if ch == ' ' {
                return;
            }
        }
        match ch {
            ' ' | '}' => {
                let Some(path) = self.math_caret() else {
                    return;
                };
                match leave_cell(&path) {
                    Some(p) => self.set_math_caret(p),
                    None if matches!(self.place(), Some(Place::Item { .. })) => {
                        self.leave_formula(true)
                    }
                    None => {}
                }
            }
            '\\' => {
                if let Some(Editing { math: Some(m), .. }) = &mut self.edit {
                    m.selected = None;
                    m.command = Some(String::new());
                }
            }
            '^' => self.math_script(true),
            '_' => self.math_script(false),
            '{' => self.insert_math_atom(Atom::Group(Vec::new())),
            '&' => self.edit_formula(ampersand),
            '\'' => {
                if let Some(prime) = command_atom("prime") {
                    self.insert_math_atom(prime);
                }
            }
            '\n' | '\r' => {}
            _ => {
                if let Some(atom) = typed_atom(ch).or_else(|| {
                    rootpoint_math::command_of(ch).map(|c| Atom::Symbol {
                        ch: c.ch,
                        class: c.class,
                        name: Some(c.name),
                    })
                }) {
                    self.insert_math_atom(atom);
                }
            }
        }
    }

    /// Backspace (`forward` false) or Delete in a formula: an atom a first press selected goes; a
    /// symbol goes; an atom with cells, or the atom whose cell's edge the caret is at, is selected;
    /// at the formula's first or last position nothing happens. While a command is typed, its
    /// name loses its last letter, and an empty name ends it.
    pub(crate) fn math_delete(&mut self, forward: bool) {
        if let Some(name) = self.typed_command() {
            if !forward {
                let mut shorter = name.clone();
                shorter.pop();
                self.rename_command(shorter.clone());
                if shorter.is_empty()
                    && let Some(m) = self.edit.as_mut().and_then(|e| e.math.as_mut())
                {
                    m.command = None;
                }
                return;
            }
            self.close_math_command();
        }
        if let Some(at) = self.selected_atom() {
            self.edit_formula(|f, _| delete_atom(f, &at));
            return;
        }
        let (Some(mut f), Some(path)) = (self.edited_formula(), self.math_caret()) else {
            return;
        };
        match delete(&mut f, &path, forward) {
            Some(Deletion::Deleted(p)) => {
                let f = Formula::parse(&f.to_latex()).unwrap_or(f);
                let p = clamp(&f, &p);
                if self.send_formula(f) {
                    self.set_math_caret(p);
                }
            }
            Some(Deletion::Select(at)) => {
                if let Some(m) = self.edit.as_mut().and_then(|e| e.math.as_mut()) {
                    m.selected = Some(at);
                }
            }
            Some(Deletion::Nothing) | None => {}
        }
    }

    /// The point `p` of the slide view in the formula being edited, as a position of it.
    pub(crate) fn math_path_at(&mut self, p: Pos2, layout: &Layout) -> Option<CellPath> {
        let view = self.formula_view()?;
        let (x, y) = view.view_to_layout(layout, p);
        path_at(&view.layout, x, y)
    }

    /// Enter editing the formula of the equation area `area` at the point `p` of the slide view,
    /// opening the stay's group, as a second click on the selected equation area does.
    pub(crate) fn edit_equation_at(&mut self, area: NodeId, p: Pos2, layout: &Layout) {
        if !self.open_group() {
            return;
        }
        self.edit = Some(Editing {
            area,
            caret: CaretPos {
                node: TextNodeRef::Root,
                offset: 0,
            },
            anchor: None,
            affinity: Affinity::Downstream,
            column: None,
            preedit: None,
            math: Some(MathCaret {
                path: start(),
                selected: None,
                command: None,
            }),
        });
        if let Some(path) = self.math_path_at(p, layout) {
            self.set_math_caret(path);
        }
    }

    /// The math item of the edited text whose drawn box holds the point `p` of the slide view: the
    /// caret position before it, and the position in it the point gives.
    pub(crate) fn formula_at_point(
        &mut self,
        area: NodeId,
        p: Pos2,
        layout: &Layout,
    ) -> Option<(CaretPos, CellPath)> {
        let text = self.area_text(area)?;
        let content = self.text_content(area)?;
        let (x, y) = layout_point(&text, layout, p);
        let (x, y) = (f64::from(x), f64::from(y));
        let f = text.formulas.iter().find(|f| {
            x >= f.origin.0
                && x <= f.origin.0 + f.layout.width
                && y >= f.origin.1 - f.layout.ascent
                && y <= f.origin.1 + f.layout.descent
        })?;
        let caret =
            crate::editing::from_byte(&content, &text.map, f.range.start, Affinity::Downstream);
        let path = path_at(&f.layout, x - f.origin.0, y - f.origin.1)?;
        Some((caret, path))
    }

    /// Ctrl+M while a text is edited: a formula at the caret, the selection's characters its
    /// content where they read as a formula, the caret in it.
    pub fn insert_formula(&mut self) {
        if self.socket_holds() || self.math_caret().is_some() {
            return;
        }
        let (Some(edit), Some(content)) = (self.edit.clone(), self.edited_text()) else {
            return;
        };
        let (node, range) = match edit.selected(&content) {
            Some(selected) => selected,
            None => (edit.caret.node, edit.caret.offset..edit.caret.offset),
        };
        let selected = self.selected_text().unwrap_or_default();
        // The selection is the formula's content only where its LaTeX writes the selection back,
        // spaces aside, which the printer sets: a sentence parsed as math would lose its spaces.
        let bare = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let formula = Formula::parse(&selected)
            .ok()
            .filter(|f| !selected.is_empty() && bare(&f.to_latex()) == bare(&selected))
            .unwrap_or_else(Formula::empty);
        let at = target(edit.area, node);
        if !range.is_empty()
            && !matches!(
                self.run(Command::Delete {
                    node: at,
                    range: range.clone(),
                }),
                Reply::Ok(_)
            )
        {
            return;
        }
        let at_end = !formula.is_empty();
        if !matches!(
            self.run(Command::InsertMath {
                node: at,
                offset: range.start,
                formula,
            }),
            Reply::Ok(_)
        ) {
            return;
        }
        if let Some(edit) = &mut self.edit {
            edit.caret = CaretPos {
                node,
                offset: range.start,
            };
        }
        self.enter_formula(at_end);
    }

    /// Ctrl+Shift+M (`numbered` false) or Ctrl+Alt+N while an area is selected and not edited:
    /// make it an equation, its text the formula where it reads as one, as LyX's `math-display`
    /// makes a display formula of its selection.
    pub fn make_equation(&mut self, numbered: bool) {
        if self.socket_holds() || self.edit.is_some() {
            return;
        }
        let Some(area) = self.selected_area() else {
            return;
        };
        let text = self
            .text_content(area)
            .map(|t| t.plain_text())
            .unwrap_or_default();
        let formula = match self.node(area) {
            Some(Node::Area(a)) => match &a.content {
                Content::Equation(e) => e.formula.clone(),
                _ => Formula::parse(&text).unwrap_or_else(|_| Formula::empty()),
            },
            _ => return,
        };
        self.end_gestures();
        self.run(Command::Equation {
            area,
            formula,
            options: vec![EquationOption::Numbered(numbered)],
        });
    }
}

/// The text node a command names.
fn target(area: NodeId, node: TextNodeRef) -> TextTarget {
    match node {
        TextNodeRef::Root => TextTarget::Root(area),
        TextNodeRef::Span(span) => TextTarget::Span(span),
    }
}
