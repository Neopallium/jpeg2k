# Migration guide: `jpeg2k` 0.10 → 0.11

Version 0.11 rebuilds `jpeg2k` on top of the safe Rust API of
[`openjp2`](https://crates.io/crates/openjp2) 0.7.2 (a pure-Rust port of
OpenJPEG).  The old C-ABI `openjpeg-sys` backend is gone, and `jpeg2k`
is now a thin re-export of `openjp2`'s public API plus a small
compatibility layer (`DumpImage` and a handful of deprecated type
aliases).

Most 0.10 code compiles unchanged — the examples and benches in this
repository do — but the following areas are breaking.  They are ordered
by how likely they are to affect you.

## 1. Cargo features

| 0.10 feature | 0.11 | Action |
|---|---|---|
| `openjpeg-sys` (in `default`) | removed | delete from your feature list |
| `openjp2` (0.6.1 backend selector) | removed (openjp2 is now mandatory) | delete |
| `file-io` (in `default`) | removed (file IO is always available) | delete |
| `strict-mode` | removed (strict decoding is now `DecodeParameters::strict(true)`, always available) | delete |
| `serde` | kept; now forwards to `openjp2/serde` (derives on `ImageData`, `ImagePixelData`, `DecodeArea`, `DecodeLimits`, `ColorSpace`, `ProgressionOrder`) | keep |
| *(new)* `openjpeg-version-string` | forwards to `openjp2/openjpeg-version-string` | see §9 |

`default` is now `["image", "threads"]` (was `["image", "threads",
"file-io", "openjpeg-sys"]`).

## 2. Error handling

`Error` is now `openjp2`'s typed enum instead of the 0.10 variants:

```rust
// 0.10
pub enum Error {
  UnsupportedComponentsError(u32),
  UnsupportedColorSpaceError(ColorSpace),
  CreateCodecError(String),
  CodecError(String),
  UnknownFormatError(String),
  FileNotFoundError(String),
  BadFilenameError(String),
  NullPointerError(&'static str),
  Other(#[from] anyhow::Error),
}

// 0.11 (re-exported from openjp2)
pub enum Error {
  Io(std::io::Error),
  Decode(DecodeError), // message + optional marker/box/position
  Encode(String),
  Unsupported(String),
}
```

Rough mapping of the old variants:

- `UnsupportedComponentsError` / `UnsupportedColorSpaceError` → `Error::Unsupported(...)`
- `CreateCodecError` / `CodecError` → `Error::Decode(...)` (decode) or `Error::Encode(...)` (encode)
- `UnknownFormatError` → `Error::Unsupported(...)` (format detection)
- `FileNotFoundError` / IO failures → `Error::Io(e)`
- `BadFilenameError` / `NullPointerError` → gone (no C strings or raw pointers anymore)
- `Other(#[from] anyhow::Error)` → gone; **`?` no longer converts an `anyhow::Error` into `jpeg2k::Result`** — map it explicitly (e.g. `map_err(|e| Error::Unsupported(e.to_string()))`).

Any `match` over the old variants needs rewriting against the new enum.

## 3. `ColorSpace`

`jpeg2k::ColorSpace` is now `openjp2::ColorSpace` as-is:

- renamed: `SRGB` → `Srgb`, `SYCC` → `Sycc`, `EYCC` → `Eycc`, `CMYK` → `Cmyk`
- removed: `Unspecified` (folded into `Unknown`)

```rust
// 0.10
match img.color_space() {
  ColorSpace::SRGB | ColorSpace::Gray => (),
  cs => return Err(Error::UnsupportedColorSpaceError(cs)),
}

// 0.11
match img.color_space() {
  ColorSpace::Srgb | ColorSpace::Gray => (),
  cs => return Err(Error::Unsupported(format!("color space {cs:?}"))),
}
```

## 4. `Image`

`Image` is now `openjp2::Image`.  The 0.10 method surface is fully
restored, including the exact dimension semantics:

- `width()` / `height()` — decoded size (first component plane, reduced
  by the resolution factor and decode area)
- `orig_width()` / `orig_height()` — full image extent
- `x_offset()` / `y_offset()` — image origin
- `save_as_file()` / `save_as_file_with()` — format detected from the
  file extension

Changes to be aware of:

- `components()` returns `&[CoreImageComp]` instead of `&[ImageComponent]` — see §5.
- `color_space()` returns the renamed enum — see §3.
- `get_pixels()` (and `ImageData`/`ImagePixelData`) is now gated behind
  the `image` feature (default on).  It is also stricter: subsampled
  images (component planes of different sizes) and truncated/partial
  decodes return an error instead of silently zipping/truncating.
- `from_file*` is no longer `file-io`-gated (always available).

Additions: `Clone`, `Send`, `Sync`, `Debug`, `encode()`/`encode_jp2()`/
`encode_to_writer()`, `from_dynamic_image()`, `from_core_image()`/
`into_core_image()`.

## 5. `ImageComponent` → `CoreImageComp`

The component type is now `openjp2::CoreImageComp`; a deprecated alias
`ImageComponent` keeps type paths compiling.  The 0.10 accessor methods
are now inherent on `CoreImageComp`: `width()`, `height()`,
`precision()`, `bpp()`, `is_alpha()`, `is_signed()`.

