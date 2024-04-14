//! # Jpeg 2000 image support.
//!
//! A safe wrapper of `openjpeg-sys` for loading/saving Jpeg 2000 images.
//!
//! ## Example: Convert a Jpeg 2000 image to a png image.
//!
//! ```rust
//! use image::DynamicImage;
//!
//! use jpeg2k::*;
//!
//! fn main() {
//!   // Load jpeg 2000 file from file.
//!   let jp2_image = Image::from_file("./assets/example.j2k")
//! 		.expect("Failed to load j2k file.");
//!
//!   // Convert to a `image::DynamicImage`
//!   let img: DynamicImage = jp2_image.try_into()?;
//!
//!   // Save as png file.
//!   img.save("out.png")?;
//! }
//! ```

/// File format detection.
pub mod format;
pub(crate) use format::*;

pub mod error;
pub(crate) use error::*;

#[cfg(feature = "openjpeg-sys")]
pub(crate) use openjpeg_sys as sys;

#[cfg(feature = "openjp2")]
pub(crate) mod sys {
  pub use openjp2::openjpeg::*;
}

impl From<J2KFormat> for sys::CODEC_FORMAT {
  fn from(format: J2KFormat) -> Self {
    use J2KFormat::*;
    match format {
      JP2 => sys::CODEC_FORMAT::OPJ_CODEC_JP2,
      J2K => sys::CODEC_FORMAT::OPJ_CODEC_J2K,
    }
  }
}

pub(crate) mod codec;
pub(crate) mod dump;
pub(crate) mod j2k_image;
pub(crate) mod stream;

pub use codec::*;
pub use dump::*;
pub(crate) use stream::*;

pub use self::j2k_image::*;

/// Image color space.
#[derive(Debug, Clone, Copy)]
pub enum ColorSpace {
  Unknown,
  Unspecified,
  SRGB,
  Gray,
  SYCC,
  EYCC,
  CMYK,
}

/// From `ColorSpace` to OpenJpeg `COLOR_SPACE`.
impl From<ColorSpace> for sys::COLOR_SPACE {
  fn from(color: ColorSpace) -> Self {
    use sys::COLOR_SPACE::*;
    use ColorSpace::*;
    match color {
      Unknown => OPJ_CLRSPC_UNKNOWN,
      Unspecified => OPJ_CLRSPC_UNSPECIFIED,
      SRGB => OPJ_CLRSPC_SRGB,
      Gray => OPJ_CLRSPC_GRAY,
      SYCC => OPJ_CLRSPC_SYCC,
      EYCC => OPJ_CLRSPC_EYCC,
      CMYK => OPJ_CLRSPC_CMYK,
    }
  }
}

/// From OpenJpeg `COLOR_SPACE` to `ColorSpace`.
impl From<sys::COLOR_SPACE> for ColorSpace {
  fn from(color: sys::COLOR_SPACE) -> Self {
    use sys::COLOR_SPACE::*;
    use ColorSpace::*;
    match color {
      OPJ_CLRSPC_UNKNOWN => Unknown,
      OPJ_CLRSPC_UNSPECIFIED => Unspecified,
      OPJ_CLRSPC_SRGB => SRGB,
      OPJ_CLRSPC_GRAY => Gray,
      OPJ_CLRSPC_SYCC => SYCC,
      OPJ_CLRSPC_EYCC => EYCC,
      OPJ_CLRSPC_CMYK => CMYK,
    }
  }
}
