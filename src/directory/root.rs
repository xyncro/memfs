use async_trait::async_trait;

use crate::ValueType;

// Root

#[async_trait]
pub trait Root<D, F> {
    async fn is_root(&self) -> bool
    where
        D: ValueType,
        F: ValueType;
}
