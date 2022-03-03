use std::sync::Arc;

use async_lock::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
use async_trait::async_trait;
use futures::FutureExt;

use crate::{
    directory::Reference,
    Child,
    Data,
    Directory,
    Name,
    Value,
    ValueType,
};

// =============================================================================
// File
// =============================================================================

#[derive(Debug)]
pub struct File<D, F>(pub(crate) Arc<RwLock<Internal<D, F>>>)
where
    D: ValueType,
    F: ValueType;

// -----------------------------------------------------------------------------
// File - Traits
// -----------------------------------------------------------------------------

impl<D, F> Clone for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl<D, F> Child<D, F> for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        self.read(|file| file.parent.1.clone())
            .map(|Reference(parent)| parent.upgrade())
            .map(|parent| parent.map(Directory))
            .await
    }
}

#[async_trait]
impl<D, F> Data<F> for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn data(&self) -> Value<F> {
        self.read(|file| file.value.clone()).await
    }
}

#[async_trait]
impl<D, F> Name for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn name(&self) -> Option<String> {
        self.read(|file| Some(file.parent.0.clone())).await
    }
}

// -----------------------------------------------------------------------------
// File - Methods
// -----------------------------------------------------------------------------

// File - Methods - Read & Write

impl<D, F> File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn read<T, R>(&self, f: R) -> T
    where
        R: FnOnce(RwLockReadGuard<'_, Internal<D, F>>) -> T + Send,
    {
        self.0.read().map(f).await
    }

    #[allow(dead_code)] // TODO: Remove when used
    async fn write<T, W>(&self, f: W) -> T
    where
        W: FnOnce(RwLockWriteGuard<'_, Internal<D, F>>) -> T + Send,
    {
        self.0.write().map(f).await
    }
}

// File - Methods - Create

impl<D, F> File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    #[must_use]
    pub(crate) fn create(value: Option<F>, parent: (String, Reference<D, F>)) -> Self {
        Self(Arc::new(RwLock::new(Internal {
            value: Value::from_option(value),
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
    D: ValueType,
    F: ValueType,
{
    parent: (String, Reference<D, F>),
    value: Value<F>,
}
