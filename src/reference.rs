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

#[derive(Clone, Debug)]
pub struct ParentRef<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    name: String,
    _parent: DirectoryWeak<D, F>,
}

// -----------------------------------------------------------------------------

// ParentRef - Create

impl<D, F> ParentRef<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn create(name: String, parent: DirectoryWeak<D, F>) -> Self {
        Self {
            _parent: parent,
            name,
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
        self.name.clone()
    }
}
