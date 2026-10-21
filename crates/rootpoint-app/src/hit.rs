//! Where the shown slide's nodes and dividers lie in the slide view, in points, and what a press
//! there hits. The slide is placed by `slide_to_pixels`, so every mark and every hit falls where
//! `render_slide` draws.

use eframe::egui::{Pos2, Rect, Vec2};
use rootpoint_model::{Axis, Document, Node, NodeId, Rect as SlideRect, Side, SlideId, node_rects};
use rootpoint_render::slide_to_pixels;

/// How far from a divider, in points, a press grabs it.
pub const GRAB: f32 = 4.0;

/// The least extent of a split's child along the split's axis, in slide units, that a divider drag
/// and a split keep.
pub const MIN_PART: f64 = 40.0;

/// How near a divider, in points, a line held with Alt snaps to it.
pub const SNAP: f32 = 8.0;

/// The side of the selected area's grip, and its distance inside the area's top-left corner, in
/// points: farther in than `GRAB`, so no divider's band reaches it.
pub const GRIP: f32 = 12.0;
pub const GRIP_INSET: f32 = 8.0;

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

/// What a press in the slide view takes, the first that applies in the order of the variants: the
/// grip comes before a divider, which comes before an area, since a press on the grip or a
/// divider's band starts a drag (`design@rootpoint-app@node-selection`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Press {
    /// On the grip of the selected area.
    Grip(NodeId),
    /// Within `GRAB` points of a divider, the index of the nearest in `Layout::dividers`.
    Divider(usize),
    /// Inside the selected node, Ctrl or Shift held: a split at the preview line.
    Split(Preview),
    Area(NodeId),
    /// The letterbox, outside the slide; or a split the preview refuses, whose click is ignored.
    Nothing,
}

/// A split of a node at the pointer, as its preview line shows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preview {
    pub node: NodeId,
    pub axis: Axis,
    /// The line's position, a fraction of the node along `axis`.
    pub at: f64,
    /// The new empty area takes the smaller part, `After` when the two are equal.
    pub side: Side,
    /// The ends of the line, in points.
    pub from: Pos2,
    pub to: Pos2,
}

/// An edge of an area, where a dropped area moves to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Up,
    Down,
}

impl Edge {
    /// The split that makes room on this edge: its axis and the side of the new area.
    pub fn split(self) -> (Axis, Side) {
        match self {
            Edge::Left => (Axis::LeftToRight, Side::Before),
            Edge::Right => (Axis::LeftToRight, Side::After),
            Edge::Up => (Axis::TopToBottom, Side::Before),
            Edge::Down => (Axis::TopToBottom, Side::After),
        }
    }

    /// The label of the drop hint.
    pub fn label(self) -> &'static str {
        match self {
            Edge::Left => "move left",
            Edge::Right => "move right",
            Edge::Up => "move up",
            Edge::Down => "move down",
        }
    }
}

/// What a drop of a dragged area onto another does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Swap,
    Move(Edge),
}

/// The target of a grip drag under the pointer: the area, the zone, and the zone's outline in
/// points, which the hint tints.
#[derive(Clone, Debug, PartialEq)]
pub struct Drop {
    pub target: NodeId,
    pub zone: Zone,
    pub shape: Vec<Pos2>,
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

    /// The point of the view where the slide's point `(u, v)`, in slide units, is drawn.
    pub fn to_points(&self, u: f64, v: f64) -> Pos2 {
        self.origin + Vec2::new(u as f32, v as f32) * self.scale
    }

    /// The point of the slide, in slide units, drawn at the view's point `p`.
    pub fn to_units(&self, p: Pos2) -> (f64, f64) {
        let d = (p - self.origin) / self.scale;
        (f64::from(d.x), f64::from(d.y))
    }

