use std::path::PathBuf;

use async_trait::async_trait;
use futures::FutureExt;

use super::{
    child::Child,
    data::ValueType,
    named::Named,
};

// Located

#[async_trait]
pub trait Located<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn path(&self) -> PathBuf;
}

// Located - Blanket Implementation

#[async_trait]
impl<N, D, F> Located<D, F> for N
where
    N: Child<D, F> + Named + Sync,
    D: ValueType,
    F: ValueType,
{
    async fn path(&self) -> PathBuf {
        if let Some(name) = self.name().await {
            if let Some(parent) = self.parent().await {
                return parent.path().map(|path| path.join(name)).await;
            }
        }

        PathBuf::from("/")
    }
}
