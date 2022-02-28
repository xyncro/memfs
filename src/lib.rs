#![deny(unsafe_code)]
#![feature(trait_alias)]

mod directory;
mod error;
mod file;
mod file_system;
mod node;

use async_trait::async_trait;

// -----------------------------------------------------------------------------

// Traits

#[async_trait]
pub trait Named {
    async fn named(&self) -> Option<String>;
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
