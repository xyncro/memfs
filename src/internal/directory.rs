pub mod count;
pub mod get;
pub mod get_ext;

use std::{
    collections::HashMap,
    ops::Deref,
    path::{
        Component,
        Path,
    },
    sync::{
        Arc,
        Weak,
    },
};

use async_lock::RwLock;
use async_trait::async_trait;
use futures::FutureExt;

use self::{
    count::Count,
    get::{
        Get,
        GetError,
        GetType,
    },
};
use super::{
    file::File,
    node::{
        child::Child,
        data::{
            Data,
            Value,
            ValueType,
        },
        named::Named,
        root::Root,
        Node,
    },
};

// Directory

#[derive(Debug)]
pub struct Directory<D, F>(pub(crate) Arc<RwLock<Internal<D, F>>>)
where
    D: ValueType,
    F: ValueType;

// Directory - Standard Traits

impl<D, F> Clone for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<D, F> Deref for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    type Target = Arc<RwLock<Internal<D, F>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Directory - Library Traits

#[async_trait]
impl<D, F> Child<D, F> for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    #[allow(clippy::use_self)]
    async fn parent(&self) -> Option<Directory<D, F>> {
        self.read()
            .map(|this| this.parent.as_ref().and_then(|parent| parent.1.upgrade()))
            .await
    }
}

#[async_trait]
impl<D, F> Count for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn count(&self) -> usize {
        self.count_predicate(|_| true).await
    }

    async fn count_dir(&self) -> usize {
        self.count_predicate(|child| matches!(child, Node::Directory(_)))
            .await
    }

    async fn count_file(&self) -> usize {
        self.count_predicate(|child| matches!(child, Node::File(_)))
            .await
    }
}

#[async_trait]
impl<D, F> Data<D> for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn data(&self) -> Value<D> {
        self.read().map(|this| this.value.clone()).await
    }
}

#[async_trait]
impl<D, F> Get<D, F> for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn get<P>(&self, path: P, get_type: GetType) -> Result<Option<Node<D, F>>, GetError>
    where
        P: AsRef<Path> + Send,
    {
        self.get(path, GetAction::ReturnNone, get_type).await
    }

    async fn get_default<P>(&self, path: P, get_type: GetType) -> Result<Node<D, F>, GetError>
    where
        P: AsRef<Path> + Send,
    {
        match self.get(path, GetAction::CreateDefault, get_type).await {
            Ok(Some(node)) => Ok(node),
            Ok(None) => Err(GetError::Other),
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl<D, F> Named for Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn name(&self) -> Option<String> {
        self.read()
            .map(|this| this.parent.as_ref().map(|parent| parent.0.clone()))
            .await
    }
}

// Directory - Methods

impl<D, F> Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    #[must_use]
    pub(crate) fn create(value: Option<D>, parent: Option<(String, Reference<D, F>)>) -> Self {
        Self(Arc::new_cyclic(|weak| {
            RwLock::new(Internal {
                children: Children::default(),
                parent,
                value: Value::from_option(value),
                weak: Reference(weak.clone()),
            })
        }))
    }

    #[must_use]
    pub(crate) fn create_root() -> Self {
        Self::create(None, None)
    }
}

#[cfg(test)]
mod create_tests {
    // use super::Directory;

    // #[tokio::test]
    // async fn create_root() {
    //     let dir: Directory<(), ()> = Directory::create_root();
    //     // let children = dir.read(|dir| dir.children.clone()).await;
    //     // let len = children.r

    //     // assert_eq!(
    //     //     dir.read(|dir| dir.children.clone())
    //     //         .then(|children| children.read(|children| children.len()))
    //     //         .await,
    //     //     0
    //     // );
    //     //assert_eq!(dir.read(|dir| dir.parent.is_none()).await, true);
    // }
}

impl<D, F> Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn count_predicate<P>(&self, predicate: P) -> usize
    where
        P: FnMut(&&Node<D, F>) -> bool + Send + Sync,
    {
        self.read()
            .then(|this| async move {
                this.children
                    .read()
                    .map(|children| children.values().filter(predicate).count())
                    .await
            })
            .await
    }
}

#[cfg(test)]
mod count_tests {
    use super::{
        Count,
        Directory,
    };

