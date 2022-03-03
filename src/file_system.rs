use std::ops::Deref;

use async_trait::async_trait;

use crate::{
    Directory,
    ValueType,
};

// =============================================================================
// Child/Parent/Root
// =============================================================================

#[async_trait]
pub trait Child<D, F> {
    async fn parent(&self) -> Option<Directory<D, F>>
    where
        D: ValueType,
        F: ValueType;
}

// #[async_trait]
// pub trait Parent<D, F> {
//     async fn children(&self) -> &HashMap<String, Node<D, F>>
//     where
//         D: ValueType,
//         F: ValueType;
// }

#[async_trait]
pub trait Root<D, F> {
    async fn is_root(&self) -> bool
    where
        D: ValueType,
        F: ValueType;
}

// =============================================================================
// Name
// =============================================================================

#[async_trait]
pub trait Name {
    async fn name(&self) -> Option<String>;
}

// =============================================================================
// FileSystem
// =============================================================================

#[derive(Debug)]
pub struct FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    root: Directory<D, F>,
}

// -----------------------------------------------------------------------------
// FileSystem - Traits
// -----------------------------------------------------------------------------

impl<D, F> Default for FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D, F> Deref for FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    type Target = Directory<D, F>;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

// -----------------------------------------------------------------------------
// FileSystem - Methods
// -----------------------------------------------------------------------------

// FileSystem - Methods - New

impl<D, F> FileSystem<D, F>
where
    D: ValueType,
    F: ValueType,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Directory::create_root(),
        }
    }
}
