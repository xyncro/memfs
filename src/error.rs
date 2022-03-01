use miette::Diagnostic;
use thiserror::Error;

// =============================================================================

// Directory Errors

// -----------------------------------------------------------------------------

// Get

#[derive(Debug, Diagnostic, Error)]
pub enum GetError {
    #[diagnostic(code(directory::open::file), help("check the supplied path"))]
    #[error("path indicated a directory, but a file was found")]
    UnexpectedFile,
    #[diagnostic(code(directory::open::orphan), help("check the supplied path"))]
    #[error("path indicated parent directory, but current directory has no parent")]
    UnexpectedOrphan,
    #[diagnostic(code(directory::open::prefix), help("check the supplied path"))]
    #[error("path contained a prefix, which is not supported")]
    UnexpectedPrefix,
    #[diagnostic(code(directory::open::root), help("check the supplied path"))]
    #[error("path was an absolute (root) path, but the directory is not a root directory")]
    UnexpectedRoot,
    #[diagnostic(code(directory::open::intermediate), help("check the supplied path"))]
    #[error("the endpoint does not exist")]
    EndpointNotFound,
    #[diagnostic(code(directory::open::intermediate), help("check the supplied path"))]
    #[error("an intermediate directory does not exist")]
    IntermediateNotFound,
}
