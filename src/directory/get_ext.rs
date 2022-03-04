use std::path::Path;

use async_trait::async_trait;
use thiserror::Error;

use super::get::GetType;
use crate::{
    Directory,
    File,
    Get,
    GetError,
    Node,
    ValueType,
};

// =============================================================================
// GetExt
// =============================================================================

#[async_trait]
pub trait GetExt<D, F>
where
    D: ValueType,
    F: ValueType,
{
    async fn get_dir<P>(&self, path: P) -> Result<Option<Directory<D, F>>, GetDirError>
    where
        P: AsRef<Path> + Send;

    async fn get_dir_default<P>(&self, path: P) -> Result<Directory<D, F>, GetDirError>
    where
        P: AsRef<Path> + Send;

    async fn get_file<P>(&self, path: P) -> Result<Option<File<D, F>>, GetFileError>
    where
        P: AsRef<Path> + Send;

    async fn get_file_default<P>(&self, path: P) -> Result<File<D, F>, GetFileError>
    where
        P: AsRef<Path> + Send;
}

#[derive(Clone, Copy, Debug, Error)]
pub enum GetDirError {
    #[error("expected directory, but file found")]
    UnexpectedFile,
    #[error("internal error getting node")]
    Get(#[from] GetError),
}

#[derive(Clone, Copy, Debug, Error)]
pub enum GetFileError {
    #[error("expected file, but directory found")]
    UnexpectedDirectory,
    #[error("internal error getting node")]
    Get(#[from] GetError),
}

// -----------------------------------------------------------------------------
// GetExt - Blanket Implementation
// -----------------------------------------------------------------------------

#[async_trait]
impl<D, F, G> GetExt<D, F> for G
where
    G: Get<D, F> + Sync,
    D: ValueType,
    F: ValueType,
{
    async fn get_dir<P>(&self, path: P) -> Result<Option<Directory<D, F>>, GetDirError>
    where
        P: AsRef<Path> + Send,
    {
        match self.get(path, GetType::Directory).await {
            Ok(Some(Node::Directory(dir))) => Ok(Some(dir)),
            Ok(Some(Node::File(_))) => Err(GetDirError::UnexpectedFile),
            Ok(None) => Ok(None),
            Err(err) => Err(GetDirError::Get(err)),
        }
    }

    async fn get_dir_default<P>(&self, path: P) -> Result<Directory<D, F>, GetDirError>
    where
        P: AsRef<Path> + Send,
    {
        match self.get_default(path, GetType::Directory).await {
            Ok(Node::Directory(dir)) => Ok(dir),
            Ok(Node::File(_)) => Err(GetDirError::UnexpectedFile),
            Err(err) => Err(GetDirError::Get(err)),
        }
    }

    async fn get_file<P>(&self, path: P) -> Result<Option<File<D, F>>, GetFileError>
    where
        P: AsRef<Path> + Send,
    {
        match self.get(path, GetType::File).await {
            Ok(Some(Node::Directory(_))) => Err(GetFileError::UnexpectedDirectory),
            Ok(Some(Node::File(file))) => Ok(Some(file)),
            Ok(None) => Ok(None),
            Err(err) => Err(GetFileError::Get(err)),
        }
    }

    async fn get_file_default<P>(&self, path: P) -> Result<File<D, F>, GetFileError>
    where
        P: AsRef<Path> + Send,
    {
        match self.get_default(path, GetType::File).await {
            Ok(Node::Directory(_)) => Err(GetFileError::UnexpectedDirectory),
            Ok(Node::File(file)) => Ok(file),
            Err(err) => Err(GetFileError::Get(err)),
        }
    }
}
