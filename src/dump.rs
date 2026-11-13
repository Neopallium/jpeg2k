//! `DumpImage`: header/decode driver for the codestream index/info
//! (kept from `jpeg2k` 0.10; facade over the `openjp2` low-level API).

use std::cell::RefCell;
use std::path::Path;

use openjp2::{CodestreamIndex, CstrInfoV2, DecodeParameters, Decoder, DecoderSettings, Error, Image, Result, Stream};

pub struct DumpImage {
  decoder: RefCell<Decoder>,
  stream: RefCell<Stream>,
  /// The codestream header image (no pixel data until `decode()`).
  pub img: Image,
}

impl DumpImage {
  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    Self::from_bytes_with(buf, DecodeParameters::default())
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    Self::from_file_with(path, DecodeParameters::default())
  }

  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    Self::from_stream(buf, params)
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  pub fn from_file_with<P: AsRef<Path>>(path: P, params: DecodeParameters) -> Result<Self> {
    let buf = std::fs::read(path)?;
    Self::from_stream(&buf, params)
  }

  fn from_stream(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let format = openjp2::detect_format(buf).map_err(Error::Unsupported)?;
    let mut stream = Stream::from_bytes(buf.to_vec());
    let mut decoder = Decoder::new(format)?;
    decoder.apply_settings(&DecoderSettings {
      layers: params.layers,
      reduce: params.reduce,
      ignore_pclr_cmap_cdef: params.ignore_pclr_cmap_cdef,
      threads: params.threads,
      ..Default::default()
    })?;
    decoder.set_strict(params.strict)?;
    decoder.set_limits(params.limits)?;

    let header = decoder.read_header(&mut stream)?;
    let img = Image::from_core_image(header.into_core_image());

    Ok(Self {
      decoder: RefCell::new(decoder),
      stream: RefCell::new(stream),
      img,
    })
  }

  /// Decode the tile data (needed to populate the codestream index with
  /// the tile markers).
  pub fn decode(&self) -> Result<()> {
    let mut decoder = self.decoder.borrow_mut();
    let mut stream = self.stream.borrow_mut();
    decoder.decode(&mut stream)?;
    decoder.end_decompress(&mut stream)?;
    Ok(())
  }

  /// The codestream index (tile-parts, markers, packets).
  pub fn get_codestream_index(&self) -> Result<CodestreamIndex> {
    self
      .decoder
      .borrow()
      .cstr_index()
      .ok_or_else(|| Error::Decode("Failed to get codestream index".into()))
  }

  /// The codestream header info (v2).
  pub fn get_codestream_info(&self) -> Result<CstrInfoV2> {
    self
      .decoder
      .borrow()
      .cstr_info()
      .ok_or_else(|| Error::Decode("Failed to get codestream info".into()))
  }
}
