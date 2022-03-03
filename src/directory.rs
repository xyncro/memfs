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

#[cfg(doc)]
use crate::FileSystem;
use crate::{
    Child,
    File,
    FileData,
    Name,
    Node,
    Root,
};

// =============================================================================
// Directory
// =============================================================================

#[derive(Debug)]
pub struct Directory<D, F>(pub(crate) Arc<Lock<Internal<D, F>>>)
where
    D: Data,
    F: FileData;

// -----------------------------------------------------------------------------
// Directory - Traits
// -----------------------------------------------------------------------------

impl<D, F> Clone for Directory<D, F>
where
    D: Data,
    F: FileData,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl<D, F> Name for Directory<D, F>
where
    D: Data,
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
    D: Data,
    F: FileData,
{
    async fn parent(&self) -> Option<Self> {
        self.read(|dir| dir.parent.as_ref().map(|parent| parent.1.clone()))
            .map(|parent| parent.and_then(|Reference(dir)| dir.upgrade()))
            .map(|parent| parent.map(Directory))
            .await
    }
}

#[async_trait]
impl<D, F> Root<D, F> for Directory<D, F>
where
    D: Data,
    F: FileData,
{
    async fn is_root(&self) -> bool {
        self.read(|dir| dir.parent.is_none()).await
    }
}

// -----------------------------------------------------------------------------
// Directory - Methods
// -----------------------------------------------------------------------------

