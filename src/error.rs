use miette::Diagnostic;
use thiserror::Error;

// =============================================================================

// Directory Errors

// -----------------------------------------------------------------------------

// Directory Errors - Get

#[derive(Debug, Diagnostic, Error)]
pub enum GetError {
    #[diagnostic(code(directory::get), help("check the contents of the directory"))]
    #[error("expected file but instead found directory")]
    ExpectedFile,
    #[diagnostic(code(directory::get), help("check the contents of the directory"))]
    #[error("expected directory but instead found file")]
    ExpectedDir,
}

// -----------------------------------------------------------------------------

// Directory Errors - Get Path

#[derive(Debug, Diagnostic, Error)]
pub enum GetPathError {
    #[diagnostic(code(directory::find), help("check the contents of the tree"))]
    #[error("intermediate file '{name}' in path")]
    IntermediateFile { name: String },
    #[diagnostic(code(directory::get), help("check the contents of the directory"))]
    #[error("expected file but instead found directory")]
    ExpectedFile,
    #[diagnostic(code(directory::get), help("check the contents of the directory"))]
    #[error("expected directory but instead found file")]
    ExpectedDir,
}

#[derive(Debug, Diagnostic, Error)]
pub enum OpenError {
    #[diagnostic(code(directory::open), help("check the supplied path"))]
    #[error("path contained a prefix, which is not supported")]
    UnexpectedPrefix,
    #[diagnostic(code(directory::get), help("check the supplied path"))]
    #[error("path was an absolute (root) path, but the directory is not a root directory")]
    UnexpectedRoot,
}
