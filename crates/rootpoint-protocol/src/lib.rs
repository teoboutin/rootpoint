//! The text protocol of rootpoint: commands, one per line, that drive a session holding one
//! document, its undo and redo stacks and its view state. The headless binary runs it today, and
//! the editor will.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod command;
mod files;
mod session;
mod tree;

pub use command::{Command, ParseError, StyleOption, parse_command};
pub use files::Files;
pub use session::{Reply, Session, View};
