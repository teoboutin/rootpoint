//! The layout of a formula from a font's MATH table, by the rules of The TeXbook's appendix G with
//! the font's constants in place of TeX's font parameters: a box per atom and per cell, each tied
//! to its `CellPath`, and the glyphs and rules to draw. Every value is in the units of the size the
//! layout is asked for, and every value is that size times a value of the font in design units, so
//! a layout at `k` times a size is the layout at the size scaled by `k`: device tables are
//! ignored, and nothing is rounded.

use ttf_parser::GlyphId;
use ttf_parser::math::{GlyphConstruction, MathValue};

use crate::font::Reader;
use crate::latex::MAX_DEPTH;
use crate::table::{drawn, operator_limits};
use crate::{
    Accent, Atom, CellPath, Class, Delim, Environment, Formula, List, MathFont, MathVariant,
};

/// The style of the formula's outer level: an equation is laid out in display style, a formula of
/// a text in text style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Display,
    Text,
}

/// A formula laid out: its width, its ascent and descent from the baseline of its first row, its
/// axis, and what to draw. The origin is the left end of the first row's baseline, and `y` grows
/// downwards, as in a text's layout.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormulaLayout {
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
    /// The height of the math axis above the baseline, at the outer style's size.
    pub axis: f64,
    pub glyphs: Vec<PlacedGlyph>,
    pub rules: Vec<Rule>,
    /// A box per atom, in the order of the walk.
    pub atoms: Vec<AtomBox>,
    /// A box per cell, an empty cell's being its placeholder.
    pub cells: Vec<CellBox>,
    /// Whether a character has no glyph in the font, and is drawn as glyph 0.
    pub missing_glyph: bool,
    /// Whether an unknown command is drawn.
    pub unknown_command: bool,
}

/// A glyph to draw: its id in the font, the point of its baseline's origin, its font size, and
/// whether it spells an unknown command, drawn red.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedGlyph {
    pub glyph: u16,
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub unknown: bool,
}

/// A filled rectangle: a fraction's bar, a radical's overbar, an overline or an underline. `y` is
/// its top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The box of one atom: its path, the position before it in its list, and its extent from the
/// point `(x, y)` of its baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomBox {
    pub path: CellPath,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
}

/// The box of one cell: its path, at offset 0, its extent, and whether it is empty, when the box is
/// its placeholder, which the editor's overlay outlines.
#[derive(Clone, Debug, PartialEq)]
pub struct CellBox {
    pub path: CellPath,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
    pub empty: bool,
}

/// Lay a formula out in `style` at the font size `size`, from `font`'s MATH table.
pub fn lay_out(formula: &Formula, font: &MathFont, style: Style, size: f64) -> FormulaLayout {
    let reader = font.reader().expect("checked when the font was made");
    let ctx = Ctx {
        r: &reader,
        size,
        x_height: f64::from(reader.face.x_height().unwrap_or(450).max(0)),
    };
    let level = match style {
        Style::Display => Level::D,
        Style::Text => Level::T,
    };
    let env = Env {
        st: St {
            level,
            cramped: false,
        },
        variant: None,
        depth: 0,
    };
    let b = ctx.grid(&formula.rows, env);
    let mut layout = FormulaLayout {
        width: b.w,
        ascent: b.h,
        descent: b.d,
        axis: ctx.c(ctx.r.constants.axis_height(), env.st),
        ..FormulaLayout::default()
    };
    for item in b.items {
        match item {
            Item::Glyph(g) => {
                layout.missing_glyph |= g.glyph == 0;
                layout.unknown_command |= g.unknown;
                layout.glyphs.push(g);
            }
            Item::Rule(r) => layout.rules.push(r),
            Item::Atom(a) => layout.atoms.push(a),
            Item::Cell(c) => layout.cells.push(c),
        }
    }
    layout
}

/// TeX's four styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    D,
    T,
    S,
    SS,
}

/// A style, cramped or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct St {
    level: Level,
    cramped: bool,
}

impl St {
    fn sup(self) -> St {
        St {
            level: match self.level {
                Level::D | Level::T => Level::S,
                Level::S | Level::SS => Level::SS,
            },
            cramped: self.cramped,
        }
    }

    fn sub(self) -> St {
        St {
            cramped: true,
            ..self.sup()
        }
    }

    fn num(self) -> St {
        St {
            level: match self.level {
                Level::D => Level::T,
                Level::T => Level::S,
                Level::S | Level::SS => Level::SS,
            },
            cramped: self.cramped,
        }
    }

    fn den(self) -> St {
        St {
            cramped: true,
            ..self.num()
        }
    }

    fn cramped(self) -> St {
        St {
            cramped: true,
            ..self
        }
    }

    fn is_script(self) -> bool {
        self.level >= Level::S
    }
}

/// The style and the font change in force.
#[derive(Clone, Copy, Debug)]
struct Env {
    st: St,
    variant: Option<MathVariant>,
    /// How many lists hold the one being laid out.
    depth: usize,
}

impl Env {
    fn with(self, st: St) -> Env {
        Env { st, ..self }
    }
}

/// What a box holds, at positions relative to its origin.
#[derive(Clone, Debug)]
enum Item {
    Glyph(PlacedGlyph),
    Rule(Rule),
    Atom(AtomBox),
    Cell(CellBox),
}

impl Item {
    fn moved(mut self, dx: f64, dy: f64) -> Item {
        match &mut self {
            Item::Glyph(g) => {
                g.x += dx;
                g.y += dy;
            }
            Item::Rule(r) => {
                r.x += dx;
                r.y += dy;
            }
            Item::Atom(a) => {
                a.x += dx;
                a.y += dy;
            }
            Item::Cell(c) => {
                c.x += dx;
                c.y += dy;
            }
        }
        self
    }
}

