use async_trait::async_trait;
use futures::FutureExt;

use super::{
    child::Child,
    data::ValueType,
};

// Root

#[async_trait]
pub trait Root<D, F> {
    async fn is_root(&self) -> bool
    where
        D: ValueType,
        F: ValueType;
}

// Root - Blanket Implementation

#[async_trait]
impl<N, D, F> Root<D, F> for N
where
    N: Child<D, F> + Sync,
    D: ValueType,
    F: ValueType,
{
    async fn is_root(&self) -> bool {
        self.parent().map(|parent| parent.is_none()).await
    }
}
