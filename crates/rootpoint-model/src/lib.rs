//! The document model of rootpoint: presentations, slides, their split trees and their content.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

/// A presentation: its slides and its assets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {}