/// A box: its width, its height and depth from its baseline, the italic correction of its last
/// glyph, which the width does not include, and, when it is one glyph, that glyph's accent
/// attachment.
#[derive(Clone, Debug, Default)]
struct Bx {
    w: f64,
    h: f64,
    d: f64,
    italic: f64,
    single: Option<Single>,
    items: Vec<Item>,
}

/// The one glyph a box is: whether a script's baseline drops apply to it, and where an accent
/// over it attaches.
#[derive(Clone, Copy, Debug)]
struct Single {
    top_accent: f64,
}

impl Bx {
    /// Put `other` with its origin at `(dx, dy)` of this box, growing this box's height and depth
    /// to hold it; the width is the caller's.
    fn put(&mut self, other: Bx, dx: f64, dy: f64) {
        self.h = self.h.max(other.h - dy);
        self.d = self.d.max(other.d + dy);
        self.items
            .extend(other.items.into_iter().map(|i| i.moved(dx, dy)));
    }
}

struct Ctx<'a, 'f> {
    r: &'a Reader<'f>,
    size: f64,
    x_height: f64,
}

/// The integral signs, whose scripts TeX never sets as limits.
fn is_integral(ch: char) -> bool {
    matches!(ch, '\u{222B}'..='\u{2233}' | '\u{2A0B}'..='\u{2A1C}')
}

impl Ctx<'_, '_> {
    /// The font size of a style.
    fn font_size(&self, st: St) -> f64 {
        let c = &self.r.constants;
        match st.level {
            Level::D | Level::T => self.size,
            Level::S => self.size * f64::from(c.script_percent_scale_down()) / 100.0,
            Level::SS => self.size * f64::from(c.script_script_percent_scale_down()) / 100.0,
        }
    }

    /// Design units to the units of a style's size.
    fn scale(&self, st: St) -> f64 {
        self.font_size(st) / self.r.upem
    }

    /// A MATH constant at a style's size; its device table is ignored.
    fn c(&self, v: MathValue<'_>, st: St) -> f64 {
        f64::from(v.value) * self.scale(st)
    }

    fn em(&self, st: St) -> f64 {
        self.font_size(st)
    }

    /// The formula's grid: one column centred, or columns aligned at `&`, right then left in
    /// turn, as amsmath's `align` sets them; rows baseline to baseline at 1.2 em at least, further
    /// where their ink would come closer than the font's `mathLeading`.
    fn grid(&self, rows: &[Vec<List>], env: Env) -> Bx {
        let columns = rows.iter().map(Vec::len).max().unwrap_or(1);
        let laid: Vec<Vec<Bx>> = rows
            .iter()
            .enumerate()
            .map(|(r, row)| {
                row.iter()
                    .enumerate()
                    .map(|(c, cell)| {
                        // An `&` stands before the relation it aligns, as amsmath's `{}` does, so a
                        // relation that opens a cell keeps the space it has after an ord.
                        let before = (c % 2 == 1).then_some(Class::Ord);
                        let path = CellPath {
                            row: r,
                            column: c,
                            ..CellPath::default()
                        };
                        self.list(cell, env, &path, before)
                    })
                    .collect()
            })
            .collect();
        let mut widths = vec![0.0f64; columns];
        for row in &laid {
            for (c, b) in row.iter().enumerate() {
                widths[c] = widths[c].max(b.w + b.italic);
            }
        }
        let em = self.em(env.st);
        // Between pairs of columns, the session's choice, near amsmath's `align` on a slide.
        let gap = 2.0 * em;
        let mut starts = Vec::with_capacity(columns);
        let mut x = 0.0;
        for (c, w) in widths.iter().enumerate() {
            if c > 0 && c % 2 == 0 {
                x += gap;
            }
            starts.push(x);
            x += w;
        }
        let total = x;
        let leading = self.c(self.r.constants.math_leading(), env.st);
        let mut out = Bx {
            w: total,
            ..Bx::default()
        };
        let mut baseline = 0.0;
        let mut previous_depth: Option<f64> = None;
        for row in laid {
            let h = row.iter().map(|b| b.h).fold(0.0, f64::max);
            let d = row.iter().map(|b| b.d).fold(0.0, f64::max);
            if let Some(pd) = previous_depth {
                baseline += (1.2 * em).max(pd + h + leading);
            }
            let row_width = if columns == 1 {
                row.first().map_or(0.0, |b| b.w + b.italic)
            } else {
                total
            };
            for (c, b) in row.into_iter().enumerate() {
                let x = if columns == 1 {
                    (total - row_width) / 2.0
                } else if c % 2 == 0 {
                    starts[c] + widths[c] - b.w - b.italic
                } else {
                    starts[c]
                };
                out.put(b, x, baseline);
            }
            previous_depth = Some(d);
        }
        out
    }

