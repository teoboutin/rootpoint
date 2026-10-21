//! The document model of rootpoint: presentations, slides, their split trees and their content,
//! and the operations that change them.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod document;
mod geometry;
mod ops;

pub use document::*;
pub use geometry::{Rect, SLIDE_HEIGHT, area_rects};
pub use ops::{Op, OpError, Side, apply};
