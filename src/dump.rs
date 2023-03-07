#[cfg(feature = "file-io")]
use std::path::Path;

use super::*;

pub struct DumpImage<'a> {
  decoder: Decoder<'a>,
  pub img: Image,
}

impl<'a> DumpImage<'a> {
  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes(buf: &'a [u8]) -> Result<Self> {
    let stream = Stream::from_bytes(buf)?;
    Self::from_stream(stream, Default::default())
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  #[cfg(feature = "file-io")]
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let stream = Stream::from_file(path)?;
    Self::from_stream(stream, Default::default())
  }

  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes_with(buf: &'a [u8], params: DecodeParameters) -> Result<Self> {
    let stream = Stream::from_bytes(buf)?;
    Self::from_stream(stream, params)
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  #[cfg(feature = "file-io")]
  pub fn from_file_with<P: AsRef<Path>>(path: P, params: DecodeParameters) -> Result<Self> {
    let stream = Stream::from_file(path)?;
    Self::from_stream(stream, params)
  }

  fn from_stream(stream: Stream<'a>, mut params: DecodeParameters) -> Result<Self> {
    let decoder = Decoder::new(stream)?;
    decoder.setup(&mut params)?;

    let img = decoder.read_header()?;

    Ok(Self { decoder, img })
  }

  pub fn decode(&self) -> Result<()> {
    self.decoder.decode(&self.img)
  }

  pub fn get_codestream_index(&self) -> Result<CodestreamIndex> {
    self.decoder.get_codestream_index()
  }

  pub fn get_codestream_info(&self) -> Result<CodestreamInfo> {
    self.decoder.get_codestream_info()
  }
}
