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
    directory::Reference,
    Child,
    Directory,
    DirectoryData,
    Name,
};

// =============================================================================
// File
// =============================================================================

#[derive(Debug)]
pub struct File<D, F>(pub(crate) Arc<Lock<Internal<D, F>>>)
where
    D: DirectoryData,
    F: Data;

// -----------------------------------------------------------------------------
// File - Traits
// -----------------------------------------------------------------------------

impl<D, F> Clone for File<D, F>
where
    D: DirectoryData,
    F: Data,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl<D, F> Name for File<D, F>
where
    D: DirectoryData,
    F: Data,
{
    async fn name(&self) -> Option<String> {
        self.read(|file| Some(file.parent.0.clone())).await
    }
}

#[async_trait]
impl<D, F> Child<D, F> for File<D, F>
where
    D: DirectoryData,
    F: Data,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        self.read(|file| file.parent.1.clone())
            .map(|Reference(parent)| parent.upgrade())
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
    F: Data,
{
    async fn read<T, R>(&self, f: R) -> T
    where
        R: FnOnce(Read<'_, Internal<D, F>>) -> T + Send,
    {
        self.0.read().map(f).await
    }

    #[allow(dead_code)] // TODO: Remove when used
    async fn write<T, W>(&self, f: W) -> T
    where
        W: FnOnce(Write<'_, Internal<D, F>>) -> T + Send,
    {
        self.0.write().map(f).await
    }
}

// File - Methods - Create

impl<D, F> File<D, F>
where
    D: DirectoryData,
    F: Data,
{
    pub(crate) fn create(data: Option<F>, parent: (String, Reference<D, F>)) -> Self {
        Self(Arc::new(Lock::new(Internal {
            _data: data.unwrap_or_default(),
            parent,
        })))
    }
}

// =============================================================================
// Internal
// =============================================================================

#[derive(Debug)]
pub struct Internal<D, F>
where
    D: DirectoryData,
    F: Data,
{
    _data: F,
    parent: (String, Reference<D, F>),
}

// =============================================================================
// Data
// =============================================================================

pub trait Data = Default + Send + Sync;
