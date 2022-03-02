use async_trait::async_trait;

use crate::{
    directory::DirectoryData,
    file::FileData,
    Child,
    Directory,
    File,
    Root,
};

// =============================================================================
// Node
// =============================================================================

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
// Node - Traits
// -----------------------------------------------------------------------------

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

#[async_trait]
impl<D, F> Child<D, F> for Node<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        match self {
            Self::Directory(dir) => dir.parent().await,
            Self::File(file) => file.parent().await,
        }
    }
}

#[async_trait]
impl<D, F> Root<D, F> for Node<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn is_root(&self) -> bool {
        match self {
            Self::Directory(dir) => dir.is_root().await,
            _ => false,
        }
    }
}
