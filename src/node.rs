use crate::{
    directory::DirectoryData,
    file::FileData,
    Directory,
    File,
};

// =============================================================================

// Node

#[derive(Debug)]
pub enum Node<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    Directory(Directory<D, F>),
    File(File<D, F>),
}

// -----------------------------------------------------------------------------

// Node - Trait Implementations

impl<D, F> Clone for Node<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    fn clone(&self) -> Self {
        match &self {
            Self::Directory(dir) => Self::Directory(dir.clone()),
            Self::File(file) => Self::File(file.clone()),
        }
    }
}
