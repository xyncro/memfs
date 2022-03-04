use std::{
    ops::Deref,
    sync::Arc,
};

use async_lock::RwLock;
use async_trait::async_trait;

// Data

#[async_trait]
pub trait Data<V>
where
    V: ValueType,
{
    async fn data(&self) -> Value<V>;
}

// Value

#[derive(Debug, Default)]
pub struct Value<V>(pub(crate) Arc<RwLock<V>>)
where
    V: ValueType;

// Value - Standard Traits

impl<V> Clone for Value<V>
where
    V: ValueType,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V> Deref for Value<V>
where
    V: ValueType,
{
    type Target = Arc<RwLock<V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Value - Methods

impl<V> Value<V>
where
    V: ValueType,
{
    #[must_use]
    pub fn from_option(data: Option<V>) -> Self {
        Self(Arc::new(RwLock::new(data.unwrap_or_default())))
    }
}

// ValueType

pub trait ValueType = Default + Send + Sync;