    /// A list of atoms, as one box, with the space between its atoms by their classes. `before` is
    /// the class TeX counts before the first atom, `None` at the start of a list.
    fn list(&self, list: &List, env: Env, path: &CellPath, before: Option<Class>) -> Bx {
        // Lists nest at most `MAX_DEPTH` deep in a formula `parse` gives; one built deeper is cut
        // there, so that laying it out cannot overflow the stack.
        if list.is_empty() || env.depth >= MAX_DEPTH {
            return self.placeholder(env.st, path);
        }
        let env = Env {
            depth: env.depth + 1,
            ..env
        };
        let classes = classes(list, before);
        let mut out = Bx::default();
        let mut x = 0.0;
        let mut previous: Option<Class> = before;
        let mut italic = 0.0;
        for (i, atom) in list.iter().enumerate() {
            let at = CellPath {
                offset: i,
                ..path.clone()
            };
            if let Atom::Space(space) = atom {
                x += italic;
                italic = 0.0;
                let w = space.mu() / 18.0 * self.em(env.st);
                out.items.push(Item::Atom(AtomBox {
                    path: at,
                    x,
                    y: 0.0,
                    width: w,
                    ascent: 0.0,
                    descent: 0.0,
                }));
                x += w;
                continue;
            }
            let class = classes[i].expect("every atom but a space has a class");
            if let Some(p) = previous {
                x += self.space_between(p, class, env.st);
            }
            x += italic;
            let b = self.atom(atom, env, &at);
            out.items.push(Item::Atom(AtomBox {
                path: at,
                x,
                y: 0.0,
                width: b.w,
                ascent: b.h,
                descent: b.d,
            }));
            let w = b.w;
            italic = b.italic;
            out.single = if list.len() == 1 { b.single } else { None };
            out.put(b, x, 0.0);
            x += w;
            previous = Some(class);
        }
        out.w = x;
        out.italic = italic;
        out.items.push(Item::Cell(CellBox {
            path: path.clone(),
            x: 0.0,
            y: 0.0,
            width: out.w,
            ascent: out.h,
            descent: out.d,
            empty: false,
        }));
        out
    }

    /// A group's list, or a nucleus: empty, it takes no width, as TeX gives an empty group none,
    /// so `{}^{14}\mathrm{C}` and `a{}={}b` are set as in LaTeX; its box is still marked empty.
    fn list_or_nothing(&self, list: &List, env: Env, path: &CellPath) -> Bx {
        if !list.is_empty() {
            return self.list(list, env, path, None);
        }
        Bx {
            items: vec![Item::Cell(CellBox {
                path: path.clone(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                empty: true,
            })],
            ..Bx::default()
        }
    }

    /// An empty cell's placeholder: half an em wide, as high as the font's x-height.
    fn placeholder(&self, st: St, path: &CellPath) -> Bx {
        let w = 0.5 * self.em(st);
        let h = self.x_height * self.scale(st);
        Bx {
            w,
            h,
            d: 0.0,
            items: vec![Item::Cell(CellBox {
                path: path.clone(),
                x: 0.0,
                y: 0.0,
                width: w,
                ascent: h,
                descent: 0.0,
                empty: true,
            })],
            ..Bx::default()
        }
    }

    /// TeX's space between two classes, by `math_spacing` of tex.web: a space in parentheses in
    /// the milestone's table is set in display and text styles only.
    fn space_between(&self, left: Class, right: Class, st: St) -> f64 {
        use Class::*;
        // 0 none, 1 conditional thin, 2 thin, 3 conditional medium, 4 conditional thick.
        const TABLE: [[u8; 8]; 8] = [
            [0, 2, 3, 4, 0, 0, 0, 1],
            [2, 2, 0, 4, 0, 0, 0, 1],
            [3, 3, 0, 0, 3, 0, 0, 3],
            [4, 4, 0, 0, 4, 0, 0, 4],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 2, 3, 4, 0, 0, 0, 1],
            [1, 1, 0, 1, 1, 1, 1, 1],
            [1, 2, 3, 4, 1, 0, 1, 1],
        ];
        let index = |c: Class| match c {
            Ord => 0,
            Op => 1,
            Bin => 2,
            Rel => 3,
            Open => 4,
            Close => 5,
            Punct => 6,
            Inner => 7,
        };
        let mu = match TABLE[index(left)][index(right)] {
            1 if !st.is_script() => 3.0,
            2 => 3.0,
            3 if !st.is_script() => 4.0,
            4 if !st.is_script() => 5.0,
            _ => 0.0,
        };
        mu / 18.0 * self.em(st)
    }

    fn atom(&self, atom: &Atom, env: Env, at: &CellPath) -> Bx {
        let cell = |k: usize| CellPath {
            steps: at.steps.iter().copied().chain([(at.offset, k)]).collect(),
            offset: 0,
            ..at.clone()
        };
        match atom {
            Atom::Symbol { ch, class, .. } => self.symbol(*ch, *class, env),
            Atom::Fraction { num, den } => self.fraction(num, den, env, cell(0), cell(1)),
            Atom::Radical { index, body } => {
                self.radical(index.as_ref(), body, env, cell(0), cell(1))
            }
            Atom::Scripts { nucleus, sub, sup } => {
                let mut k = 1;
                let sub_path = sub.as_ref().map(|_| {
                    k += 1;
                    cell(k - 1)
                });
                let sup_path = sup.as_ref().map(|_| cell(k));
                self.scripts(
                    nucleus,
                    sub.as_ref(),
                    sup.as_ref(),
                    env,
                    cell(0),
                    sub_path,
                    sup_path,
                )
            }
            Atom::Delimited { open, close, body } => {
                let b = self.list(body, env, &cell(0), None);
                self.delimited(*open, *close, b, env.st)
            }
            Atom::Accent { accent, body } => self.accent(*accent, body, env, cell(0)),
            // `\liminf` and `\limsup` are `lim\,inf` and `lim\,sup` in ltmath.dtx.
            Atom::OperatorName(name) => match name.strip_prefix("lim").filter(|r| !r.is_empty()) {
                Some(rest) => {
                    let mut out = self.upright("lim", env.st, false);
                    let thin = 3.0 / 18.0 * self.em(env.st);
                    let r = self.upright(rest, env.st, false);
                    let x = out.w + thin;
                    out.w = x + r.w;
                    out.put(r, x, 0.0);
                    out
                }
                None => self.upright(name, env.st, false),
            },
            Atom::FontChange { font, body } => self.list(
                body,
                Env {
                    variant: Some(*font),
                    ..env
                },
                &cell(0),
                None,
            ),
            Atom::Text(text) => self.upright(&text_characters(text), env.st, false),
            Atom::Matrix { env: kind, rows } => self.matrix(*kind, rows, env, at),
            Atom::Space(_) => Bx::default(),
            Atom::Group(list) => self.list_or_nothing(list, env, &cell(0)),
            Atom::Unknown(name) => {
                let spelled = format!("\\{name}");
                self.upright(&spelled, env.st, true)
            }
        }
    }

