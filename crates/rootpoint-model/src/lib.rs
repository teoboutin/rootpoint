//! The document model of rootpoint: presentations, slides, their split trees and their content,
//! and the operations that change them.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod document;
mod file;
mod geometry;
mod ops;
mod validate;

pub use document::*;
pub use file::{FORMAT_VERSION, LoadError, OLDEST_VERSION, load_ron, save_ron};
pub use geometry::{Rect, SLIDE_HEIGHT, area_rects, node_rects};
pub use ops::{Op, OpError, Replacement, Side, apply, span_style_users};
pub use validate::{Breach, Invalid};
