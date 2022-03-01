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
    RwLockWriteGuard as Write,
};

use crate::{
    error::OpenError,
    Child,
    File,
    FileData,
    Name,
    Node,
    Root,
};

// =============================================================================

// DirectoryData

pub trait DirectoryData = Default + Send + Sync;

// =============================================================================

// Directory

#[derive(Debug)]
pub struct Directory<D, F>(pub(crate) Arc<Lock<DirectoryInternal<D, F>>>)
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
impl<D, F> Name for Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn name(&self) -> Option<String> {
        self.read_lock(|dir| dir.parent.as_ref().map(|parent| parent.0.clone()))
            .await
    }
}

#[async_trait]
impl<D, F> Child<D, F> for Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn parent(&self) -> Option<Directory<D, F>> {
        self.read_lock(|dir| dir.parent.as_ref().map(|parent| parent.1.clone()))
            .map(|parent| parent.and_then(|DirectoryWeak(dir)| dir.upgrade()))
            .map(|parent| parent.map(Directory))
            .await
    }
}

#[async_trait]
impl<D, F> Root<D, F> for Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn is_root(&self) -> bool {
        self.read_lock(|dir| dir.parent.is_none()).await
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

    async fn write_lock<T>(&self, f: impl FnOnce(Write<DirectoryInternal<D, F>>) -> T) -> T {
        self.0.write().map(f).await
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
            Lock::new(DirectoryInternal {
                children: Default::default(),
                _data: data.unwrap_or_default(),
                parent,
                weak: DirectoryWeak(weak.clone()),
            })
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

// Directory - Open (?)

pub type OpenResult<T> = Result<Option<T>, OpenError>;

// TODO: Tidy this in to a more meanigful config/options structure

#[derive(Clone, Copy, Debug)]
pub enum OpenIntermediate {
    Default,
    Error,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum OpenEndpoint {
    Default,
    Error,
    None,
}

#[derive(Clone, Copy, Debug)]
enum Position {
    Endpoint,
    Intermediate,
}

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn open(
        &self,
        path: impl AsRef<Path>,
        end: OpenEndpoint,
        inter: OpenIntermediate,
    ) -> OpenResult<Node<D, F>> {
        let mut current = Some(Node::Directory(self.clone()));
        let mut components = path.as_ref().components().peekable();

        while let Some(component) = components.next() {
            match current.as_ref() {
                Some(node) => match component {
                    Component::CurDir => {}
                    Component::Prefix(_) => current = self.open_prefix().await?,
                    Component::RootDir => current = self.open_root(node.clone()).await?,
                    Component::ParentDir => current = self.open_parent(node.clone()).await?,
                    Component::Normal(name) => {
                        let node = node.clone();
                        let name = String::from(name.to_string_lossy());
                        let position = match components.peek() {
                            Some(_) => Position::Intermediate,
                            _ => Position::Endpoint,
                        };

                        current = self.open_normal(node, name, position, end, inter).await?
                    }
                },
                _ => return Ok(None),
            }
        }

        Ok(current)
    }

    async fn open_prefix(&self) -> OpenResult<Node<D, F>> {
        Err(OpenError::UnexpectedPrefix)
    }

    async fn open_root(&self, current: Node<D, F>) -> OpenResult<Node<D, F>> {
        match current.is_root().await {
            true => Ok(Some(current)),
            _ => Err(OpenError::UnexpectedRoot),
        }
    }

    async fn open_parent(&self, current: Node<D, F>) -> OpenResult<Node<D, F>> {
        match current.parent().await {
            Some(parent) => Ok(Some(Node::Directory(parent))),
            _ => Err(OpenError::UnexpectedOrphan),
        }
    }

    async fn open_normal(
        &self,
        current: Node<D, F>,
        name: String,
        position: Position,
        end: OpenEndpoint,
        inter: OpenIntermediate,
    ) -> OpenResult<Node<D, F>> {
        match current {
            Node::Directory(dir) => {
                let node = dir
                    .read_lock(|dir| dir.children.get(&name).map(Clone::clone))
                    .await;

                match node {
                    Some(node) => Ok(Some(node)),
                    _ => match position {
                        Position::Endpoint => self.open_normal_end(dir, name, end).await,
                        Position::Intermediate => self.open_normal_inter(dir, name, inter).await,
                    },
                }
            }
            Node::File(_) => Err(OpenError::UnexpectedFile),
        }
    }

    async fn open_normal_end(
        &self,
        dir: Directory<D, F>,
        name: String,
        end: OpenEndpoint,
    ) -> OpenResult<Node<D, F>> {
        match end {
            OpenEndpoint::Error => Err(OpenError::EndpointNotFound),
            OpenEndpoint::None => Ok(None),
            OpenEndpoint::Default => {
                dir.write_lock(|mut dir| {
                    // TODO: We've assumed a file here. It doesn't have to be...

                    let parent = (name.clone(), dir.weak.clone());
                    let node = Node::File(File::create(None, parent));
                    let node = match dir.children.try_insert(name, node) {
                        Ok(node) => node.clone(),
                        Err(err) => err.entry.get().clone(),
                    };

                    Ok(Some(node))
                })
                .await
            }
        }
    }

    async fn open_normal_inter(
        &self,
        dir: Directory<D, F>,
        name: String,
        inter: OpenIntermediate,
    ) -> OpenResult<Node<D, F>> {
        match inter {
            OpenIntermediate::Error => Err(OpenError::IntermediateNotFound),
            OpenIntermediate::None => Ok(None),
            OpenIntermediate::Default => {
                dir.write_lock(|mut dir| {
                    let parent = Some((name.clone(), dir.weak.clone()));
                    let node = Node::Directory(Directory::create(None, parent));
                    let node = match dir.children.try_insert(name, node) {
                        Ok(node) => node.clone(),
                        Err(err) => err.entry.get().clone(),
                    };

                    Ok(Some(node))
                })
                .await
            }
        }
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
    weak: DirectoryWeak<D, F>,
}