    /// One glyph at a style: its `ssty` form in a script style, its box from its ink.
    fn glyph(&self, glyph: GlyphId, st: St, unknown: bool) -> Bx {
        let glyph = if st.is_script() {
            self.r.script_glyph(glyph)
        } else {
            glyph
        };
        let s = self.scale(st);
        let (h, d) = self.r.ink(glyph);
        let w = self.r.advance(glyph) * s;
        Bx {
            w,
            h: h * s,
            d: d * s,
            italic: self.r.italic_correction(glyph) * s,
            single: Some(Single {
                top_accent: self.r.top_accent(glyph).map_or(w / 2.0, |a| a * s),
            }),
            items: vec![Item::Glyph(PlacedGlyph {
                glyph: glyph.0,
                x: 0.0,
                y: 0.0,
                size: self.font_size(st),
                unknown,
            })],
        }
    }

    fn glyph_of(&self, ch: char) -> GlyphId {
        self.r.face.glyph_index(ch).unwrap_or(GlyphId(0))
    }

    /// A character atom: its code point under the font change in force; a large operator in
    /// display style takes the font's display variant, and every large operator is centred on the
    /// axis.
    fn symbol(&self, ch: char, class: Class, env: Env) -> Bx {
        let ch = font_change(drawn(ch), env.variant);
        let glyph = self.glyph_of(ch);
        if class == Class::Op
            && let Some(construction) = self.r.vertical(glyph)
        {
            let glyph = if env.st.level == Level::D {
                let min = f64::from(self.r.constants.display_operator_min_height());
                display_variant(&construction, min).unwrap_or(glyph)
            } else {
                glyph
            };
            let mut b = self.glyph(glyph, env.st, false);
            let axis = self.c(self.r.constants.axis_height(), env.st);
            let shift = (b.h - b.d) / 2.0 - axis;
            // Centred, the operator is a box of its own, as TeX's rule 13 makes it, so a script's
            // baseline drops apply to it, and an accent over it attaches at its middle.
            let mut out = Bx {
                w: b.w,
                italic: b.italic,
                ..Bx::default()
            };
            b.single = None;
            out.put(b, 0.0, shift);
            return out;
        }
        self.glyph(glyph, env.st, false)
    }

    /// Letters set upright from the font's own Latin glyphs: an operator name, a `\text` run, or an
    /// unknown command spelled with its backslash.
    fn upright(&self, text: &str, st: St, unknown: bool) -> Bx {
        let mut out = Bx::default();
        let mut x = 0.0;
        for ch in text.chars() {
            let b = self.glyph(self.glyph_of(ch), st, unknown);
            let w = b.w;
            out.put(b, x, 0.0);
            x += w;
        }
        out.w = x;
        out
    }

    fn fraction(&self, num: &List, den: &List, env: Env, np: CellPath, dp: CellPath) -> Bx {
        let c = &self.r.constants;
        let st = env.st;
        let display = st.level == Level::D;
        let n = self.list(num, env.with(st.num()), &np, None);
        let d = self.list(den, env.with(st.den()), &dp, None);
        let axis = self.c(c.axis_height(), st);
        let t = self.c(c.fraction_rule_thickness(), st);
        let (mut u, mut v, gn, gd) = if display {
            (
                self.c(c.fraction_numerator_display_style_shift_up(), st),
                self.c(c.fraction_denominator_display_style_shift_down(), st),
                self.c(c.fraction_num_display_style_gap_min(), st),
                self.c(c.fraction_denom_display_style_gap_min(), st),
            )
        } else {
            (
                self.c(c.fraction_numerator_shift_up(), st),
                self.c(c.fraction_denominator_shift_down(), st),
                self.c(c.fraction_numerator_gap_min(), st),
                self.c(c.fraction_denominator_gap_min(), st),
            )
        };
        // The numerator's bottom keeps its gap above the bar, the denominator's top below it.
        u = u.max(axis + t / 2.0 + gn + n.d);
        v = v.max(d.h + gd + t / 2.0 - axis);
        // TeX's `\nulldelimiterspace`, 1.2 pt at 10 pt, on each side.
        let pad = 0.12 * self.em(st);
        let inner = n.w.max(d.w);
        let mut out = Bx {
            w: inner + 2.0 * pad,
            ..Bx::default()
        };
        let (nw, dw) = (n.w, d.w);
        out.put(n, pad + (inner - nw) / 2.0, -u);
        out.put(d, pad + (inner - dw) / 2.0, v);
        out.items.push(Item::Rule(Rule {
            x: pad,
            y: -(axis + t / 2.0),
            width: inner,
            height: t,
        }));
        out.h = out.h.max(axis + t / 2.0);
        out
    }

