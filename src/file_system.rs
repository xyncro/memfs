use std::ops::Deref;

use crate::{
    Directory,
    DirectoryData,
    FileData,
};

// =============================================================================

// FileSystem

#[derive(Debug)]
pub struct FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    root: Directory<D, F>,
}

// -----------------------------------------------------------------------------

// FileSystem - Trait Implementations

impl<D, F> Deref for FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    type Target = Directory<D, F>;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

// -----------------------------------------------------------------------------

// FileSystem - Create

impl<D, F> FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn new() -> Self {
        Self {
            root: Directory::create_root(),
        }
    }
}

// -----------------------------------------------------------------------------

// FileSystem - Root

impl<D, F> FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn root(&self) -> Directory<D, F> {
        self.root.clone()
    }
}