// Directory - Methods - Read & Write

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    async fn read<T, R>(&self, f: R) -> T
    where
        R: FnOnce(Read<'_, Internal<D, F>>) -> T + Send,
    {
        self.0.read().map(f).await
    }

    async fn write<T, W>(&self, f: W) -> T
    where
        W: FnOnce(Write<'_, Internal<D, F>>) -> T + Send,
    {
        self.0.write().map(f).await
    }
}

// Directory - Methods - Create

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    pub(crate) fn create(data: Option<D>, parent: Option<(String, Reference<D, F>)>) -> Self {
        Self(Arc::new_cyclic(|weak| {
            Lock::new(Internal {
                children: HashMap::default(),
                _data: data.unwrap_or_default(),
                parent,
                weak: Reference(weak.clone()),
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

// Directory - Methods - Count

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    pub async fn count(&self) -> usize {
        self.read(|dir| dir.children.len()).await
    }

    pub async fn count_dir(&self) -> usize {
        self.count_predicate(|child| matches!(child, Node::Directory(_)))
            .await
    }

    pub async fn count_file(&self) -> usize {
        self.count_predicate(|child| matches!(child, Node::File(_)))
            .await
    }

    async fn count_predicate<P>(&self, predicate: P) -> usize
    where
        P: Fn(&Node<D, F>) -> bool + Send + Sync,
    {
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

// Directory - Methods - Get

#[derive(Clone, Copy, Debug)]
pub enum GetType {
    Directory,
    File,
}

impl Default for GetType {
    fn default() -> Self {
        Self::Directory
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GetAction {
    CreateDefault,
    ReturnNone,
}

impl Default for GetAction {
    fn default() -> Self {
        Self::ReturnNone
    }
}

#[derive(Clone, Copy, Debug)]
enum GetPosition {
    Child,
    Parent,
}

impl GetPosition {
    const fn from_next(component: Option<&Component<'_>>) -> Self {
        match component {
            Some(_) => Self::Parent,
            _ => Self::Child,
        }
    }
}

#[derive(Clone, Copy, Debug, Diagnostic, Error)]
pub enum GetError {
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
    #[diagnostic(code(directory::get::other), help("please report this error"))]
    #[error("an internal error occurred")]
    Other,
}

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    pub async fn get<P>(&self, path: P, get_type: GetType) -> Result<Option<Node<D, F>>, GetError>
    where
        P: AsRef<Path> + Send,
    {
        self.get_node(path, GetAction::ReturnNone, get_type).await
    }

    pub async fn get_default<P>(&self, path: P, get_type: GetType) -> Result<Node<D, F>, GetError>
    where
        P: AsRef<Path> + Send,
    {
        match self
            .get_node(path, GetAction::CreateDefault, get_type)
            .await
        {
            Ok(Some(node)) => Ok(node),
            Ok(None) => Err(GetError::Other),
            Err(err) => Err(err),
        }
    }
}

// Directory - Methods - GetDir

#[derive(Clone, Copy, Debug, Diagnostic, Error)]
pub enum GetDirError {
    #[diagnostic(code(directory::get_dir::file), help("check the supplied path"))]
    #[error("expected directory, but file found")]
    UnexpectedFile,
    #[error("internal error getting node")]
    Get(#[from] GetError),
}

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    pub async fn get_dir<P>(&self, path: P) -> Result<Option<Self>, GetDirError>
    where
        P: AsRef<Path> + Send,
    {
        match self
            .get_node(path, GetAction::ReturnNone, GetType::Directory)
            .await
        {
            Ok(Some(Node::Directory(dir))) => Ok(Some(dir)),
            Ok(Some(Node::File(_))) => Err(GetDirError::UnexpectedFile),
            Ok(None) => Ok(None),
            Err(err) => Err(GetDirError::Get(err)),
        }
    }

    pub async fn get_dir_default<P>(&self, path: P) -> Result<Self, GetDirError>
    where
        P: AsRef<Path> + Send,
    {
        match self
            .get_node(path, GetAction::CreateDefault, GetType::Directory)
            .await
        {
            Ok(Some(Node::Directory(dir))) => Ok(dir),
            Ok(Some(Node::File(_))) => Err(GetDirError::UnexpectedFile),
            Ok(None) => Err(GetDirError::Get(GetError::Other)),
            Err(err) => Err(GetDirError::Get(err)),
        }
    }
}

// Directory - Methods - GetFile

#[derive(Clone, Copy, Debug, Diagnostic, Error)]
pub enum GetFileError {
    #[diagnostic(code(directory::get_file::directory), help("check the supplied path"))]
    #[error("expected file, but directory found")]
    UnexpectedDirectory,
    #[error("internal error getting node")]
    Get(#[from] GetError),
}

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    pub async fn get_file<P>(&self, path: P) -> Result<Option<File<D, F>>, GetFileError>
    where
        P: AsRef<Path> + Send,
    {
        match self
            .get_node(path, GetAction::ReturnNone, GetType::File)
            .await
        {
            Ok(Some(Node::Directory(_))) => Err(GetFileError::UnexpectedDirectory),
            Ok(Some(Node::File(file))) => Ok(Some(file)),
            Ok(None) => Ok(None),
            Err(err) => Err(GetFileError::Get(err)),
        }
    }

    pub async fn get_file_default<P>(&self, path: P) -> Result<File<D, F>, GetFileError>
    where
        P: AsRef<Path> + Send,
    {
        match self
            .get_node(path, GetAction::CreateDefault, GetType::File)
            .await
        {
            Ok(Some(Node::Directory(_))) => Err(GetFileError::UnexpectedDirectory),
            Ok(Some(Node::File(file))) => Ok(file),
            Ok(None) => Err(GetFileError::Get(GetError::Other)),
            Err(err) => Err(GetFileError::Get(err)),
        }
    }
}

// Directory - Methods - GetNode

impl<D, F> Directory<D, F>
where
    D: Data,
    F: FileData,
{
    async fn get_node<P>(
        &self,
        path: P,
        get_action: GetAction,
        get_type: GetType,
    ) -> Result<Option<Node<D, F>>, GetError>
    where
        P: AsRef<Path> + Send,
    {
        let mut current = Some(Node::Directory(self.clone()));
        let mut components = path.as_ref().components().peekable();

        while let Some(component) = components.next() {
            match current.as_ref() {
                Some(Node::Directory(dir)) => match component {
                    Component::CurDir => {}
                    Component::Prefix(_) => return Err(GetError::UnexpectedPrefix),
                    Component::RootDir => current = dir.get_node_root().await?,
                    Component::ParentDir => current = dir.get_node_parent().await?,
                    Component::Normal(name) => {
                        let name = String::from(name.to_string_lossy());
                        let get_position = GetPosition::from_next(components.peek());

                        current = dir
                            .get_node_named(name, get_position, get_action, get_type)
                            .await?;
                    }
                },
                Some(Node::File(_)) => return Err(GetError::UnexpectedFile),
                _ => return Ok(None),
            }
        }

        Ok(current)
    }

    #[allow(clippy::match_bool)]
    async fn get_node_root(&self) -> Result<Option<Node<D, F>>, GetError> {
        match self.is_root().await {
            true => Ok(Some(Node::Directory(self.clone()))),
            _ => Err(GetError::UnexpectedRoot),
        }
    }

    async fn get_node_parent(&self) -> Result<Option<Node<D, F>>, GetError> {
        match self.parent().await {
            Some(parent) => Ok(Some(Node::Directory(parent))),
            _ => Err(GetError::UnexpectedOrphan),
        }
    }

    async fn get_node_named(
        &self,
        name: String,
        get_position: GetPosition,
        get_action: GetAction,
        get_type: GetType,
    ) -> Result<Option<Node<D, F>>, GetError> {
        match self.get_node_named_child(&name).await {
            Some(node) => Ok(Some(node)),
            _ => match get_position {
                GetPosition::Child => {
                    self.get_node_named_child_action(name, get_action, get_type)
                        .await
                }
                GetPosition::Parent => {
                    self.get_node_named_child_action(name, get_action, GetType::Directory)
                        .await
                }
            },
        }
    }

    async fn get_node_named_child(&self, name: &str) -> Option<Node<D, F>> {
        self.read(|dir| dir.children.get(name).map(Clone::clone))
            .await
    }

    async fn get_node_named_child_action(
        &self,
        name: String,
        get_action: GetAction,
        get_type: GetType,
    ) -> Result<Option<Node<D, F>>, GetError> {
        match get_action {
            GetAction::CreateDefault => {
                self.write(|mut dir| {
                    let parent = (name.clone(), dir.weak.clone());
                    let node = match get_type {
                        GetType::Directory => {
                            let dir = Self::create(None, Some(parent));
                            Node::Directory(dir)
                        }
                        GetType::File => {
                            let file = File::create(None, parent);
                            Node::File(file)
                        }
                    };
                    let node = match dir.children.try_insert(name, node) {
                        Ok(node) => node.clone(),
                        Err(err) => err.entry.get().clone(),
                    };

                    Ok(Some(node))
                })
                .await
            }
            GetAction::ReturnNone => Ok(None),
        }
    }
}

// =============================================================================
// Reference
// =============================================================================

#[derive(Debug)]
pub struct Reference<D, F>(pub(crate) Weak<Lock<Internal<D, F>>>)
where
    D: Data,
    F: FileData;

// -----------------------------------------------------------------------------
// Reference - Traits
// -----------------------------------------------------------------------------

impl<D, F> Clone for Reference<D, F>
where
    D: Data,
    F: FileData,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// =============================================================================
// Internal
// =============================================================================

#[derive(Debug)]
pub struct Internal<D, F>
where
    D: Data,
    F: FileData,
{
    children: HashMap<String, Node<D, F>>,
    _data: D,
    parent: Option<(String, Reference<D, F>)>,
    weak: Reference<D, F>,
}

// =============================================================================
// Data
// =============================================================================

pub trait Data = Default + Send + Sync;
