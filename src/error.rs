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

// Directory Errors - Find

#[derive(Debug, Diagnostic, Error)]
pub enum FindError {
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
