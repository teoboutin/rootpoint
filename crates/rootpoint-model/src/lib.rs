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
pub use file::{FORMAT_VERSION, LoadError, load_ron, save_ron};
pub use geometry::{Rect, SLIDE_HEIGHT, area_rects};
pub use ops::{Op, OpError, Side, apply};
pub use validate::Invalid;
