//! Where the shown slide's nodes and dividers lie in the slide view, in points, and what a press
//! there hits. The slide is placed by `slide_to_pixels`, so every mark and every hit falls where
//! `render_slide` draws.

use eframe::egui::{Pos2, Rect, Vec2};
use rootpoint_model::{Axis, Document, Node, NodeId, Rect as SlideRect, SlideId, node_rects};
use rootpoint_render::slide_to_pixels;

/// How far from a divider, in points, a press grabs it.
pub const GRAB: f32 = 4.0;

/// The least extent of a split's child along the split's axis, in slide units, that a divider drag
/// keeps.
pub const MIN_PART: f64 = 40.0;

/// A divider of a split: the line between its children `index` and `index + 1`.
#[derive(Clone, Debug, PartialEq)]
pub struct Divider {
    pub split: NodeId,
    pub index: usize,
    pub axis: Axis,
    /// The ends of the line, in points.
    pub from: Pos2,
    pub to: Pos2,
    /// The positions the divider lies between, as fractions of the split: its neighbours, or the
    /// split's edges.
    pub low: f64,
    pub high: f64,
    /// Where the split starts along its axis, and its extent, in slide units.
    start: f64,
    extent: f64,
}

/// What a press in the slide view takes, per "Pointer input to gestures" of the slice's spec: the
/// first that applies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Press {
    /// Within `GRAB` points of a divider, the index of the nearest in `Layout::dividers`.
    Divider(usize),
    Area(NodeId),
    /// The letterbox, outside the slide.
    Nothing,
}

/// The shown slide as the view places it.
#[derive(Clone, Debug)]
pub struct Layout {
    /// A point of the slide at `(u, v)` slide units is at `origin + scale * (u, v)`.
    origin: Pos2,
    scale: f32,
    /// Every node, splits included, with its rectangle in points, in tree order.
    pub nodes: Vec<(NodeId, Rect)>,
    /// The areas alone, in tree order.
    areas: Vec<(NodeId, Rect)>,
    pub dividers: Vec<Divider>,
}

impl Layout {
    /// The slide `slide` of `doc` in a view of `view` points, whose texture is `pixels` in size.
    pub fn new(doc: &Document, slide: SlideId, view: Rect, pixels: [usize; 2]) -> Option<Self> {
        let the_slide = doc.slide(slide)?;
        let (w, h) = (
            u16::try_from(pixels[0]).ok()?,
            u16::try_from(pixels[1]).ok()?,
        );
        if w == 0 || h == 0 {
            return None;
        }
        let t = slide_to_pixels(doc, w, h);
        // The texture covers the view, so a pixel is this many points.
        let point = view.width() / f32::from(w);
        let origin = view.min + Vec2::new(t.x as f32, t.y as f32) * point;
        let scale = t.scale as f32 * point;
        let to_points = |r: SlideRect| {
            Rect::from_min_size(
                origin + Vec2::new(r.x as f32, r.y as f32) * scale,
                Vec2::new(r.w as f32, r.h as f32) * scale,
            )
        };
        let units = node_rects(the_slide, doc.aspect);
        let mut nodes = Vec::new();
        let mut areas = Vec::new();
        let mut dividers = Vec::new();
        for (id, r) in &units {
            let rect = to_points(*r);
            nodes.push((*id, rect));
            match the_slide.root.find(*id) {
                Some(Node::Area(_)) => areas.push((*id, rect)),
                Some(Node::Split(s)) => {
                    for index in 0..s.dividers.len() {
                        let pos = s.dividers[index];
                        let low = s.child_extent(index).0;
                        let high = s.child_extent(index + 1).1;
                        let (from, to, start, extent) = match s.axis {
                            Axis::LeftToRight => {
                                let x = rect.min.x + pos as f32 * rect.width();
                                let ends = (Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y));
                                (ends.0, ends.1, r.x, r.w)
                            }
                            Axis::TopToBottom => {
                                let y = rect.min.y + pos as f32 * rect.height();
                                let ends = (Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y));
                                (ends.0, ends.1, r.y, r.h)
                            }
                        };
                        dividers.push(Divider {
                            split: s.id,
                            index,
                            axis: s.axis,
                            from,
                            to,
                            low,
                            high,
                            start,
                            extent,
                        });
                    }
                }
                None => {}
            }
        }
        Some(Layout {
            origin,
            scale,
            nodes,
            areas,
            dividers,
        })
    }

    /// The rectangle of a node, in points.
    pub fn rect(&self, node: NodeId) -> Option<Rect> {
        self.nodes.iter().find(|(id, _)| *id == node).map(|n| n.1)
    }

    /// The area under a point, if any.
    pub fn area_at(&self, p: Pos2) -> Option<NodeId> {
        self.areas
            .iter()
            .find(|(_, r)| p.x >= r.min.x && p.x < r.max.x && p.y >= r.min.y && p.y < r.max.y)
            .map(|a| a.0)
    }

    /// The divider nearest a point, within `GRAB` points of it.
    pub fn divider_at(&self, p: Pos2) -> Option<usize> {
        self.dividers
            .iter()
            .enumerate()
            .map(|(i, d)| (i, distance(p, d.from, d.to)))
            .filter(|(_, dist)| *dist <= GRAB)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|d| d.0)
    }

    /// What a press at `p` takes: a divider before an area.
    pub fn press(&self, p: Pos2) -> Press {
        if let Some(i) = self.divider_at(p) {
            Press::Divider(i)
        } else if let Some(a) = self.area_at(p) {
            Press::Area(a)
        } else {
            Press::Nothing
        }
    }

    /// The position a drag of divider `i` to the point `p` gives it, as a fraction of its split,
    /// stopped where a child would be less than `MIN_PART` along the split's axis; `None` when the
    /// children have no room to move at all.
    pub fn drag_position(&self, i: usize, p: Pos2) -> Option<f64> {
        let d = &self.dividers[i];
        let along = match d.axis {
            Axis::LeftToRight => (p.x - self.origin.x) / self.scale,
            Axis::TopToBottom => (p.y - self.origin.y) / self.scale,
        };
        let pos = (f64::from(along) - d.start) / d.extent;
        let margin = MIN_PART / d.extent;
        let (low, high) = (d.low + margin, d.high - margin);
        (low <= high).then(|| pos.clamp(low, high))
    }
}

/// The distance in points from `p` to the segment from `a` to `b`.
fn distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = if ab.length_sq() == 0.0 {
        0.0
    } else {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
    };
    (p - (a + ab * t)).length()
}
