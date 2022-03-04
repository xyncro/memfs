#![feature(
    derive_default_enum,
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

mod internal;

pub use internal::{
    directory::Directory,
    file::File,
    file_system::FileSystem,
    node::Node,
};

pub mod directory {
    pub use super::internal::directory::{
        count::Count,
        get::{
            Get,
            GetError,
            GetType,
        },
        get_ext::{
            GetDirectoryError,
            GetExt,
            GetFileError,
        },
    };
}

pub mod node {
    pub use super::internal::node::{
        child::Child,
        data::{
            Data,
            Value,
            ValueType,
        },
        data_ext::DataExt,
        located::Located,
        named::Named,
        root::Root,
    };
}
