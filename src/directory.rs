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
use miette::Diagnostic;
use thiserror::Error;
use tokio::sync::{
    RwLock as Lock,
    RwLockReadGuard as Read,
    RwLockWriteGuard as Write,
};

use crate::{
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

// Directory - Traits

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
        self.read(|dir| dir.parent.as_ref().map(|parent| parent.0.clone()))
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
        self.read(|dir| dir.parent.as_ref().map(|parent| parent.1.clone()))
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
        self.read(|dir| dir.parent.is_none()).await
    }
}

// -----------------------------------------------------------------------------

// Directory - Read & Write

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn read<T>(&self, f: impl FnOnce(Read<DirectoryInternal<D, F>>) -> T) -> T {
        self.0.read().map(f).await
    }

    async fn write<T>(&self, f: impl FnOnce(Write<DirectoryInternal<D, F>>) -> T) -> T {
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

        assert_eq!(dir.read(|dir| dir.children.len()).await, 0);
        assert_eq!(dir.read(|dir| dir._data).await, Default::default());
        assert_eq!(dir.read(|dir| dir.parent.is_none()).await, true);
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
        self.read(|dir| dir.children.len()).await
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
        self.read(|dir| {
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

// GetOptions

#[derive(Clone, Debug)]
pub struct GetOptions {
    intermediate: GetIntermediateAction,
    endpoint: GetEndpointAction,
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
    pub fn create(intermediate: GetIntermediateAction, endpoint: GetEndpointAction) -> Self {
        Self {
            intermediate,
            endpoint,
        }
    }
}

// GetIntermediateAction

#[derive(Clone, Debug)]
pub enum GetIntermediateAction {
    CreateDefault,
    ReturnError,
    ReturnNone,
}

impl Default for GetIntermediateAction {
    fn default() -> Self {
        Self::ReturnNone
    }
}

// GetEndpointAction

#[derive(Clone, Debug)]
pub enum GetEndpointAction {
    CreateDefault,
    ReturnError,
    ReturnNone,
}

impl Default for GetEndpointAction {
    fn default() -> Self {
        Self::ReturnNone
    }
}

// GetDirResult

pub type GetDirResult<D, F> = Result<Option<Directory<D, F>>, GetDirError>;

// GetDirError

#[derive(Debug, Diagnostic, Error)]
pub enum GetDirError {
    #[diagnostic(code(directory::get_dir::file), help("check the supplied path"))]
    #[error("expected directory, but file found")]
    UnexpectedFile,
    #[error("internal error getting node")]
    Node(#[from] GetNodeError),
}

// GetDir

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn get_dir(&self, path: impl AsRef<Path>, options: GetOptions) -> GetDirResult<D, F> {
        let options = GetNodeOptions {
            get: options,
            _get_node_type: GetNodeType::Directory,
        };

        match self.get_node(path, options).await {
            Ok(Some(Node::Directory(dir))) => Ok(Some(dir)),
            Ok(Some(Node::File(_))) => Err(GetDirError::UnexpectedFile),
            Ok(None) => Ok(None),
            Err(err) => Err(GetDirError::Node(err)),
        }
    }
}

// GetFileResult

pub type GetFileResult<D, F> = Result<Option<File<D, F>>, GetFileError>;

// GetFileError

#[derive(Debug, Diagnostic, Error)]
pub enum GetFileError {
    #[diagnostic(code(directory::get_file::directory), help("check the supplied path"))]
    #[error("expected file, but directory found")]
    UnexpectedDirectory,
    #[error("internal error getting node")]
    Node(#[from] GetNodeError),
}

// GetFile

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    pub async fn get_file(
        &self,
        path: impl AsRef<Path>,
        options: GetOptions,
    ) -> GetFileResult<D, F> {
        let options = GetNodeOptions {
            get: options,
            _get_node_type: GetNodeType::File,
        };

        match self.get_node(path, options).await {
            Ok(Some(Node::Directory(_))) => Err(GetFileError::UnexpectedDirectory),
            Ok(Some(Node::File(file))) => Ok(Some(file)),
            Ok(None) => Ok(None),
            Err(err) => Err(GetFileError::Node(err)),
        }
    }
}

// GetNodeResult

pub type GetNodeResult<T> = Result<Option<T>, GetNodeError>;

// GetNodeError

#[derive(Debug, Diagnostic, Error)]
pub enum GetNodeError {
    #[diagnostic(code(directory::get::file), help("check the supplied path"))]
    #[error("path indicated a directory, but a file was found")]
    UnexpectedFile,
    #[diagnostic(code(directory::get::orphan), help("check the supplied path"))]
    #[error("path indicated parent directory, but current directory has no parent")]
    UnexpectedOrphan,
    #[diagnostic(code(directory::get::prefix), help("check the supplied path"))]
    #[error("path contained a prefix, which is not supported")]
    UnexpectedPrefix,
    #[diagnostic(code(directory::get::root), help("check the supplied path"))]
    #[error("path was an absolute (root) path, but the directory is not a root directory")]
    UnexpectedRoot,
    #[diagnostic(code(directory::get::endpoint), help("check the supplied path"))]
    #[error("the endpoint does not exist")]
    EndpointNotFound,
    #[diagnostic(code(directory::get::intermediate), help("check the supplied path"))]
    #[error("an intermediate directory does not exist")]
    IntermediateNotFound,
}

// GetNodeOptions

#[derive(Clone, Debug)]
struct GetNodeOptions {
    get: GetOptions,
    _get_node_type: GetNodeType,
}

// GetNodeType

#[derive(Clone, Debug)]
enum GetNodeType {
    Directory,
    File,
}

// GetNode

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn get_node(
        &self,
        path: impl AsRef<Path>,
        options: GetNodeOptions,
    ) -> GetNodeResult<Node<D, F>> {
        let mut current = Some(Node::Directory(self.clone()));
        let mut components = path.as_ref().components().peekable();

        while let Some(component) = components.next() {
            match current.as_ref() {
                Some(Node::Directory(dir)) => match component {
                    Component::CurDir => {}
                    Component::Prefix(_) => return Err(GetNodeError::UnexpectedPrefix),
                    Component::RootDir => current = dir.get_node_root().await?,
                    Component::ParentDir => current = dir.get_node_parent().await?,
                    Component::Normal(name) => {
                        let name = String::from(name.to_string_lossy());
                        let position = GetNormalPosition::from_next(components.peek());
                        let options = GetNormalOptions::create(options.clone(), name, position);

                        current = dir.get_normal(options).await?
                    }
                },
                Some(Node::File(_)) => return Err(GetNodeError::UnexpectedFile),
                _ => return Ok(None),
            }
        }

        Ok(current)
    }

    async fn get_node_root(&self) -> GetNodeResult<Node<D, F>> {
        match self.is_root().await {
            true => Ok(Some(Node::Directory(self.clone()))),
            _ => Err(GetNodeError::UnexpectedRoot),
        }
    }

    async fn get_node_parent(&self) -> GetNodeResult<Node<D, F>> {
        match self.parent().await {
            Some(parent) => Ok(Some(Node::Directory(parent))),
            _ => Err(GetNodeError::UnexpectedOrphan),
        }
    }
}

// GetNormalOptions

#[derive(Clone, Debug)]
struct GetNormalOptions {
    node: GetNodeOptions,
    name: String,
    position: GetNormalPosition,
}

impl GetNormalOptions {
    fn create(node: GetNodeOptions, name: String, position: GetNormalPosition) -> Self {
        Self {
            node,
            name,
            position,
        }
    }
}

// GetNormalPosition

#[derive(Clone, Debug)]
enum GetNormalPosition {
    Endpoint,
    Intermediate,
}

impl GetNormalPosition {
    fn from_next(component: Option<&Component>) -> Self {
        match component {
            Some(_) => Self::Intermediate,
            _ => Self::Endpoint,
        }
    }
}

// GetNormal

impl<D, F> Directory<D, F>
where
    D: DirectoryData,
    F: FileData,
{
    async fn get_normal(&self, options: GetNormalOptions) -> GetNodeResult<Node<D, F>> {
        match self.get_normal_child(&options.name).await {
            Some(node) => Ok(Some(node)),
            _ => match options.position {
                GetNormalPosition::Endpoint => self.get_normal_end(options).await,
                GetNormalPosition::Intermediate => self.get_normal_inter(options).await,
            },
        }
    }

    async fn get_normal_child(&self, name: &str) -> Option<Node<D, F>> {
        self.read(|dir| dir.children.get(name).map(Clone::clone))
            .await
    }

    async fn get_normal_inter(&self, options: GetNormalOptions) -> GetNodeResult<Node<D, F>> {
        match options.node.get.intermediate {
            GetIntermediateAction::ReturnError => Err(GetNodeError::IntermediateNotFound),
            GetIntermediateAction::ReturnNone => Ok(None),
            GetIntermediateAction::CreateDefault => {
                self.write(|mut dir| {
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

    async fn get_normal_end(&self, options: GetNormalOptions) -> GetNodeResult<Node<D, F>> {
        match options.node.get.endpoint {
            GetEndpointAction::ReturnError => Err(GetNodeError::EndpointNotFound),
            GetEndpointAction::ReturnNone => Ok(None),
            GetEndpointAction::CreateDefault => {
                self.write(|mut dir| {
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

// DirectoryWeak - Traits

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
