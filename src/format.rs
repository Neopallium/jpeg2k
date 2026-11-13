//! File format detection.
//!
//! Re-exports `openjp2`'s format types and detection functions, plus the
//! deprecated `jpeg2k` 0.10-era helpers.

pub use openjp2::{
  detect_format, detect_format_from_extension, detect_format_from_extension_os_str,
  detect_format_from_file, J2KFormat, JP2_MAGIC, JP2_RFC3745_MAGIC, J2K_CODESTREAM_MAGIC,
};

use openjp2::{Error, Result};

/// Detect Jpeg 2000 format from magic bytes.
///
/// Kept from `jpeg2k` 0.10: thin wrapper over
/// [`openjp2::detect_format`] mapping the string error into the typed
/// [`Error`].
#[deprecated(note = "use `jpeg2k::detect_format`")]
pub fn j2k_detect_format(buf: &[u8]) -> Result<J2KFormat> {
  openjp2::detect_format(buf).map_err(Error::Unsupported)
}

/// Detect Jpeg 2000 format from file extension.
///
/// Kept from `jpeg2k` 0.10: thin wrapper over
/// [`openjp2::detect_format_from_extension_os_str`].
#[deprecated(note = "use `jpeg2k::detect_format_from_extension_os_str`")]
pub fn j2k_detect_format_from_extension(ext: Option<&std::ffi::OsStr>) -> Result<J2KFormat> {
  openjp2::detect_format_from_extension_os_str(ext).map_err(Error::Unsupported)
}
