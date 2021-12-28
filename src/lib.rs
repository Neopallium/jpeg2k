use anyhow::{anyhow, Result};

/// Magic bytes for JP2 RFC3745.
pub const JP2_RFC3745_MAGIC: &'static [u8] = &[
  0x00, 0x00, 0x00, 0x0c, 0x6a, 0x50, 0x20, 0x20, 0x0d, 0x0a, 0x87, 0x0a
];
/// Magic bytes for J2K Codestream.
pub const J2K_CODESTREAM_MAGIC: &'static [u8] = &[
  0xff, 0x4f, 0xff, 0x51
];

/// Supported Jpeg 2000 formats.
#[derive(Debug, Clone, Copy)]
pub enum J2KFormat {
  JP2,
  J2K,
}

/// Detect Jpeg 2000 format from magic bytes.
pub fn j2k_detect_format(buf: &[u8]) -> Result<J2KFormat> {
  if buf.starts_with(JP2_RFC3745_MAGIC) {
    Ok(J2KFormat::JP2)
  } else if buf.starts_with(J2K_CODESTREAM_MAGIC) {
    Ok(J2KFormat::J2K)
  } else {
    Err(anyhow!("Unknown format"))
  }
}

mod openjpeg;
pub use openjpeg::*;

#[cfg(feature = "bevy")]
mod bevy_loader;
#[cfg(feature = "bevy")]
pub use bevy_loader::*;
