use async_lock::{
    RwLockReadGuard,
    RwLockWriteGuard,
};
use async_trait::async_trait;
use futures::FutureExt;

use super::data::{
    Data,
    ValueType,
};

// DataExt

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

// DataExt - Blanket Implementation

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
            .then(|value| async move { value.read().map(f).await })
            .await
    }

    async fn write<T, W>(&self, f: W) -> T
    where
        W: FnOnce(RwLockWriteGuard<'_, V>) -> T + Send,
    {
        self.data()
            .then(|value| async move { value.write().map(f).await })
            .await
    }
}