    fn radical(
        &self,
        index: Option<&List>,
        body: &List,
        env: Env,
        bp: CellPath,
        ip: CellPath,
    ) -> Bx {
        let c = &self.r.constants;
        let st = env.st;
        let b = self.list(body, env.with(st.cramped()), &bp, None);
        let mut gap = if st.level == Level::D {
            self.c(c.radical_display_style_vertical_gap(), st)
        } else {
            self.c(c.radical_vertical_gap(), st)
        };
        let t = self.c(c.radical_rule_thickness(), st);
        let extra = self.c(c.radical_extra_ascender(), st);
        let target = b.h + b.d + gap + t;
        let sign = self.stretched('\u{221A}', target, st);
        let total = sign.h + sign.d;
        if total > target {
            gap += (total - target) / 2.0;
        }
        // The sign's top meets the rule's top.
        let top = b.h + gap + t;
        let dy = sign.h - top;
        let mut out = Bx::default();
        let mut x = 0.0;
        if let Some(index) = index {
            let i = self.list(
                index,
                // `\root` sets its index in `\scriptscriptstyle`, uncramped.
                env.with(St {
                    level: Level::SS,
                    cramped: false,
                }),
                &ip,
                None,
            );
            let before = self.c(c.radical_kern_before_degree(), st);
            let after = self.c(c.radical_kern_after_degree(), st);
            let raise = f64::from(c.radical_degree_bottom_raise_percent()) / 100.0 * total;
            let bottom = sign.d + dy;
            let iw = i.w;
            let id = i.d;
            out.put(i, before, bottom - raise - id);
            x = (before + iw + after).max(0.0);
        }
        let sw = sign.w;
        out.put(sign, x, dy);
        x += sw;
        let bw = b.w;
        out.put(b, x, 0.0);
        out.items.push(Item::Rule(Rule {
            x,
            y: -top,
            width: bw,
            height: t,
        }));
        out.w = x + bw;
        out.h = out.h.max(top + extra);
        out
    }

    /// A glyph of a character made at least `target` high, by the font's variants, else its
    /// assembly, else its largest variant.
    fn stretched(&self, ch: char, target: f64, st: St) -> Bx {
        let base = self.glyph_of(ch);
        let s = self.scale(st);
        let Some(construction) = self.r.vertical(base) else {
            return self.plain_glyph(base, st);
        };
        for v in construction.variants {
            if f64::from(v.advance_measurement) * s >= target {
                return self.plain_glyph(v.variant_glyph, st);
            }
        }
        if let Some(assembly) = construction.assembly {
            return self.assembly(assembly, target, st);
        }
        let last = construction
            .variants
            .last()
            .map_or(base, |v| v.variant_glyph);
        self.plain_glyph(last, st)
    }

    /// A glyph at a style's size, with no `ssty` form.
    fn plain_glyph(&self, glyph: GlyphId, st: St) -> Bx {
        let s = self.scale(st);
        let (h, d) = self.r.ink(glyph);
        Bx {
            w: self.r.advance(glyph) * s,
            h: h * s,
            d: d * s,
            italic: self.r.italic_correction(glyph) * s,
            single: None,
            items: vec![Item::Glyph(PlacedGlyph {
                glyph: glyph.0,
                x: 0.0,
                y: 0.0,
                size: self.font_size(st),
                unknown: false,
            })],
        }
    }

    /// A vertical assembly at least `target` high: the fewest repeats of its extenders that
    /// reach it, the overlaps between parts shared equally, never below the font's minimum.
    fn assembly(&self, assembly: ttf_parser::math::GlyphAssembly<'_>, target: f64, st: St) -> Bx {
        let s = self.scale(st);
        let min_overlap = f64::from(self.r.variants.min_connector_overlap) * s;
        let parts: Vec<_> = assembly.parts.into_iter().collect();
        let mut repeats = 0usize;
        let sequence = loop {
            let seq: Vec<_> = parts
                .iter()
                .flat_map(|p| {
                    let n = if p.part_flags.extender() { repeats } else { 1 };
                    std::iter::repeat_n(*p, n)
                })
                .collect();
            let advance: f64 = seq.iter().map(|p| f64::from(p.full_advance) * s).sum();
            let joints = seq.len().saturating_sub(1) as f64;
            let longest = advance - joints * min_overlap;
            if longest >= target || repeats > 64 || !parts.iter().any(|p| p.part_flags.extender()) {
                break seq;
            }
            repeats += 1;
        };
        let advance: f64 = sequence.iter().map(|p| f64::from(p.full_advance) * s).sum();
        let joints = sequence.len().saturating_sub(1);
        let largest_overlap = sequence
            .windows(2)
            .map(|w| f64::from(w[0].end_connector_length.min(w[1].start_connector_length)) * s)
            .fold(f64::INFINITY, f64::min);
        let overlap = if joints == 0 {
            0.0
        } else {
            ((advance - target) / joints as f64)
                .min(largest_overlap.max(min_overlap))
                .max(min_overlap)
        };
        // Parts go from the bottom up, the first part's baseline at the assembly's; its box is the
        // parts' ink, which the caller centres.
        let mut out = Bx::default();
        let mut bottom = 0.0;
        let mut width = 0.0f64;
        for p in &sequence {
            let g = self.plain_glyph(p.glyph_id, st);
            width = width.max(g.w);
            out.put(g, 0.0, -bottom);
            bottom += f64::from(p.full_advance) * s - overlap;
        }
        out.w = width;
        out
    }

