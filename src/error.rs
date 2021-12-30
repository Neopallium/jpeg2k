use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Unsupported components/color space")]
  UnsupportedComponentsError(u32),

  #[error("Failed to create codec: {0}")]
  CreateCodecError(String),

  #[error("Codec failed to encode/decode: {0}")]
  CodecError(String),

  #[error("Unknown format: {0}")]
  UnknownFormatError(String),

  #[error("File not found: {0}")]
  FileNotFoundError(String),

  #[error("Bad filename: {0}")]
  BadFilenameError(String),

  #[error("Null pointer from openjpeg-sys")]
  NullPointerError(&'static str),

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
