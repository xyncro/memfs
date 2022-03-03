#![feature(
    map_try_insert,
    trait_alias
)]
#![deny(
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    unsafe_code,
    unused,
    warnings
)]
#![deny(
    rust_2018_compatibility,
    rust_2018_idioms
)]
#![deny(
    rust_2021_compatibility,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions
)]
#![deny(
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic
)]
#![allow( // TODO
    clippy::missing_errors_doc,
    missing_docs,
    rustdoc::all
)]

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
