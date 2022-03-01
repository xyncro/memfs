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
    error::GetError,
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

pub type GetResult<T> = Result<Option<T>, GetError>;

// TODO: Tidy this in to a more meanigful config/options structure

#[derive(Clone, Debug)]
pub struct GetOptions {
    intermediate: IntermediateAction,
    endpoint: EndpointAction,
}

impl Default for GetOptions {
    fn default() -> Self {
        Self {
            intermediate: Default::default(),
            endpoint: Default::default(),
        }
    }
}

impl GetOptions {
    pub fn create(intermediate: IntermediateAction, endpoint: EndpointAction) -> Self {
        Self {
            intermediate,
            endpoint,
        }
    }
}

#[derive(Clone, Debug)]
pub enum IntermediateAction {
    CreateDefault,
    ReturnError,
    ReturnNone,
}

impl Default for IntermediateAction {
    fn default() -> Self {
        Self::ReturnNone
    }
}

#[derive(Clone, Debug)]
pub enum EndpointAction {
    CreateDefault,
    ReturnError,
    ReturnNone,
}

impl Default for EndpointAction {
    fn default() -> Self {
        Self::ReturnNone
    }
}

#[derive(Clone, Debug)]
struct GetNormalOptions {
    get: GetOptions,
    name: String,
    position: Position,
}

impl GetNormalOptions {
    fn create(options: GetOptions, name: String, position: Position) -> Self {
        Self {
            get: options,
            name,
            position,
        }
    }
}

#[derive(Clone, Debug)]
enum Position {
    Endpoint,
    Intermediate,
}

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn get(&self, path: impl AsRef<Path>, options: GetOptions) -> GetResult<Node<D, F>> {
        let mut current = Some(Node::Directory(self.clone()));
        let mut components = path.as_ref().components().peekable();

        while let Some(component) = components.next() {
            match current.as_ref() {
                Some(Node::Directory(dir)) => match component {
                    Component::CurDir => {}
                    Component::Prefix(_) => current = dir.get_prefix().await?,
                    Component::RootDir => current = dir.get_root().await?,
                    Component::ParentDir => current = dir.get_parent().await?,
                    Component::Normal(name) => {
                        let name = String::from(name.to_string_lossy());
                        let position = match components.peek() {
                            Some(_) => Position::Intermediate,
                            _ => Position::Endpoint,
                        };
                        let options = GetNormalOptions::create(options.clone(), name, position);

                        current = dir.get_normal(options).await?
                    }
                },
                Some(Node::File(_)) => return Err(GetError::UnexpectedFile),
                _ => return Ok(None),
            }
        }

        Ok(current)
    }

    async fn get_prefix(&self) -> GetResult<Node<D, F>> {
        Err(GetError::UnexpectedPrefix)
    }

    async fn get_root(&self) -> GetResult<Node<D, F>> {
        match self.is_root().await {
            true => Ok(Some(Node::Directory(self.clone()))),
            _ => Err(GetError::UnexpectedRoot),
        }
    }

    async fn get_parent(&self) -> GetResult<Node<D, F>> {
        match self.parent().await {
            Some(parent) => Ok(Some(Node::Directory(parent))),
            _ => Err(GetError::UnexpectedOrphan),
        }
    }

    async fn get_normal(&self, options: GetNormalOptions) -> GetResult<Node<D, F>> {
        match self.get_normal_node(&options.name).await {
            Some(node) => Ok(Some(node)),
            _ => match options.position {
                Position::Endpoint => self.get_normal_endpoint(options).await,
                Position::Intermediate => self.get_normal_intermediate(options).await,
            },
        }
    }

    async fn get_normal_node(&self, name: &str) -> Option<Node<D, F>> {
        self.read_lock(|dir| dir.children.get(name).map(Clone::clone))
            .await
    }

    async fn get_normal_intermediate(&self, options: GetNormalOptions) -> GetResult<Node<D, F>> {
        match options.get.intermediate {
            IntermediateAction::ReturnError => Err(GetError::IntermediateNotFound),
            IntermediateAction::ReturnNone => Ok(None),
            IntermediateAction::CreateDefault => {
                self.write_lock(|mut dir| {
                    let parent = Some((options.name.clone(), dir.weak.clone()));
                    let node = Node::Directory(Directory::create(None, parent));
                    let node = match dir.children.try_insert(options.name, node) {
                        Ok(node) => node.clone(),
                        Err(err) => err.entry.get().clone(),
                    };

                    Ok(Some(node))
                })
                .await
            }
        }
    }

    async fn get_normal_endpoint(&self, options: GetNormalOptions) -> GetResult<Node<D, F>> {
        match options.get.endpoint {
            EndpointAction::ReturnError => Err(GetError::EndpointNotFound),
            EndpointAction::ReturnNone => Ok(None),
            EndpointAction::CreateDefault => {
                self.write_lock(|mut dir| {
                    // TODO: We've assumed a file here. It doesn't have to be...

                    let parent = (options.name.clone(), dir.weak.clone());
                    let node = Node::File(File::create(None, parent));
                    let node = match dir.children.try_insert(options.name, node) {
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
