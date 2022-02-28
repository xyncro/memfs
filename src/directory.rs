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
    File,
    FileData,
    GetError,
    GetPathError,
    Named,
    Node,
};

// =============================================================================

// DirectoryData

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
        self.read_lock(|dir| dir.parent.as_ref().map(|parent| parent.0.clone()))
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
    pub(crate) fn create(data: Option<D>, parent: Option<(String, DirectoryWeak<D, F>)>) -> Self {
        Self(Arc::new_cyclic(|weak| {
            Lock::new(DirectoryInternal::create(
                data,
                parent,
                DirectoryWeak(weak.clone()),
            ))
        }))
    }

    pub(crate) fn create_root() -> Self {
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

    pub async fn count_dir(&self) -> usize {
        self.count_predicate(|child| match child {
            Node::Directory(_) => true,
            _ => false,
        })
        .await
    }

    pub async fn count_file(&self) -> usize {
        self.count_predicate(|child| match child {
            Node::File(_) => true,
            _ => false,
        })
        .await
    }

    async fn count_predicate(&self, predicate: impl Fn(&Node<D, F>) -> bool) -> usize {
        self.read_lock(|dir| {
            dir.children
                .values()
                .filter(|child| predicate(child))
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
        assert_eq!(dir.count_dir().await, 0);
        assert_eq!(dir.count_file().await, 0);
    }
}

// -----------------------------------------------------------------------------

// Directory - Get

type GetResult<T> = Result<Option<T>, GetError>;

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn get(&self, name: &str) -> Option<Node<D, F>> {
        self.read_lock(|dir| dir.children.get(name).map(Clone::clone))
            .await
    }

    pub async fn get_dir(&self, name: &str) -> GetResult<Directory<D, F>> {
        self.get(name)
            .map(|child| match child {
                Some(Node::Directory(dir)) => Ok(Some(dir)),
                Some(Node::File(_)) => Err(GetError::ExpectedDir),
                _ => Ok(None),
            })
            .await
    }

    pub async fn get_file(&self, name: &str) -> GetResult<File<D, F>> {
        self.get(name)
            .map(|child| match child {
                Some(Node::Directory(_)) => Err(GetError::ExpectedFile),
                Some(Node::File(file)) => Ok(Some(file)),
                _ => Ok(None),
            })
            .await
    }
}

#[cfg(test)]
mod get_tests {
    use crate::Directory;

    #[tokio::test]
    async fn get() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.get("test").await.is_none());
    }

    #[tokio::test]
    async fn get_dir() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.get_dir("test").await.is_ok());
        assert!(dir.get_dir("test").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_file() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.get_file("test").await.is_ok());
        assert!(dir.get_file("test").await.unwrap().is_none());
    }
}

// -----------------------------------------------------------------------------

// Directory - Get (by) Path

type GetPathResult<T> = Result<Option<T>, GetPathError>;

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn get_path(&self, path: impl AsRef<Path>) -> GetPathResult<Node<D, F>> {
        let mut current = Node::Directory(self.clone());
        let components = path.as_ref().components();

        for component in components {
            match component {
                Component::Normal(name) => match &current {
                    Node::Directory(dir) => match dir.get(&name.to_string_lossy()).await {
                        Some(child) => current = child,
                        _ => return Ok(None),
                    },
                    _ => {
                        return Err(GetPathError::IntermediateFile {
                            name: name.to_string_lossy().into(),
                        });
                    }
                },
                _ => {}
            }
        }

        Ok(Some(current))
    }

    pub async fn get_path_dir(&self, path: impl AsRef<Path>) -> GetPathResult<Directory<D, F>> {
        self.get_path(path)
            .map(|child| match child {
                Ok(Some(Node::Directory(dir))) => Ok(Some(dir)),
                Ok(Some(Node::File(_))) => Err(GetPathError::ExpectedDir),
                Ok(_) => Ok(None),
                Err(err) => Err(err),
            })
            .await
    }

    pub async fn get_path_file(&self, path: impl AsRef<Path>) -> GetPathResult<File<D, F>> {
        self.get_path(path)
            .map(|child| match child {
                Ok(Some(Node::Directory(_))) => Err(GetPathError::ExpectedFile),
                Ok(Some(Node::File(file))) => Ok(Some(file)),
                Ok(_) => Ok(None),
                Err(err) => Err(err),
            })
            .await
    }
}

#[cfg(test)]
mod get_path_tests {
    use crate::Directory;

    #[tokio::test]
    async fn get_path() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.get_path("/test").await.is_ok());
        assert!(dir.get_path("/test").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_dir_path() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.get_path_dir("/test").await.is_ok());
        assert!(dir.get_path_dir("/test").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_file_path() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert!(dir.get_path_file("/test").await.is_ok());
        assert!(dir.get_path_file("/test").await.unwrap().is_none());
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
    children: HashMap<String, Node<D, F>>,
    _data: D,
    parent: Option<(String, DirectoryWeak<D, F>)>,
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
        parent: Option<(String, DirectoryWeak<D, F>)>,
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