    #[tokio::test]
    async fn count_empty() {
        let dir: Directory<(), ()> = Directory::create_root();

        assert_eq!(dir.count().await, 0);
        assert_eq!(dir.count_dir().await, 0);
        assert_eq!(dir.count_file().await, 0);
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum GetAction {
    CreateDefault,
    #[default]
    ReturnNone,
}

#[derive(Clone, Copy, Debug)]
enum GetPosition {
    Child,
    Parent,
}

impl<D, F> Directory<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn get<P>(
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
                    Component::RootDir => current = dir.get_root().await?,
                    Component::ParentDir => current = dir.get_parent().await?,
                    Component::Normal(name) => {
                        let name = String::from(name.to_string_lossy());
                        let get_position = components
                            .peek()
                            .map_or(GetPosition::Child, |_| GetPosition::Parent);

                        current = dir
                            .get_named(name, get_position, get_action, get_type)
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
    async fn get_root(&self) -> Result<Option<Node<D, F>>, GetError> {
        match self.is_root().await {
            true => Ok(Some(Node::Directory(self.clone()))),
            _ => Err(GetError::UnexpectedRoot),
        }
    }

    async fn get_parent(&self) -> Result<Option<Node<D, F>>, GetError> {
        match self.parent().await {
            Some(parent) => Ok(Some(Node::Directory(parent))),
            _ => Err(GetError::UnexpectedOrphan),
        }
    }

    async fn get_named(
        &self,
        name: String,
        get_position: GetPosition,
        get_action: GetAction,
        get_type: GetType,
    ) -> Result<Option<Node<D, F>>, GetError> {
        match self.get_child(&name).await {
            Some(node) => Ok(Some(node)),
            _ => match get_position {
                GetPosition::Child => self.get_action(name, get_action, get_type).await,
                GetPosition::Parent => self.get_action(name, get_action, GetType::Directory).await,
            },
        }
    }

    async fn get_child(&self, name: &str) -> Option<Node<D, F>> {
        self.read()
            .then(|this| async move {
                this.children
                    .read()
                    .map(|children| children.get(name).map(Clone::clone))
                    .await
            })
            .await
    }

    async fn get_action(
        &self,
        name: String,
        get_action: GetAction,
        get_type: GetType,
    ) -> Result<Option<Node<D, F>>, GetError> {
        match get_action {
            GetAction::CreateDefault => {
                self.read()
                    .then(|this| async move {
                        let parent = (name.clone(), this.weak.clone());
                        let new_node = match get_type {
                            GetType::Directory => Node::Directory(Self::create(None, Some(parent))),
                            GetType::File => Node::File(File::create(None, parent)),
                        };

                        let node = this
                            .children
                            .write()
                            .map(|mut children| match children.try_insert(name, new_node) {
                                Ok(node) => node.clone(),
                                Err(err) => err.entry.get().clone(),
                            })
                            .await;

                        Ok(Some(node))
                    })
                    .await
            }
            GetAction::ReturnNone => Ok(None),
        }
    }
}

// Children

#[derive(Debug, Default)]
pub struct Children<D, F>(pub(crate) Arc<RwLock<HashMap<String, Node<D, F>>>>)
where
    D: ValueType,
    F: ValueType;

// Children - Standard Traits

impl<D, F> Clone for Children<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<D, F> Deref for Children<D, F>
where
    D: ValueType,
    F: ValueType,
{
    type Target = Arc<RwLock<HashMap<String, Node<D, F>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Reference

#[derive(Debug)]
pub struct Reference<D, F>(pub(crate) Weak<RwLock<Internal<D, F>>>)
where
    D: ValueType,
    F: ValueType;

// Reference - Standard Traits

impl<D, F> Clone for Reference<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Reference - Methods

impl<D, F> Reference<D, F>
where
    D: ValueType,
    F: ValueType,
{
    fn upgrade(&self) -> Option<Directory<D, F>> {
        self.0.upgrade().map(Directory)
    }
}

// Internal

#[derive(Debug)]
pub struct Internal<D, F>
where
    D: ValueType,
    F: ValueType,
{
    children: Children<D, F>,
    parent: Option<(String, Reference<D, F>)>,
    value: Value<D>,
    weak: Reference<D, F>,
}
