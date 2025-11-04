//! Core traits and types for the form_factor framework
//!
//! This crate defines the foundational traits (App, Backend, AppContext)
//! that other crates build upon. It has minimal dependencies.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod app;
mod backend;
mod error;

pub use app::{App, AppContext};
pub use backend::{Backend, BackendConfig};
pub use error::{IoError, IoOperation};
