//! # Jpeg 2000 image support.
//!
//! Since version 0.11 this crate is a thin re-export of the safe
//! high-level API of [`openjp2`](https://crates.io/crates/openjp2) (a
//! pure-Rust port of OpenJPEG).  Most of `openjp2`'s public API is
//! re-exported at the crate root.  A small compatibility layer keeps the
//! `jpeg2k` 0.10 API surface working: [`DumpImage`] is a compat wrapper
//! over the `openjp2` equivalent (see its docs) and a few deprecated
//! type aliases ([`CodestreamInfo`], [`ImageComponent`], ...) map the
//! old names onto the `openjp2` types.  Everything else — including
//! [`Image`], [`DecodeParameters`], [`DecodeArea`], [`Error`] and
//! [`ColorSpace`] — comes straight from `openjp2`.
//!
//! ## Example: Convert a Jpeg 2000 image to a png image.
//!
//! ```no_run
//! use image::DynamicImage;
//!
//! use jpeg2k::*;
//!
//! fn main() {
//!   // Load jpeg 2000 file from file.
//!   let jp2_image = Image::from_file("./assets/example.j2k")
//!     .expect("Failed to load j2k file.");
//!
//!   // Convert to a `image::DynamicImage`
//!   let img: DynamicImage = (&jp2_image).try_into()
//!     .expect("Failed to convert image.");
//!
//!   // Save as png file.
//!   img.save("out.png").expect("Failed to save image.");
//! }
//! ```
//!
//! ## Migration from `jpeg2k` 0.10 and earlier
//!
//! - [`Error`] is now `openjp2`'s typed error enum (`Io`, `Decode`,
//!   `Encode`, `Unsupported`) instead of the old variants.
//! - [`ColorSpace`] is now `openjp2`'s enum as-is: the variants are
//!   renamed (`SRGB` → `Srgb`, `SYCC` → `Sycc`, `EYCC` → `Eycc`,
//!   `CMYK` → `Cmyk`) and `Unspecified` was removed.
//! - [`Image`] is now `openjp2::Image`; `save_as_file*`, `x_offset()`/
//!   `y_offset()`, `orig_width()`/`orig_height()` and
//!   [`Image::width`]/[`Image::height`] are provided by `openjp2`
//!   0.7.2+ with the `jpeg2k` 0.10 semantics (`width`/`height` report
//!   the decoded, possibly resolution-reduced component-plane size;
//!   `orig_width`/`orig_height` the full image extent).
//! - `ImageComponent` is now [`CoreImageComp`] (deprecated alias kept;
//!   its accessor methods are now inherent on `CoreImageComp`).
//!   `data()` returns `Option<&[i32]>`.
//! - `ImageFormat` was replaced by `image::ExtendedColorType` (deprecated
//!   alias kept, behind the `image` feature).
//! - The codestream index/info types come from `openjp2` with new field
//!   layouts; the deprecated `CodestreamMarker`/`CodestreamPacketInfo`/
//!   `CodestreamTilePartIndex`/`TileCodingParamInfo`/`CodestreamInfo`
//!   aliases map the old names to the new types.
//! - [`DumpImage`] keeps its 0.10 shape (`img` field, `decode(&self)`,
//!   `Result`-returning index/info getters) as a compat wrapper.
//! - The `openjpeg-sys` backend and the `openjpeg-sys`, `file-io` and
//!   `strict-mode` features were removed.  Strict decoding is now
//!   [`DecodeParameters::strict`]; file IO is always available.  The
//!   `serde` feature forwards to `openjp2/serde` (derives on
//!   `ImageData`/`ImagePixelData` and the other safe-API types).
//! - See `doc/migration-0.10-to-0.11.md` for the full migration guide.

// Re-export of the openjp2 public API (high-level safe API plus the
// low-level `Decoder`/`Encoder`/`Stream` types).
//
// Note: explicit re-exports instead of a glob so that the `image` *module*
// is not re-exported (its name collides with the `image` crate when a user
// writes `use jpeg2k::*;` together with `use image::...`).  The module's
// contents (`CoreImage`, `CoreImageComp`, ICC profiles) are re-exported
// at the crate root below.  `DumpImage` is intentionally not re-exported:
// the compat wrapper below shadows it.
pub use openjp2::{
  api, design_spec, stream, // modules
  ColorSpace, DecodeArea, DecodeError, DecodeLimits, DecodeParameters, EncodeParameters,
  EncoderParameters, Error, Image, ProgressionOrder, Result, // safe API
  ALL, IMG_INFO, J2K_MH_IND, J2K_MH_INFO,
  CoreImage, CoreImageComp, ICCProfile, ICCProfileMut, ICCProfileRef, CIE_LAB_BYTE_SIZE, // image model
  J2KFormat, JP2_RFC3745_MAGIC, JP2_MAGIC, J2K_CODESTREAM_MAGIC, // formats
  detect_format, detect_format_from_extension, detect_format_from_extension_os_str,
  detect_format_from_file, has_thread_support, StringWriter, // helpers
  Decoder, DecoderSettings, Encoder, HeaderImage, TileInfo, Stream, // codecs/streams
  CodestreamIndex, CodestreamTileIndex, CstrCompInfoV2, CstrInfoV2, CstrTccpInfoV2, MarkerInfo,
  PacketInfo, Poc, TpIndex, // codestream index/info
};
#[cfg(feature = "image")]
pub use openjp2::{ImageData, ImagePixelData};

/// File format detection.
pub mod format;
/// Errors.
pub mod error;

mod compat;
mod dump;

pub use dump::DumpImage;
#[allow(deprecated)]
pub use compat::{
  CodestreamInfo, CodestreamMarker, CodestreamPacketInfo, CodestreamTilePartIndex,
  ImageComponent, TileCodingParamInfo,
};
#[cfg(feature = "image")]
#[allow(deprecated)]
pub use compat::ImageFormat;
