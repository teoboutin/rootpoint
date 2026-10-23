//! The one-line form of a slide's tree that `tree` replies with.

use rootpoint_model::{Axis, Content, ImageFit, Node, Slide};

use crate::command::quote_always;

/// The slide as `s<n> <node>`: a split as `n<n>:<axis>(<dividers>)[<child> | ...]`, an area as
/// `n<n>` and its content.
pub(crate) fn slide_tree(slide: &Slide) -> String {
    format!("{} {}", slide.id, node_tree(&slide.root))
}

fn node_tree(node: &Node) -> String {
    match node {
        Node::Split(s) => {
            let axis = match s.axis {
                Axis::LeftToRight => "beside",
                Axis::TopToBottom => "stacked",
            };
            let dividers: Vec<String> = s.dividers.iter().map(|d| d.to_string()).collect();
            let children: Vec<String> = s.children.iter().map(node_tree).collect();
            format!(
                "{}:{axis}({})[{}]",
                s.id,
                dividers.join(","),
                children.join(" | ")
            )
        }
        Node::Area(a) => match &a.content {
            Content::Text(t) => format!("{} text {}", a.id, quote_always(&t.text)),
            Content::Image(i) => {
                let fit = match i.fit {
                    ImageFit::Fit => "fit",
                    ImageFit::Fill => "fill",
                };
                format!("{} image {} {fit}", a.id, i.asset)
            }
        },
    }
}
