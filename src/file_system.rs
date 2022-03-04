use std::ops::Deref;

use crate::{
    Directory,
    ValueType,
};

// FileSystem

#[derive(Debug)]
pub struct FileSystem<D, F>(pub(crate) Directory<D, F>)
where
    D: ValueType,
    F: ValueType;

// FileSystem - Standard Traits

impl<D, F> Default for FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D, F> Deref for FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    type Target = Directory<D, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// FileSystem - Methods

impl<D, F> FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    #[must_use]
    pub fn new() -> Self {
        Self(Directory::create_root())
    }
}