    /// A delimiter of a character centred on the axis, at least `size` high.
    fn delimiter(&self, delim: Option<Delim>, size: f64, st: St) -> Bx {
        let Some(delim) = delim else {
            return Bx {
                w: 0.12 * self.em(st),
                ..Bx::default()
            };
        };
        let mut g = self.stretched(delim.ch, size, st);
        let axis = self.c(self.r.constants.axis_height(), st);
        let shift = (g.h - g.d) / 2.0 - axis;
        let mut out = Bx {
            w: g.w,
            ..Bx::default()
        };
        g.single = None;
        out.put(g, 0.0, shift);
        out
    }

    /// `\left`, a body, `\right`: each delimiter as tall as TeX makes it, at least 90.1 % of twice
    /// the body's larger extent from the axis, and short of it by at most half an em.
    fn delimited(&self, open: Option<Delim>, close: Option<Delim>, body: Bx, st: St) -> Bx {
        let axis = self.c(self.r.constants.axis_height(), st);
        let extent = (body.h - axis).max(body.d + axis);
        let size = (2.0 * extent * 0.901).max(2.0 * extent - 0.5 * self.em(st));
        let l = self.delimiter(open, size, st);
        let r = self.delimiter(close, size, st);
        let mut out = Bx::default();
        let lw = l.w;
        out.put(l, 0.0, 0.0);
        let bw = body.w + body.italic;
        out.put(body, lw, 0.0);
        let rw = r.w;
        out.put(r, lw + bw, 0.0);
        out.w = lw + bw + rw;
        out
    }

