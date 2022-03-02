//! ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/xyncro/memfs/ci?label=build%20%26%20test&style=flat-square)
//! ![Docs.rs Documentation](https://img.shields.io/docsrs/memfs?style=flat-square)
//! ![GitHub Issues](https://img.shields.io/github/issues/xyncro/memfs?style=flat-square)
//! ![Crates.io Lience](https://img.shields.io/crates/l/memfs?style=flat-square)
//! ![Crates.io Version](https://img.shields.io/crates/v/memfs?label=crate&style=flat-square)
//!
//! An in-memory, async, filesystem-like data structure, with parameterised
//! directory and file data nodes.
//! 
//! # Example
//! 
//! ```rust
//! # use anyhow::Result;
//! use memfs::*;
//! 
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! // Both directories and files will hold data of type u32
//! let fs: FileSystem<u32, u32> = FileSystem::new();
//! 
//! // Using the `get_file_default` method will ensure that any missing
//! // directories in the path, and the file if missing, will be created with
//! // default data values. Because of this, the return value is always a
//! // concrete value inside the `Result`, and not an `Option`. 
//! let file_1 = fs.get_file_default("/dir_a/dir_b/file_1.txt").await?;
//! 
//! // The newly created directories can also be retrieved, and used to
//! // interact with files, etc.
//! let dir_b = fs.get_dir_default("/dir_a/dir_b").await?;
//! 
//! // Files and directories can be obtained using relative paths.
//! let file_1 = dir_b.get_file_default("file_1.txt").await?;
//! 
//! # Ok(())
//! # }
//! ```
//! 
//! # Details
//! 
//! * MemFS has been designed for use as a simple filesystem-like abstraction
//! when a tree of path-addressable data is needed. It is not intended to be a
//! general abstraction over filesystems in general, or to implement any
//! abstractions which would allow for such.
//! * MemFS does not mimic or mirror Posix APIs or similar, and does not mimic
//! the `std::fs` implementation in the Rust Standard Library. These APIs are
//! tried and tested in a very broad context, but this context does not apply to
//! MemFS and a different approach has been adopted.
//! * MemFS deals in `Directory`s and `File`s, both of which hold data through
//! a generic (parameterised) property. A `FileSystem` is therefore
//! parameterised by the types of data held by both `Directory`s and `File`s,
//! providing strongly-typed guarantees of the directory and file content.

#![deny(unsafe_code)]
#![feature(map_try_insert)]
#![feature(trait_alias)]

mod directory;
mod file;
mod file_system;
mod node;

use std::collections::HashMap;

use async_trait::async_trait;

// -----------------------------------------------------------------------------

// Traits

#[async_trait]
pub trait Name {
    async fn name(&self) -> Option<String>;
}

#[async_trait]
pub trait Child<D, F> {
    async fn parent(&self) -> Option<Directory<D, F>>
    where
        D: DirectoryData,
        F: FileData;
}

#[async_trait]
pub trait Parent<D, F> {
    async fn children(&self) -> &HashMap<String, Node<D, F>>
    where
        D: DirectoryData,
        F: FileData;
}

#[async_trait]
pub trait Root<D, F> {
    async fn is_root(&self) -> bool
    where
        D: DirectoryData,
        F: FileData;
}

// -----------------------------------------------------------------------------

// Public Exports

pub use directory::{
    Directory,
    DirectoryData,
    GetDirError,
    GetFileError,
    GetError
};
pub use file::{
    File,
    FileData,
};
pub use file_system::FileSystem;
pub use node::Node;
