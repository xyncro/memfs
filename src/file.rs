use std::sync::Arc;

use async_trait::async_trait;
use futures_util::FutureExt;
use tokio::sync::{
    RwLock as Lock,
    RwLockReadGuard as Read,
    RwLockWriteGuard as Write,
};

#[cfg(doc)]
use crate::FileSystem;
use crate::{
    directory::DirectoryWeak,
    Child,
    Directory,
    DirectoryData,
    Name,
};

// =============================================================================
// File
// =============================================================================

#[derive(Debug)]
pub struct File<D, F>(pub(crate) Arc<Lock<FileInternal<D, F>>>)
where
    D: DirectoryData,
    F: FileData;

// -----------------------------------------------------------------------------
// File - Traits
// -----------------------------------------------------------------------------

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
impl<D, F> Name for File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn name(&self) -> Option<String> {
        self.read(|file| Some(file.parent.0.clone())).await
    }
}

#[async_trait]
impl<D, F> Child<D, F> for File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        self.read(|file| file.parent.1.clone())
            .map(|DirectoryWeak(parent)| parent.upgrade())
            .map(|parent| parent.map(Directory))
            .await
    }
}

// -----------------------------------------------------------------------------
// File - Methods
// -----------------------------------------------------------------------------

// File - Methods - Read & Write

impl<D, F> File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn read<T>(&self, f: impl FnOnce(Read<'_, FileInternal<D, F>>) -> T) -> T {
        self.0.read().map(f).await
    }

    #[allow(dead_code)] // TODO: Remove when used
    async fn write<T>(&self, f: impl FnOnce(Write<'_, FileInternal<D, F>>) -> T) -> T {
        self.0.write().map(f).await
    }
}

// File - Methods - Create

impl<D, F> File<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub(crate) fn create(data: Option<F>, parent: (String, DirectoryWeak<D, F>)) -> Self {
        Self(Arc::new(Lock::new(FileInternal {
            _data: data.unwrap_or_default(),
            parent,
        })))
    }
}

// =============================================================================
// FileInternal
// =============================================================================

#[derive(Debug)]
pub(crate) struct FileInternal<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    _data: F,
    parent: (String, DirectoryWeak<D, F>),
}

// =============================================================================
// FileData
// =============================================================================

pub trait FileData = Default + Send + Sync;
