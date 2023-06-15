use thiserror::Error;

/// TLDExtractError
pub type Result<T> = std::result::Result<T, TLDExtractError>;

/// TLDExtractError Enum
#[derive(Debug, Error)]
pub enum TLDExtractError {
  /// invalid domain
  #[error("invalid domain: '{0}'")]
  DomainError(String),
  /// suffix list error
  #[error("suffix list error: '{0}'")]
  SuffixListError(String),
  /// Parse Error
  #[error(transparent)]
  ParseError(#[from] reqwest::Error),
  /// Io Error
  #[error(transparent)]
  Io(#[from] std::io::Error),
}
