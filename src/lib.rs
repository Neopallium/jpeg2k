pub mod format;
pub(crate) use format::*;

pub mod error;
pub(crate) use error::*;

mod openjpeg;
pub use openjpeg::*;

#[cfg(feature = "bevy")]
mod bevy_loader;
#[cfg(feature = "bevy")]
pub use bevy_loader::*;

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