    fn accent(&self, accent: Accent, body: &List, env: Env, bp: CellPath) -> Bx {
        let c = &self.r.constants;
        let st = env.st;
        let b = self.list(body, env.with(st.cramped()), &bp, None);
        let mut out = Bx::default();
        match accent.mark() {
            None if accent == Accent::Overline => {
                let gap = self.c(c.overbar_vertical_gap(), st);
                let t = self.c(c.overbar_rule_thickness(), st);
                let extra = self.c(c.overbar_extra_ascender(), st);
                let (bw, bh) = (b.w, b.h);
                out.put(b, 0.0, 0.0);
                out.items.push(Item::Rule(Rule {
                    x: 0.0,
                    y: -(bh + gap + t),
                    width: bw,
                    height: t,
                }));
                out.w = bw;
                out.h = out.h.max(bh + gap + t + extra);
            }
            None => {
                let gap = self.c(c.underbar_vertical_gap(), st);
                let t = self.c(c.underbar_rule_thickness(), st);
                let extra = self.c(c.underbar_extra_descender(), st);
                let (bw, bd) = (b.w, b.d);
                out.put(b, 0.0, 0.0);
                out.items.push(Item::Rule(Rule {
                    x: 0.0,
                    y: bd + gap,
                    width: bw,
                    height: t,
                }));
                out.w = bw;
                out.d = out.d.max(bd + gap + t + extra);
            }
            Some(mark) => {
                let glyph = self.glyph_of(mark);
                let s = self.scale(st);
                let a = self.plain_glyph(glyph, st);
                let attach_mark = self.r.top_accent(glyph).map(|v| v * s).unwrap_or_else(|| {
                    self.r
                        .face
                        .glyph_bounding_box(glyph)
                        .map_or(a.w / 2.0, |bb| f64::from(bb.x_min + bb.x_max) / 2.0 * s)
                });
                let attach_body = b.single.map_or(b.w / 2.0, |one| one.top_accent);
                let base = self.c(c.accent_base_height(), st);
                let raise = (b.h - base).max(0.0);
                let (bw, bi) = (b.w, b.italic);
                out.put(b, 0.0, 0.0);
                out.put(a, attach_body - attach_mark, -raise);
                out.w = bw;
                out.italic = bi;
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn scripts(
        &self,
        nucleus: &List,
        sub: Option<&List>,
        sup: Option<&List>,
        env: Env,
        np: CellPath,
        subp: Option<CellPath>,
        supp: Option<CellPath>,
    ) -> Bx {
        let c = &self.r.constants;
        let st = env.st;
        let n = self.list_or_nothing(nucleus, env, &np);
        let sub_box = sub.map(|l| self.list(l, env.with(st.sub()), subp.as_ref().unwrap(), None));
        let sup_box = sup.map(|l| self.list(l, env.with(st.sup()), supp.as_ref().unwrap(), None));
        let limits = match nucleus.as_slice() {
            [
                Atom::Symbol {
                    ch,
                    class: Class::Op,
                    ..
                },
            ] => st.level == Level::D && !is_integral(*ch),
            [Atom::OperatorName(name)] => st.level == Level::D && operator_limits(name),
            _ => false,
        };
        if limits {
            return self.limits(n, sub_box, sup_box, st);
        }
        let script = st.sup();
        let mut u = if st.cramped {
            self.c(c.superscript_shift_up_cramped(), st)
        } else {
            self.c(c.superscript_shift_up(), st)
        };
        let mut v = self.c(c.subscript_shift_down(), st);
        if n.single.is_none() {
            u = u.max(n.h - self.c(c.superscript_baseline_drop_max(), script));
            v = v.max(n.d + self.c(c.subscript_baseline_drop_min(), script));
        }
        if let Some(b) = &sub_box {
            v = v.max(b.h - self.c(c.subscript_top_max(), st));
        }
        if let Some(b) = &sup_box {
            u = u.max(self.c(c.superscript_bottom_min(), st) + b.d);
        }
        if let (Some(p), Some(b)) = (&sup_box, &sub_box) {
            let gap = (u - p.d) - (b.h - v);
            let min = self.c(c.sub_superscript_gap_min(), st);
            if gap < min {
                v += min - gap;
                let lift = self.c(c.superscript_bottom_max_with_subscript(), st) - (u - p.d);
                if lift > 0.0 {
                    u += lift;
                    v -= lift;
                }
            }
        }
        let after = self.c(c.space_after_script(), st);
        let (nw, ni) = (n.w, n.italic);
        let mut out = Bx::default();
        out.put(n, 0.0, 0.0);
        let mut w = nw + ni;
        if let Some(p) = sup_box {
            let pw = p.w + p.italic;
            out.put(p, nw + ni, -u);
            w = w.max(nw + ni + pw);
        }
        if let Some(b) = sub_box {
            let bw = b.w + b.italic;
            out.put(b, nw, v);
            w = w.max(nw + bw);
        }
        out.w = w + after;
        out
    }

    /// A large operator with its scripts as limits, above and below, centred, each moved by half
    /// the operator's italic correction.
    fn limits(&self, op: Bx, sub: Option<Bx>, sup: Option<Bx>, st: St) -> Bx {
        let c = &self.r.constants;
        let width =
            op.w.max(sub.as_ref().map_or(0.0, |b| b.w))
                .max(sup.as_ref().map_or(0.0, |b| b.w));
        let italic = op.italic;
        let (oh, od, ow) = (op.h, op.d, op.w);
        let mut out = Bx::default();
        out.put(op, (width - ow) / 2.0, 0.0);
        if let Some(p) = sup {
            let rise = (self.c(c.upper_limit_gap_min(), st) + p.d)
                .max(self.c(c.upper_limit_baseline_rise_min(), st));
            let pw = p.w;
            out.put(p, (width - pw) / 2.0 + italic / 2.0, -(oh + rise));
        }
        if let Some(b) = sub {
            let drop = (self.c(c.lower_limit_gap_min(), st) + b.h)
                .max(self.c(c.lower_limit_baseline_drop_min(), st));
            let bw = b.w;
            out.put(b, (width - bw) / 2.0 - italic / 2.0, od + drop);
        }
        out.w = width;
        out
    }

    /// A matrix: its cells in text style, as LaTeX's array sets every cell whatever the style
    /// around it, columns apart by one em, rows by struts of 1.2 em, the whole centred on the
    /// axis, with the environment's delimiters.
    fn matrix(&self, kind: Environment, rows: &[Vec<List>], env: Env, at: &CellPath) -> Bx {
        let st = St {
            level: Level::T,
            cramped: env.st.cramped,
        };
        let em = self.em(st);
        // amsmath's cases sets `\arraystretch` to 1.2.
        let stretch = if kind == Environment::Cases { 1.2 } else { 1.0 };
        let strut_h = 0.7 * 1.2 * em * stretch;
        let strut_d = 0.3 * 1.2 * em * stretch;
        let mut k = 0;
        let laid: Vec<Vec<Bx>> = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        let path = CellPath {
                            steps: at.steps.iter().copied().chain([(at.offset, k)]).collect(),
                            offset: 0,
                            ..at.clone()
                        };
                        k += 1;
                        self.list(cell, env.with(st), &path, None)
                    })
                    .collect()
            })
            .collect();
        let columns = laid.iter().map(Vec::len).max().unwrap_or(0);
        let mut widths = vec![0.0f64; columns];
        for row in &laid {
            for (c, b) in row.iter().enumerate() {
                widths[c] = widths[c].max(b.w + b.italic);
            }
        }
        let gap = em;
        let total_w = widths.iter().sum::<f64>() + gap * columns.saturating_sub(1) as f64;
        let mut grid = Bx {
            w: total_w,
            ..Bx::default()
        };
        let mut y = 0.0;
        let mut first = true;
        let mut first_h = 0.0;
        for row in laid {
            let h = row.iter().map(|b| b.h).fold(strut_h, f64::max);
            let d = row.iter().map(|b| b.d).fold(strut_d, f64::max);
            if first {
                first_h = h;
                first = false;
            } else {
                y += h;
            }
            let mut x = 0.0;
            for (c, b) in row.into_iter().enumerate() {
                let dx = if kind == Environment::Cases {
                    0.0
                } else {
                    (widths[c] - b.w - b.italic) / 2.0
                };
                grid.put(b, x + dx, y);
                x += widths[c] + gap;
            }
            y += d;
        }
        let height = first_h + y;
        let axis = self.c(self.r.constants.axis_height(), st);
        // The grid's top goes where its middle is on the axis.
        let shift = -(axis + height / 2.0) + first_h;
        let mut centred = Bx {
            w: grid.w,
            ..Bx::default()
        };
        grid.h = first_h;
        grid.d = y;
        centred.put(grid, 0.0, shift);
        let (open, close) = match kind {
            Environment::Matrix => return centred,
            Environment::PMatrix => ('(', ')'),
            Environment::BMatrix => ('[', ']'),
            Environment::Cases => ('{', '\0'),
        };
        let delim = |ch: char| (ch != '\0').then_some(Delim { ch, name: None });
        self.delimited(delim(open), delim(close), centred, st)
    }
}

/// The first of a construction's variants at least `min` high, in design units, else its largest.
fn display_variant(construction: &GlyphConstruction<'_>, min: f64) -> Option<GlyphId> {
    let mut last = None;
    for v in construction.variants {
        last = Some(v.variant_glyph);
        if f64::from(v.advance_measurement) >= min {
            return last;
        }
    }
    last
}

