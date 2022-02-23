use std::{
    collections::HashMap,
    path::{
        Component,
        Path,
    },
    sync::{
        Arc,
        Weak,
    },
};

use async_trait::async_trait;
use futures_util::FutureExt;
use tokio::sync::{
    RwLock as Lock,
    RwLockReadGuard as Read,
};

use crate::{
    ChildRef,
    File,
    FileData,
    FindError,
    GetError,
    Named,
    ParentRef,
};

// =============================================================================

// Directory Data

pub trait DirectoryData = Default + Send + Sync;

// =============================================================================

// Directory

#[derive(Debug)]
pub struct Directory<D, F>(Arc<Lock<DirectoryInternal<D, F>>>)
where
    D: DirectoryData,
    F: FileData;

// -----------------------------------------------------------------------------

// Directory - Trait Implementations

impl<D, F> Clone for Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl<D, F> Named for Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn named(&self) -> Option<String> {
        self.read_lock(|dir| dir.parent.as_ref().map(|parent| parent.name()))
            .await
    }
}

// -----------------------------------------------------------------------------

// Directory - Read/Write (Internal)

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn read_lock<T>(&self, f: impl FnOnce(Read<DirectoryInternal<D, F>>) -> T) -> T {
        self.0.read().map(f).await
    }
}

// -----------------------------------------------------------------------------

// Directory - Create

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn create(data: Option<D>, parent: Option<ParentRef<D, F>>) -> Self {
        Self(Arc::new_cyclic(|weak| {
            Lock::new(DirectoryInternal::create(
                data,
                parent,
                DirectoryWeak(weak.clone()),
            ))
        }))
    }

    pub fn create_root() -> Self {
        Self::create(None, None)
    }
}

#[cfg(test)]
mod create_tests {
    use super::Directory;

    #[tokio::test]
    async fn create_root() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert_eq!(dir.read_lock(|dir| dir.children.len()).await, 0);
        assert_eq!(dir.read_lock(|dir| dir._data).await, Default::default());
        assert_eq!(dir.read_lock(|dir| dir.parent.is_none()).await, true);
    }
}

// -----------------------------------------------------------------------------

// Directory - Count

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn count(&self) -> usize {
        self.read_lock(|dir| dir.children.len()).await
    }

    pub async fn count_dirs(&self) -> usize {
        self.count_predicate(|child| match child {
            ChildRef::Directory(_) => true,
            _ => false,
        })
        .await
    }

    pub async fn count_files(&self) -> usize {
        self.count_predicate(|child| match child {
            ChildRef::File(_) => true,
            _ => false,
        })
        .await
    }

    async fn count_predicate(&self, predicate: impl Fn(&ChildRef<D, F>) -> bool) -> usize {
        self.read_lock(|dir| {
            dir.children
                .iter()
                .filter(|(_, child)| predicate(child))
                .count()
        })
        .await
    }
}

#[cfg(test)]
mod count_tests {
    use super::Directory;

    #[tokio::test]
    async fn count_empty() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert_eq!(dir.count().await, 0);
        assert_eq!(dir.count_dirs().await, 0);
        assert_eq!(dir.count_files().await, 0);
    }
}

// -----------------------------------------------------------------------------

// Directory - Get

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn get(&self, name: &str) -> Option<ChildRef<D, F>> {
        self.read_lock(|dir| dir.children.get(name).map(Clone::clone))
            .await
    }

    pub async fn get_dir(&self, name: &str) -> Result<Option<Directory<D, F>>, GetError> {
        self.get(name)
            .map(|child| match child {
                Some(ChildRef::Directory(dir)) => Ok(Some(dir)),
                Some(ChildRef::File(_)) => Err(GetError::ExpectedDir),
                _ => Ok(None),
            })
            .await
    }

    pub async fn get_file(&self, name: &str) -> Result<Option<File<D, F>>, GetError> {
        self.get(name)
            .map(|child| match child {
                Some(ChildRef::Directory(_)) => Err(GetError::ExpectedFile),
                Some(ChildRef::File(file)) => Ok(Some(file)),
                _ => Ok(None),
            })
            .await
    }
}

// -----------------------------------------------------------------------------

// Directory - Find

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn find(&self, path: impl AsRef<Path>) -> Result<Option<ChildRef<D, F>>, FindError> {
        let mut current = ChildRef::Directory(self.clone());
        let components = path.as_ref().components();

        for component in components {
            match component {
                Component::Normal(name) => match &current {
                    ChildRef::Directory(dir) => match dir.get(&name.to_string_lossy()).await {
                        Some(child) => current = child,
                        _ => return Ok(None),
                    },
                    ChildRef::File(_) => {
                        return Err(FindError::IntermediateFile {
                            name: name.to_string_lossy().into(),
                        });
                    }
                },
                _ => {}
            }
        }

        Ok(Some(current))
    }

    pub async fn find_dir(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<Directory<D, F>>, FindError> {
        self.find(path)
            .map(|child| match child {
                Ok(Some(ChildRef::Directory(dir))) => Ok(Some(dir)),
                Ok(Some(ChildRef::File(_))) => Err(FindError::ExpectedDir),
                Ok(_) => Ok(None),
                Err(err) => Err(err),
            })
            .await
    }

    pub async fn find_file(&self, path: impl AsRef<Path>) -> Result<Option<File<D, F>>, FindError> {
        self.find(path)
            .map(|child| match child {
                Ok(Some(ChildRef::Directory(_))) => Err(FindError::ExpectedFile),
                Ok(Some(ChildRef::File(file))) => Ok(Some(file)),
                Ok(_) => Ok(None),
                Err(err) => Err(err),
            })
            .await
    }
}

#[cfg(test)]
mod find_tests {
    use crate::Directory;

    #[tokio::test]
    async fn find() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.find("/test").await.is_ok());
        assert!(dir.find("/test").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn find_dir() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.find_dir("/test").await.is_ok());
        assert!(dir.find_dir("/test").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn find_file() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.find_file("/test").await.is_ok());
        assert!(dir.find_file("/test").await.unwrap().is_none());
    }
}

// =============================================================================

// DirectoryWeak

#[derive(Debug)]
pub struct DirectoryWeak<D, F>(pub Weak<Lock<DirectoryInternal<D, F>>>)
where
    D: DirectoryData,
    F: FileData;

// -----------------------------------------------------------------------------

// DirectoryWeak - Trait Implementations

impl<D, F> Clone for DirectoryWeak<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// =============================================================================

// DirectoryInternal

#[derive(Debug)]
pub struct DirectoryInternal<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    children: HashMap<String, ChildRef<D, F>>,
    _data: D,
    parent: Option<ParentRef<D, F>>,
    _weak: DirectoryWeak<D, F>,
}

// -----------------------------------------------------------------------------

// DirectoryInternal - Create

impl<D, F> DirectoryInternal<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub fn create(
        data: Option<D>,
        parent: Option<ParentRef<D, F>>,
        weak: DirectoryWeak<D, F>,
    ) -> Self {
        Self {
            children: Default::default(),
            _data: data.unwrap_or_default(),
            parent,
            _weak: weak,
        }
    }
}
