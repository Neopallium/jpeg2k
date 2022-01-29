pub(crate) use openjpeg_sys as sys;

use super::*;

impl From<J2KFormat> for sys::CODEC_FORMAT {
  fn from(format: J2KFormat) -> Self {
    match format {
      J2KFormat::JP2 => sys::CODEC_FORMAT::OPJ_CODEC_JP2,
      J2KFormat::J2K => sys::CODEC_FORMAT::OPJ_CODEC_J2K,
    }
  }
}

pub(crate) mod codec;
pub(crate) mod dump;
pub(crate) mod image;
pub(crate) mod stream;

pub use codec::*;
pub use dump::*;
pub(crate) use stream::*;

pub use self::image::*;
