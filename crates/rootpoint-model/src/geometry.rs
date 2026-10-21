//! Where each area of a slide lies, in slide units.

use crate::document::{AspectRatio, Axis, Node, NodeId, Slide};

/// The height of every slide, in slide units. A slide's width follows from its aspect ratio, so a
/// 16:9 slide is 1920×1080 and a font size of 48 means what it would on a 1080p screen.
pub const SLIDE_HEIGHT: f64 = 1080.0;

/// A rectangle in slide units: its top-left corner, its width and its height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl AspectRatio {
    /// The whole slide, in slide units.
    pub fn slide_rect(self) -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: SLIDE_HEIGHT * f64::from(self.width) / f64::from(self.height),
            h: SLIDE_HEIGHT,
        }
    }
}

/// Every area of the slide with its rectangle, in tree order.
pub fn area_rects(slide: &Slide, aspect: AspectRatio) -> Vec<(NodeId, Rect)> {
    let mut out = Vec::new();
    collect(&slide.root, aspect.slide_rect(), &mut out);
    out
}

fn collect(node: &Node, rect: Rect, out: &mut Vec<(NodeId, Rect)>) {
    match node {
        Node::Area(a) => out.push((a.id, rect)),
        Node::Split(s) => {
            for (i, child) in s.children.iter().enumerate() {
                let (start, end) = s.child_extent(i);
                let part = match s.axis {
                    // Children side by side, left to right.
                    Axis::LeftToRight => Rect {
                        x: rect.x + start * rect.w,
                        w: (end - start) * rect.w,
                        ..rect
                    },
                    // Children stacked, top to bottom.
                    Axis::TopToBottom => Rect {
                        y: rect.y + start * rect.h,
                        h: (end - start) * rect.h,
                        ..rect
                    },
                };
                collect(child, part, out);
            }
        }
    }
}
