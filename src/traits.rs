use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    Directory,
    DirectoryData,
    FileData,
    Node,
};

#[async_trait]
pub trait Name {
    async fn name(&self) -> Option<String>;
}

#[async_trait]
pub trait Child<D, F> {
    async fn parent(&self) -> Option<Directory<D, F>>
    where
        D: DirectoryData,
        F: FileData;
}

#[async_trait]
pub trait Parent<D, F> {
    async fn children(&self) -> &HashMap<String, Node<D, F>>
    where
        D: DirectoryData,
        F: FileData;
}

#[async_trait]
pub trait Root<D, F> {
    async fn is_root(&self) -> bool
    where
        D: DirectoryData,
        F: FileData;
}
