use async_trait::async_trait;

use super::super::node::data::ValueType;

// Root

#[async_trait]
pub trait Root<D, F> {
    async fn is_root(&self) -> bool
    where
        D: ValueType,
        F: ValueType;
}
