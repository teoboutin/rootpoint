//! The one-line form of a slide's tree that `tree` replies with.

use std::collections::BTreeMap;

use rootpoint_model::{
    Axis, Content, ImageFit, Node, NodeId, Slide, SpanChoice, StyleChoice, TextItem, TextNode,
};

use crate::command::{EquationOption, SpanOption, TreeItem, TreeSpan, quote_always, tree_text};

/// The slide as `s<n> <node>`: a split as `n<n>:<axis>(<dividers>)[<child> | ...]`, an area as
/// `n<n>` and its content, a text as its tree in the grammar `text` takes, then its style,
/// `style="<name>"` or `custom`, an equation as `equation` takes it, all its options, then its
/// number, `(<n>)`, when it is numbered.
pub(crate) fn slide_tree(slide: &Slide, numbers: &BTreeMap<NodeId, u32>) -> String {
    format!("{} {}", slide.id, node_tree(&slide.root, numbers))
}

fn node_tree(node: &Node, numbers: &BTreeMap<NodeId, u32>) -> String {
    match node {
        Node::Split(s) => {
            let axis = match s.axis {
                Axis::LeftToRight => "beside",
                Axis::TopToBottom => "stacked",
            };
            let dividers: Vec<String> = s.dividers.iter().map(|d| d.to_string()).collect();
            let children: Vec<String> = s.children.iter().map(|c| node_tree(c, numbers)).collect();
            format!(
                "{}:{axis}({})[{}]",
                s.id,
                dividers.join(","),
                children.join(" | ")
            )
        }
        Node::Area(a) => match &a.content {
            Content::Text(t) => {
                let style = match &t.style {
                    StyleChoice::Named(name) => format!("style={}", quote_always(name)),
                    StyleChoice::Custom(_) => "custom".to_owned(),
                };
                // The tree's items are separated by commas, which never meet the ` | ` that
                // separates a split's children, so a slide's line reads as one.
                format!("{} text {} {style}", a.id, tree_text(&tree_items(&t.root)))
            }
            Content::Image(i) => {
                let fit = match i.fit {
                    ImageFit::Fit => "fit",
                    ImageFit::Fill => "fill",
                };
                format!("{} image {} {fit}", a.id, i.asset)
            }
            Content::Equation(e) => {
                let options = [
                    EquationOption::Numbered(e.numbered),
                    EquationOption::Size(e.size),
                    EquationOption::Colour(e.colour),
                ]
                .map(|o| o.to_string())
                .join(" ");
                let mut line = format!("{} equation ${}$ {options}", a.id, e.formula.to_latex());
                if let Some(n) = numbers.get(&a.id) {
                    line.push_str(&format!(" ({n})"));
                }
                line
            }
        },
    }
}

/// A text node as the items of a tree, each span with its id and its style: `style="<name>"` for a
/// named one, the values it sets for a custom one. A built-in family the binary does not carry,
/// which only a file edited by hand holds, has no option, so it is left out.
pub(crate) fn tree_items(node: &TextNode) -> Vec<TreeItem> {
    node.items
        .iter()
        .map(|item| match item {
            TextItem::Chars(text) => TreeItem::Chars(text.clone()),
            TextItem::Span(span) => TreeItem::Span(TreeSpan {
                id: Some(span.id),
                style: match &span.style {
                    SpanChoice::Named(name) => vec![SpanOption::Named(name.clone())],
                    SpanChoice::Custom(values) => SpanOption::values_of(values),
                },
                items: tree_items(&span.text),
            }),
        })
        .collect()
}
