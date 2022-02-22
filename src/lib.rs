#![deny(unsafe_code)]
#![feature(trait_alias)]

mod directory;
mod error;
mod file;
mod file_system;
mod reference;

use async_trait::async_trait;

// -----------------------------------------------------------------------------

// Traits

#[async_trait]
pub trait Named {
    async fn named(&self) -> Option<String>;
}

// -----------------------------------------------------------------------------

// Public Exports

pub use directory::Directory;
pub use file::File;
pub use file_system::FileSystem;
