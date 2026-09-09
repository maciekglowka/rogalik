#[derive(Debug)]
pub enum EngineError {
    NameConflict,
    InvalidResource,
    ResourceNotFound,
    GraphicsInternalError,
    GraphicsNotReady,
}
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameConflict => f.write_str("Name conflict"),
            Self::InvalidResource => f.write_str("Invalid resource"),
            Self::ResourceNotFound => f.write_str("Resource not found"),
            Self::GraphicsInternalError => f.write_str("Graphics internal error"),
            Self::GraphicsNotReady => f.write_str("Graphics not ready"),
        }
    }
}
impl std::error::Error for EngineError {}
