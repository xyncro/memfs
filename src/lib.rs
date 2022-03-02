// Features
#![feature(map_try_insert)]
#![feature(trait_alias)]
// Lints
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![allow(missing_docs)]
#![deny(unsafe_code)]
// Lint Groups
#![warn(future_incompatible)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2021_incompatible_closure_captures)]
#![warn(rust_2021_incompatible_or_patterns)]
#![warn(rust_2021_prefixes_incompatible_syntax)]
#![warn(rust_2021_prelude_collisions)]
// Lint Groups (RustDoc)
#![deny(rustdoc::all)]

mod directory;
mod file;
mod file_system;
mod node;
mod traits;

pub use directory::{
    Directory,
    DirectoryData,
    GetDirError,
    GetError,
    GetFileError,
};
pub use file::{
    File,
    FileData,
};
pub use file_system::FileSystem;
pub use node::Node;
pub use traits::{
    Child,
    Name,
    Parent,
    Root,
};
