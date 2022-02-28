#![deny(unsafe_code)]
#![feature(trait_alias)]

mod directory;
mod error;
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
};
pub use error::{
    GetError,
    GetPathError,
};
pub use file::{
    File,
    FileData,
};
pub use file_system::FileSystem;
pub use node::Node;
