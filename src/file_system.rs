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
