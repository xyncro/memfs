use std::sync::Arc;

use async_trait::async_trait;
use futures_util::FutureExt;
use tokio::sync::{
    RwLock,
    RwLockReadGuard,
};

use crate::{
    directory::DirectoryData,
    reference::ParentRef,
    Named,
};

pub trait FileData = Default + Send + Sync;

#[derive(Debug)]
pub struct File<D, F>(Arc<RwLock<FileInternal<D, F>>>)
where
    D: DirectoryData,
    F: FileData;

impl<D, F> Clone for File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl<D, F> Named for File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn named(&self) -> Option<String> {
        self.read_lock(|d| Some(d.parent.name())).await
    }
}

impl<D, F> File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn read_lock<T>(&self, f: impl FnOnce(RwLockReadGuard<FileInternal<D, F>>) -> T) -> T {
        self.0.read().map(f).await
    }
}

#[derive(Debug)]
pub struct FileInternal<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    _data: F,
    parent: ParentRef<D, F>,
}
