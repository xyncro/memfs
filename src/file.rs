use std::sync::Arc;

use async_trait::async_trait;
use futures_util::FutureExt;
use tokio::sync::{
    RwLock as Lock,
    RwLockReadGuard as Read,
};

use crate::{
    directory::DirectoryWeak,
    DirectoryData,
    Named,
};

// =============================================================================

// FileData

pub trait FileData = Default + Send + Sync;

// =============================================================================

// File

#[derive(Debug)]
pub struct File<D, F>(Arc<Lock<FileInternal<D, F>>>)
where
    D: DirectoryData,
    F: FileData;

// -----------------------------------------------------------------------------

// File - Trait Implementations

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
        self.read_lock(|file| Some(file.parent.0.clone())).await
    }
}

// -----------------------------------------------------------------------------

// File - Read/Write (Internal)

impl<D, F> File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn read_lock<T>(&self, f: impl FnOnce(Read<FileInternal<D, F>>) -> T) -> T {
        self.0.read().map(f).await
    }
}

// =============================================================================

// FileInternal

#[derive(Debug)]
pub struct FileInternal<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    _data: F,
    parent: (String, DirectoryWeak<D, F>),
}
