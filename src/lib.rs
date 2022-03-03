// Features
#![feature(map_try_insert)]
#![feature(trait_alias)]
// Lints
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![allow(missing_docs)] // TODO
#![deny(unsafe_code)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(rust_2021_incompatible_closure_captures)]
#![deny(rust_2021_incompatible_or_patterns)]
#![deny(rust_2021_prefixes_incompatible_syntax)]
#![deny(rust_2021_prelude_collisions)]
// Lints (Clippy)
#![deny(clippy::cargo)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // TODO
// Lints (RustDoc)
#![allow(rustdoc::all)] // TODO

mod directory;
mod file;
mod file_system;
mod node;
mod traits;

pub use directory::{
    Data as DirectoryData,
    Directory,
    GetDirError,
    GetError,
    GetFileError,
};
pub use file::{
    Data as FileData,
    File,
};
pub use file_system::FileSystem;
pub use node::Node;
pub use traits::{
    Child,
    Name,
    Parent,
    Root,
};
