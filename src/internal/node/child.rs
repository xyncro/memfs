use async_trait::async_trait;

use super::{
    super::directory::Directory,
    data::ValueType,
};

// Child

#[async_trait]
pub trait Child<D, F> {
    async fn parent(&self) -> Option<Directory<D, F>>
    where
        D: ValueType,
        F: ValueType;
}
