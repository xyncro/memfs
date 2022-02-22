use crate::{
    directory::DirectoryData,
    file::FileData,
    Directory,
};

#[derive(Debug)]
pub struct FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    root: Directory<D, F>,
}

// Create

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

// Root

impl<D, F> FileSystem<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn root(&self) -> Directory<D, F> {
        self.root.clone()
    }
}
