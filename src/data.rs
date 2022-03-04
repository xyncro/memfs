use std::{
    ops::Deref,
    sync::Arc,
};

use async_lock::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
use async_trait::async_trait;
use futures::FutureExt;

// =============================================================================
// Data
// =============================================================================

#[async_trait]
pub trait Data<V>
where
    V: ValueType,
{
    async fn data(&self) -> Value<V>;
}

// =============================================================================
// DataExt
// =============================================================================

#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait DataExt<V>
where
    V: ValueType,
{
    async fn read<T, R>(&self, f: R) -> T
    where
        R: FnOnce(RwLockReadGuard<'_, V>) -> T + Send;

    async fn write<T, W>(&self, f: W) -> T
    where
        W: FnMut(RwLockWriteGuard<'_, V>) -> T + Send;
}

// -----------------------------------------------------------------------------
// DataExt - Implementations
// -----------------------------------------------------------------------------

#[async_trait]
impl<D, V> DataExt<V> for D
where
    D: Data<V> + Sync,
    V: ValueType,
{
    async fn read<T, R>(&self, f: R) -> T
    where
        R: FnOnce(RwLockReadGuard<'_, V>) -> T + Send,
    {
        self.data()
            .then(|value| async move { value.read().map(|value| f(value)).await })
            .await
    }

    async fn write<T, W>(&self, f: W) -> T
    where
        W: FnOnce(RwLockWriteGuard<'_, V>) -> T + Send,
    {
        self.data()
            .then(|value| async move { value.write().map(|value| f(value)).await })
            .await
    }
}

// =============================================================================
// Value
// =============================================================================

#[derive(Debug)]
pub struct Value<D>(pub(crate) Arc<RwLock<D>>)
where
    D: ValueType;

// -----------------------------------------------------------------------------
// Value - Traits
// -----------------------------------------------------------------------------

impl<D> Clone for Value<D>
where
    D: ValueType,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<D> Default for Value<D>
where
    D: ValueType,
{
    fn default() -> Self {
        Self(Arc::new(RwLock::new(D::default())))
    }
}

impl<D> Deref for Value<D>
where
    D: ValueType,
{
    type Target = Arc<RwLock<D>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// -----------------------------------------------------------------------------
// Value - Methods
// -----------------------------------------------------------------------------

// Value - Methods - From

impl<D> Value<D>
where
    D: ValueType,
{
    #[must_use]
    pub fn from_option(data: Option<D>) -> Self {
        Self(Arc::new(RwLock::new(data.unwrap_or_default())))
    }
}

// =============================================================================
// ValueType
// =============================================================================

pub trait ValueType = Default + Send + Sync;
