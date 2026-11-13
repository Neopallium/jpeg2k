# jpeg2k

JPEG 2000 image loader.

Since version 0.11 this crate is a thin wrapper over the safe high-level API of
[`openjp2`](https://crates.io/crates/openjp2) (a pure-Rust port of OpenJPEG).
Most of `openjp2`'s public API is re-exported at the crate root, and a small
compatibility layer keeps the `jpeg2k` 0.10 API surface working.

## Example: Convert a Jpeg 2000 image to a png image.

```rust
use image::DynamicImage;

use jpeg2k::*;

fn main() {
  // Load jpeg 2000 file from file.
  let jp2_image = Image::from_file("./assets/example.j2k")
    .expect("Failed to load j2k file.");

  // Convert to a `image::DynamicImage`
  let img: DynamicImage = (&jp2_image).try_into()?;

  // Save as png file.
  img.save("out.png")?;
}
```

## Features

- `image` (default): `image`-crate interop (`get_pixels`, `ImageData`,
  `ImagePixelData` and the `DynamicImage` conversions).
- `threads` (default): multi-threaded decode/encode (`openjp2`'s `rayon`
  feature).
- `serde`: `serde` derives for `ImageData`/`ImagePixelData` (and the
  other `openjp2` safe-API types), forwarded to `openjp2`'s `serde`
  feature.

## Migration from `jpeg2k` 0.10 and earlier

The `jpeg2k` crate is now built on top of `openjp2` 0.7.2's safe Rust API.
The old `openjpeg-sys` C-ABI backend and the `openjpeg-sys`, `file-io` and
`strict-mode` features were removed (file IO is always available now; strict
decoding is `DecodeParameters::strict(true)`).

Most of the 0.10 API keeps working; the notable changes are:

- `Error` is now `openjp2`'s typed error enum (`Io`, `Decode`, `Encode`,
  `Unsupported`) instead of the old variants.
- `ColorSpace` is now `openjp2`'s enum as-is (variants renamed to
  `Srgb`/`Sycc`/`Eycc`/`Cmyk`; `Unspecified` removed).
- `Image` is now `openjp2::Image`.  `save_as_file*`, `x_offset()`/
  `y_offset()`, `orig_width()`/`orig_height()` and `width()`/`height()`
  are provided by `openjp2` 0.7.2+ with the `jpeg2k` 0.10 semantics
  (`width()`/`height()` report the decoded, possibly resolution-reduced
  component-plane size; `orig_width()`/`orig_height()` the full extent).
- `ImageComponent` is now `CoreImageComp` (deprecated alias kept; the
  accessor methods are now inherent on `CoreImageComp`).  `data()`
  returns `Option<&[i32]>`.
- `ImageFormat` was replaced by `image::ExtendedColorType` (deprecated
  alias kept, behind the `image` feature).
- The codestream index/info types come from `openjp2` with new field
  layouts (`MarkerInfo`, `PacketInfo`, `TpIndex`, `CstrInfoV2`, ...); the
  deprecated `CodestreamMarker`/`CodestreamPacketInfo`/
  `CodestreamTilePartIndex`/`TileCodingParamInfo`/`CodestreamInfo`
  aliases map the old names onto the new types.
- `DumpImage` keeps its 0.10 shape (`img` field, `decode(&self)`, the
  index/info getters) as a compat wrapper.

The full breaking-change details and migration steps are in the
[version migration guide](doc/migration-0.10-to-0.11.md).
