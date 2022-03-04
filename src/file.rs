use std::{
    ops::Deref,
    sync::Arc,
};

use async_lock::RwLock;
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

impl<D, F> Deref for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    type Target = Arc<RwLock<Internal<D, F>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<D, F> Child<D, F> for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        self.read()
            .map(|this| this.parent.1.clone())
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
        self.read().map(|this| this.value.clone()).await
    }
}

#[async_trait]
impl<D, F> Name for File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn name(&self) -> Option<String> {
        self.read().map(|this| Some(this.parent.0.clone())).await
    }
}

// -----------------------------------------------------------------------------
// File - Methods
// -----------------------------------------------------------------------------

// File - Methods - Create

impl<D, F> File<D, F>
where
    D: ValueType,
    F: ValueType,
{
    #[must_use]
    pub(crate) fn create(value: Option<F>, parent: (String, Reference<D, F>)) -> Self {
        Self(Arc::new(RwLock::new(Internal {
            parent,
            value: Value::from_option(value),
        })))
    }
}

// =============================================================================
// Internals
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