    /// How many points one slide unit is.
    pub fn points_per_unit(&self) -> f32 {
        self.scale
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

    /// What a press at `p` takes, the first that applies: the grip of the selected area, a
    /// divider, a split of the selected node when `split` names an axis (Ctrl or Shift held), an
    /// area, or nothing.
    pub fn press(&self, p: Pos2, selected: Option<NodeId>, split: Option<(Axis, bool)>) -> Press {
        if let Some(area) = selected
            && self.grip(area).is_some_and(|g| g.contains(p))
        {
            return Press::Grip(area);
        }
        if let Some(i) = self.divider_at(p) {
            return Press::Divider(i);
        }
        if let (Some(node), Some((axis, snap))) = (selected, split)
            && self.rect(node).is_some_and(|r| r.contains(p))
        {
            return match self.preview(node, axis, p, snap) {
                Some(preview) => Press::Split(preview),
                None => Press::Nothing,
            };
        }
        match self.area_at(p) {
            Some(a) => Press::Area(a),
            None => Press::Nothing,
        }
    }

    /// The grip of an area: a square of `GRIP` points, `GRIP_INSET` points inside its top-left
    /// corner. `None` for a split, or an area too small to hold it.
    pub fn grip(&self, area: NodeId) -> Option<Rect> {
        let r = self.areas.iter().find(|(id, _)| *id == area)?.1;
        let grip = Rect::from_min_size(r.min + Vec2::splat(GRIP_INSET), Vec2::splat(GRIP));
        r.contains_rect(grip).then_some(grip)
    }

    /// The split of `node` along `axis` at the pointer `p`, snapped with Alt; `None` where a part
    /// would be less than `MIN_PART` slide units.
    pub fn preview(&self, node: NodeId, axis: Axis, p: Pos2, snap: bool) -> Option<Preview> {
        let r = self.rect(node)?;
        let (start, len, along) = match axis {
            Axis::LeftToRight => (r.min.x, r.width(), p.x),
            Axis::TopToBottom => (r.min.y, r.height(), p.y),
        };
        let along = if snap {
            self.snap(along, axis, start, len, None)
        } else {
            along
        };
        let at = f64::from((along - start) / len);
        let units = f64::from(len / self.scale);
        if at * units < MIN_PART || (1.0 - at) * units < MIN_PART {
            return None;
        }
        let (from, to) = match axis {
            Axis::LeftToRight => (Pos2::new(along, r.min.y), Pos2::new(along, r.max.y)),
            Axis::TopToBottom => (Pos2::new(r.min.x, along), Pos2::new(r.max.x, along)),
        };
        Some(Preview {
            node,
            axis,
            at,
            side: if at < 0.5 { Side::Before } else { Side::After },
            from,
            to,
        })
    }

    /// A coordinate along `axis`, in points, snapped to the nearest twelfth of the extent from
    /// `start` of `len` points, or to a divider of the slide across that axis within `SNAP`
    /// points, the nearer of the two; `exclude` names a divider that is not a target, the one
    /// being dragged.
    pub fn snap(
        &self,
        along: f32,
        axis: Axis,
        start: f32,
        len: f32,
        exclude: Option<(NodeId, usize)>,
    ) -> f32 {
        let twelfth = start + ((along - start) / len * 12.0).round() / 12.0 * len;
        let divider = self
            .dividers
            .iter()
            .filter(|d| d.axis == axis && Some((d.split, d.index)) != exclude)
            .map(|d| match axis {
                Axis::LeftToRight => d.from.x,
                Axis::TopToBottom => d.from.y,
            })
            .filter(|c| (c - along).abs() <= SNAP)
            .min_by(|a, b| (a - along).abs().total_cmp(&(b - along).abs()));
        match divider {
            Some(c) if (c - along).abs() < (twelfth - along).abs() => c,
            _ => twelfth,
        }
    }

    /// The position a drag of divider `i` to the point `p` gives it, as a fraction of its split,
    /// snapped with Alt as the preview line is, and stopped where a child would be less than
    /// `MIN_PART` along the split's axis; `None` when the children have no room to move at all.
    pub fn drag_position(&self, i: usize, p: Pos2, snap: bool) -> Option<f64> {
        let d = &self.dividers[i];
        let (along, origin) = match d.axis {
            Axis::LeftToRight => (p.x, self.origin.x),
            Axis::TopToBottom => (p.y, self.origin.y),
        };
        let along = if snap {
            let start = origin + d.start as f32 * self.scale;
            let len = d.extent as f32 * self.scale;
            self.snap(along, d.axis, start, len, Some((d.split, d.index)))
        } else {
            along
        };
        let pos = (f64::from((along - origin) / self.scale) - d.start) / d.extent;
        let margin = MIN_PART / d.extent;
        let (low, high) = (d.low + margin, d.high - margin);
        (low <= high).then(|| pos.clamp(low, high))
    }

    /// Where a drop of `dragged` at `p` goes: the area under the pointer and its zone. The middle
    /// third of the area, by width and by height, swaps; the rest moves to the edge nearest the
    /// pointer, the distances taken as fractions of the area's width and height, left and right
    /// winning a tie. `None` outside any area, on `dragged` itself, and on a move zone whose split
    /// of the target at its middle would leave a part under `MIN_PART` slide units, the bound a
    /// split keeps.
    pub fn drop(&self, p: Pos2, dragged: NodeId) -> Option<Drop> {
        let target = self.area_at(p).filter(|a| *a != dragged)?;
        let r = self.rect(target)?;
        let fx = (p.x - r.min.x) / r.width();
        let fy = (p.y - r.min.y) / r.height();
        let third = 1.0 / 3.0;
        let middle = |f: f32| (third..=2.0 * third).contains(&f);
        let at = |x: f32, y: f32| Pos2::new(r.min.x + x * r.width(), r.min.y + y * r.height());
        let (zone, shape) = if middle(fx) && middle(fy) {
            let (a, b) = (third, 2.0 * third);
            (Zone::Swap, vec![at(a, a), at(b, a), at(b, b), at(a, b)])
        } else {
            let (a, b) = (third, 2.0 * third);
            let edge = if fx.min(1.0 - fx) <= fy.min(1.0 - fy) {
                if fx <= 1.0 - fx {
                    Edge::Left
                } else {
                    Edge::Right
                }
            } else if fy <= 1.0 - fy {
                Edge::Up
            } else {
                Edge::Down
            };
            let extent = match edge {
                Edge::Left | Edge::Right => r.width(),
                Edge::Up | Edge::Down => r.height(),
            };
            if f64::from(extent / self.scale) * 0.5 < MIN_PART {
                return None;
            }
            // The part of the area nearer this edge than any other, less the middle third.
            let shape = match edge {
                Edge::Left => vec![at(0.0, 0.0), at(a, a), at(a, b), at(0.0, 1.0)],
                Edge::Right => vec![at(1.0, 0.0), at(1.0, 1.0), at(b, b), at(b, a)],
                Edge::Up => vec![at(0.0, 0.0), at(1.0, 0.0), at(b, a), at(a, a)],
                Edge::Down => vec![at(0.0, 1.0), at(a, b), at(b, b), at(1.0, 1.0)],
            };
            (Zone::Move(edge), shape)
        };
        Some(Drop {
            target,
            zone,
            shape,
        })
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
