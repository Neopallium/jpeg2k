//! Deprecated type aliases mapping the old `jpeg2k` 0.10 names onto the
//! `openjp2` equivalents.

/// Alias kept from `jpeg2k` 0.10 for [`openjp2::CoreImageComp`].
#[deprecated(note = "use `CoreImageComp`")]
pub type ImageComponent = openjp2::CoreImageComp;

/// Alias for the `ImageData::format` field type, replacing the removed
/// `jpeg2k` 0.10 `ImageFormat` enum (same variant names for the 8/16-bit
/// L/La/Rgb/Rgba layouts).
#[cfg(feature = "image")]
#[deprecated(note = "use `image::ExtendedColorType`")]
pub type ImageFormat = ::image::ExtendedColorType;

/// Alias kept from `jpeg2k` 0.10 for [`openjp2::TpIndex`].
#[deprecated(note = "use `TpIndex`")]
pub type CodestreamTilePartIndex = openjp2::TpIndex;

/// Alias kept from `jpeg2k` 0.10 for [`openjp2::PacketInfo`].
#[deprecated(note = "use `PacketInfo`")]
pub type CodestreamPacketInfo = openjp2::PacketInfo;

/// Alias kept from `jpeg2k` 0.10 for [`openjp2::MarkerInfo`].
#[deprecated(note = "use `MarkerInfo`")]
pub type CodestreamMarker = openjp2::MarkerInfo;

/// Alias kept from `jpeg2k` 0.10 for [`openjp2::CstrCompInfoV2`].
#[deprecated(note = "use `CstrCompInfoV2`")]
pub type TileCodingParamInfo = openjp2::CstrCompInfoV2;

/// Alias kept from `jpeg2k` 0.10 for [`openjp2::CstrInfoV2`].
#[deprecated(note = "use `CstrInfoV2`")]
pub type CodestreamInfo = openjp2::CstrInfoV2;