Remaining breaks:

- `data()` returns `Option<&[i32]>` instead of `&[i32]` — use
  `.unwrap()`/`as_deref()` or the public `data: Vec<i32>` field.
- `data_u8()`/`data_u16()` return `impl Iterator<Item = u8/u16>` instead
  of `Box<dyn Iterator<...>>`, and are gated behind the `image` feature.
- `CoreImageComp` has no `Debug` impl (0.10's `ImageComponent` did).

```rust
// 0.10
let d: &[i32] = comp.data();

// 0.11
let d: &[i32] = comp.data().expect("decoded component data"); // or &comp.data
```

## 6. `ImageData` / `ImagePixelData` / `ImageFormat`

- `ImageData::format` is now `image::ExtendedColorType` instead of the
  0.10 `ImageFormat` enum.  A deprecated `ImageFormat` alias maps the
  name, but exhaustive `match`es break (`ExtendedColorType` has more
  variants, e.g. `A8`, `Rgb32F`, `Unknown`).
- `ImageData` and `ImagePixelData` are gated behind the `image` feature
  (default on).
- `ImagePixelData` keeps the same variant names (`L8`, `La8`, `Rgb8`,
  ...) and gained `len()`/`is_empty()`.
- `serde` derives are restored via the `serde` feature (§1).

## 7. `DecodeArea`

`Default`, `Copy`, `Clone`, `new()` and `FromStr` (`"x0:y0:x1:y1"`) are
restored, but `FromStr::Err` changed from `anyhow::Error` to
`std::num::ParseIntError`.  `.parse::<DecodeArea>()?` still works
wherever `ParseIntError` converts (it does for `anyhow`); code that
names the error type must change.

## 8. Codestream index/info types

These are now `openjp2`'s types with new field layouts:

| 0.10 | 0.11 |
|---|---|
| `CodestreamIndex` | `openjp2::CodestreamIndex` (re-exported) — `markers`/`tiles` are now public fields; `markers()`/`tile_indices()` methods are gone; new `packets()`/`layer_byte_thresholds()` |
| `CodestreamTileIndex` | `openjp2::CodestreamTileIndex` — `tp_index`/`markers`/`packet_index` public fields instead of methods |
| `CodestreamTilePartIndex` | deprecated alias for `TpIndex` |
| `CodestreamPacketInfo` | deprecated alias for `PacketInfo` |
| `CodestreamMarker` | deprecated alias for `MarkerInfo` |
| `TileCodingParamInfo` | deprecated alias for `CstrCompInfoV2` |
| `CodestreamInfo` | deprecated alias for `CstrInfoV2` |
| `TileInfo` | now `openjp2`'s tile-part header summary (different shape; only returned by the new low-level `Decoder::read_tile_header`) |

All of these now derive `Debug` (upstream addition), so `{:#?}` printing
keeps working, with different field names.

## 9. Behavior notes

- **Encoded output**: `openjp2` is heavily tested against the OpenJPEG C
  code (C-parity tests in CI).  Encoded files are byte-identical to the C
  reference except for the default COM ("comment") marker, which records
  the encoder's program name.  To make the output fully byte-identical,
  enable the `openjpeg-version-string` feature (writes the upstream
  OpenJPEG product/version string into the COM marker instead).
- **Threading**: the `threads` feature now enables rayon-based
  parallelism in `openjp2` instead of configuring `opj_codec_set_threads`.
- **Build environment**: no C toolchain or system libraries are needed
  anymore (pure Rust).
- **MSRV**: no `rust-version` is declared, but the new dependency
  requires roughly Rust 1.71+.
- **Resource limits**: `DecodeParameters::limits(DecodeLimits::...)` is
  available for untrusted input (image/tile size, memory budget,
  metadata length caps).

## 10. Format detection

- `j2k_detect_format` / `j2k_detect_format_from_extension` are kept but
  deprecated; use `detect_format` / `detect_format_from_extension_os_str`
  (or `detect_format_from_file`) instead.  Their error is now
  `Error::Unsupported` (was `UnknownFormatError`).
- `J2KFormat` gained a `JPT` variant (add a wildcard arm to exhaustive
  matches).
- Detection additionally accepts the bare JP2 signature magic
  (`0D 0A 87 0A`).

## 11. `DumpImage`

The compat wrapper keeps the 0.10 shape (`pub img: Image`,
`from_bytes*`/`from_file*`, `decode(&self)`, `Result`-returning
`get_codestream_index()`/`get_codestream_info()`).  Differences:

- The getters return `Result<CodestreamIndex>`/`Result<CstrInfoV2>`
  (the `CodestreamInfo` alias keeps the old name compiling; the types
  are the upstream ones — see §8).
- The struct no longer has a lifetime parameter (`DumpImage` instead of
  `DumpImage<'a>`).
- `from_file*` is no longer `file-io`-gated.

## Deprecated aliases still working (with warnings)

`ImageComponent`, `ImageFormat`, `CodestreamMarker`,
`CodestreamPacketInfo`, `CodestreamTilePartIndex`, `TileCodingParamInfo`,
`CodestreamInfo`, `j2k_detect_format`, `j2k_detect_format_from_extension`.
