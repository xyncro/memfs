use std::ops::Deref;

use crate::{
    Directory,
    DirectoryData,
    FileData,
};

// =============================================================================
// FileSystem
// =============================================================================

#[derive(Debug)]
pub struct FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    root: Directory<D, F>,
}

// -----------------------------------------------------------------------------
// FileSystem - Traits
// -----------------------------------------------------------------------------

impl<D, F> Default for FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    fn default() -> Self {
        Self::new()
    }
}

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
// FileSystem - Methods
// -----------------------------------------------------------------------------

// FileSystem - Methods - New

impl<D, F> FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Directory::create_root(),
        }
    }
}