/// The class of each atom, `None` for a space, after TeX's conversions of a bin to an ord, by
/// §728 and §729 of tex.web: a bin first, or after a bin, an op, a rel, an open or a punct; a bin
/// last, or before a rel, a close or a punct.
fn classes(list: &List, before: Option<Class>) -> Vec<Option<Class>> {
    let mut out: Vec<Option<Class>> = list.iter().map(atom_class).collect();
    let mut previous: Option<(usize, Class)> = None;
    for i in 0..out.len() {
        let Some(class) = out[i] else { continue };
        let prev_class = previous.map(|(_, c)| c).or(before);
        if class == Class::Bin
            && matches!(
                prev_class,
                None | Some(Class::Bin | Class::Op | Class::Rel | Class::Open | Class::Punct)
            )
        {
            out[i] = Some(Class::Ord);
        } else if matches!(class, Class::Rel | Class::Close | Class::Punct)
            && let Some((p, Class::Bin)) = previous
        {
            out[p] = Some(Class::Ord);
        }
        previous = Some((i, out[i].unwrap()));
    }
    if let Some((p, Class::Bin)) = previous {
        out[p] = Some(Class::Ord);
    }
    out
}

/// An atom's class: a symbol's own; a script's, its nucleus's when the nucleus is one atom; inner
/// for a fraction, a delimited group and a matrix with delimiters; op for an operator name; ord
/// for the rest; none for a space.
fn atom_class(atom: &Atom) -> Option<Class> {
    Some(match atom {
        Atom::Symbol { class, .. } => *class,
        Atom::Scripts { nucleus, .. } => match nucleus.as_slice() {
            [one] => atom_class(one).unwrap_or(Class::Ord),
            _ => Class::Ord,
        },
        Atom::Fraction { .. } | Atom::Delimited { .. } => Class::Inner,
        Atom::Matrix { env, .. } if *env != Environment::Matrix => Class::Inner,
        Atom::OperatorName(_) => Class::Op,
        Atom::Space(_) => return None,
        _ => Class::Ord,
    })
}

/// The code point a character is drawn with under a font change, by the milestone's mapping
/// table "A character under a font change, to the code point drawn", the holes of the
/// Mathematical Alphanumeric Symbols filled from the Letterlike Symbols.
pub(crate) fn font_change(ch: char, variant: Option<MathVariant>) -> char {
    let latin = |base: u32| -> Option<char> {
        let i = match ch {
            'A'..='Z' => ch as u32 - 'A' as u32,
            'a'..='z' => ch as u32 - 'a' as u32 + 26,
            _ => return None,
        };
        char::from_u32(base + i)
    };
    let hole = |table: &[(char, char)]| table.iter().find(|(c, _)| *c == ch).map(|(_, h)| *h);
    // Lowercase Greek, ς included, and the lowercase variant letters, in the order of the
    // italic and bold blocks, which follow ω with ∂ ϵ ϑ ϰ ϕ ϱ ϖ.
    let lower_greek = |base: u32| -> Option<char> {
        let i = match ch {
            'α'..='ω' => ch as u32 - 'α' as u32,
            'ϵ' => 26,
            'ϑ' => 27,
            'ϰ' => 28,
            'ϕ' => 29,
            'ϱ' => 30,
            'ϖ' => 31,
            _ => return None,
        };
        char::from_u32(base + i)
    };
    let upper_greek = |base: u32| -> Option<char> {
        match ch {
            'Α'..='Ω' => char::from_u32(base + (ch as u32 - 'Α' as u32)),
            _ => None,
        }
    };
    let digit = |base: u32| ch.to_digit(10).and_then(|d| char::from_u32(base + d));
    let mapped = match variant {
        None | Some(MathVariant::Italic) => {
            if ch == 'h' {
                Some('\u{210E}')
            } else if let Some(c) = latin(0x1D434) {
                Some(c)
            } else if let Some(c) = lower_greek(0x1D6FC) {
                Some(c)
            } else if variant == Some(MathVariant::Italic) {
                upper_greek(0x1D6E2)
            } else {
                None
            }
        }
        Some(MathVariant::Roman) => None,
        Some(MathVariant::Bold) => latin(0x1D400)
            .or_else(|| lower_greek(0x1D6C2))
            .or_else(|| upper_greek(0x1D6A8))
            .or_else(|| digit(0x1D7CE)),
        Some(MathVariant::Calligraphic) => hole(&[
            ('B', '\u{212C}'),
            ('E', '\u{2130}'),
            ('F', '\u{2131}'),
            ('H', '\u{210B}'),
            ('I', '\u{2110}'),
            ('L', '\u{2112}'),
            ('M', '\u{2133}'),
            ('R', '\u{211B}'),
            ('e', '\u{212F}'),
            ('g', '\u{210A}'),
            ('o', '\u{2134}'),
        ])
        .or_else(|| latin(0x1D49C)),
        Some(MathVariant::DoubleStruck) => hole(&[
            ('C', '\u{2102}'),
            ('H', '\u{210D}'),
            ('N', '\u{2115}'),
            ('P', '\u{2119}'),
            ('Q', '\u{211A}'),
            ('R', '\u{211D}'),
            ('Z', '\u{2124}'),
        ])
        .or_else(|| latin(0x1D538))
        .or_else(|| digit(0x1D7D8)),
    };
    mapped.unwrap_or(ch)
}

/// The characters a `\text` run draws: each of LaTeX's escaped special characters, as `\$` or
/// `\{`, as the character itself, the rest as written.
fn text_characters(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match chars.peek() {
            Some(&next) if ch == '\\' && "$%&#_{}".contains(next) => {
                out.push(next);
                chars.next();
            }
            // Any other backslash keeps the character after it, so `\\` is read as the parser
            // reads it, one token.
            Some(&next) if ch == '\\' => {
                out.push(ch);
                out.push(next);
                chars.next();
            }
            _ => out.push(ch),
        }
    }
    out
}
