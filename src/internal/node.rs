pub mod child;
pub mod data;
pub mod data_ext;
pub mod named;

use async_trait::async_trait;

use self::{
    child::Child,
    data::ValueType,
    named::Named,
};
use super::{
    directory::{
        root::Root,
        Directory,
    },
    file::File,
};

// Node

#[derive(Debug)]
pub enum Node<D, F>
where
    D: ValueType,
    F: ValueType,
{
    Directory(Directory<D, F>),
    File(File<D, F>),
}

// Node - Standard Traits

impl<D, F> Clone for Node<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn clone(&self) -> Self {
        match &self {
            Self::Directory(dir) => Self::Directory(dir.clone()),
            Self::File(file) => Self::File(file.clone()),
        }
    }
}

// Node - Library Traits

#[async_trait]
impl<D, F> Child<D, F> for Node<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        match self {
            Self::Directory(dir) => dir.parent().await,
            Self::File(file) => file.parent().await,
        }
    }
}

#[async_trait]
impl<D, F> Named for Node<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn name(&self) -> Option<String> {
        match self {
            Self::Directory(dir) => dir.name().await,
            Self::File(file) => file.name().await,
        }
    }
}

#[async_trait]
impl<D, F> Root<D, F> for Node<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn is_root(&self) -> bool {
        match self {
            Self::Directory(dir) => dir.is_root().await,
            Self::File(_) => false,
        }
    }
}
