use std::path::Path;

use async_trait::async_trait;
use thiserror::Error;

use super::super::node::{
    data::ValueType,
    Node,
};

// Get

#[async_trait]
pub trait Get<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn get<P>(&self, path: P, get_type: GetType) -> Result<Option<Node<D, F>>, GetError>
    where
        P: AsRef<Path> + Send;

    async fn get_default<P>(&self, path: P, get_type: GetType) -> Result<Node<D, F>, GetError>
    where
        P: AsRef<Path> + Send;
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Default)]
pub enum GetType {
    #[default]
    Directory,
    File,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Error)]
pub enum GetError {
    #[error("path indicated a directory, but a file was found")]
    UnexpectedFile,
    #[error("path indicated parent directory, but current directory has no parent")]
    UnexpectedOrphan,
    #[error("path contained a prefix, which is not supported")]
    UnexpectedPrefix,
    #[error("path was an absolute (root) path, but the directory is not a root directory")]
    UnexpectedRoot,
    #[error("an internal error occurred")]
    Other,
}
