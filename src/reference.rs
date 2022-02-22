use crate::{
    directory::{
        DirectoryData,
        DirectoryWeak,
    },
    file::FileData,
    Directory,
    File,
};

// =============================================================================

// ChildRef

#[derive(Debug)]
pub enum ChildRef<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    Directory(Directory<D, F>),
    File(File<D, F>),
}

// -----------------------------------------------------------------------------

// ChildRef - Trait Implementations

impl<D, F> Clone for ChildRef<D, F>
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

// =============================================================================

// ParentRef

#[derive(Debug)]
pub enum ParentRef<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    Directory(String, DirectoryWeak<D, F>),
}

// -----------------------------------------------------------------------------

// ParentRef - Trait Implementations

impl<D, F> Clone for ParentRef<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    fn clone(&self) -> Self {
        match self {
            Self::Directory(name, dir) => Self::Directory(name.clone(), dir.clone()),
        }
    }
}

// -----------------------------------------------------------------------------

// ParentRef - Name

impl<D, F> ParentRef<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn name(&self) -> String {
        match self {
            Self::Directory(name, _) => name.clone(),
        }
    }
}
