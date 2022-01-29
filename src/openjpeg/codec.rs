use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::ptr;

use log::{Level, log_enabled};

use super::*;

/// The area of the source image to decode.
///
/// This is useful for loading a small part of a
/// very large image.
///
/// ```rust
/// let area = DecodeArea::new(10, 10, 200, 200);
///
/// // or from a string:
/// let area: DecodeArea = "10:10:200:200".parse()?;
/// let area = DecodeArea::from_str("10:10:200:200")?;
/// ```
#[derive(Default, Clone, Copy)]
pub struct DecodeArea {
  start_x: u32,
  start_y: u32,
  end_x: u32,
  end_y: u32,
}

impl std::str::FromStr for DecodeArea {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let dim = s.splitn(4, ":")
      .map(|v| v.parse::<u32>())
      .collect::<Result<Vec<u32>, _>>()?;
    Ok(Self {
      start_x: dim.get(0).copied().unwrap_or(0),
      start_y: dim.get(1).copied().unwrap_or(0),
      end_x: dim.get(2).copied().unwrap_or(0),
      end_y: dim.get(3).copied().unwrap_or(0),
    })
  }
}

impl DecodeArea {
  pub fn new(start_x: u32, start_y: u32, end_x: u32, end_y: u32) -> Self {
    Self {
      start_x, start_y,
      end_x, end_y,
    }
  }
}

pub struct DecodeParameters {
  params: sys::opj_dparameters,
  area: Option<DecodeArea>,
  strict: bool,
}

impl Default for DecodeParameters {
  fn default() -> Self {
    let params = unsafe {
      let mut ptr = std::mem::zeroed::<sys::opj_dparameters>();
      sys::opj_set_default_decoder_parameters(&mut ptr as *mut _);
      ptr
    };
    Self {
      params,
      area: Default::default(),
      strict: false,
    }
  }
}

impl DecodeParameters {
  pub fn new() -> Self {
    Default::default()
  }

  /// How much to reduce the image's resolution.
  ///
  /// If `reduce == 0`, image is decoded to the full resolution.  This is the default.
  /// If `reduce > 0`, then original dimension divided by 2^(reduce)
  pub fn reduce(mut self, reduce: u32) -> Self {
    self.params.cp_reduce = reduce;
    self
  }

  /// Enable/disable strict decoing mode.
  ///
  /// If disabled then progressive downloading is supported (truncated codestreams).  This is the default.
  /// If enabled then partial/truncated codestreams will return an error.
  pub fn strict(mut self, strict: bool) -> Self {
    self.strict = strict;
    self
  }

  /// The number of quality layers to decode.
  ///
  /// If there are less quality layers than the specified number,
  /// all the quality layers are decoded.
  /// 
  /// If `layers == 0`, all the quality layers are decoded.  This is the default.
  /// If `layers > 0`, then only the first `layers` layers are decoded.
  pub fn layers(mut self, layers: u32) -> Self {
    self.params.cp_layer = layers;
    self
  }

  /// The area to decode.
  ///
  /// If `area == None`, then the whole image will be decoded.  This is the defult.
  pub fn decode_area(mut self, area: Option<DecodeArea>) -> Self {
    self.area = area;
    self
  }

  pub(crate) fn as_ptr(&mut self) -> &mut sys::opj_dparameters {
    &mut self.params
  }
}

pub struct EncodeParameters(sys::opj_cparameters);

impl Default for EncodeParameters {
  fn default() -> Self {
    Self(unsafe {
      let mut ptr = std::mem::zeroed::<sys::opj_cparameters>();
      sys::opj_set_default_encoder_parameters(&mut ptr as *mut _);
      ptr
    })
  }
}

pub struct CodestreamTilePartIndex(pub(crate) sys::opj_tp_index_t);

impl std::fmt::Debug for CodestreamTilePartIndex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CodestreamTilePartIndex")
      .field("start_pos", &self.0.start_pos)
      .field("end_header", &self.0.end_header)
      .field("end_pos", &self.0.end_pos)
      .finish()
  }
}

pub struct CodestreamPacketInfo(pub(crate) sys::opj_packet_info_t);

impl std::fmt::Debug for CodestreamPacketInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CodestreamPacketInfo")
      .field("start_pos", &self.0.start_pos)
      .field("end_ph_pos", &self.0.end_ph_pos)
      .field("end_pos", &self.0.end_pos)
      .field("disto", &self.0.disto)
      .finish()
  }
}

pub struct CodestreamMarker(pub(crate) sys::opj_marker_info_t);

impl std::fmt::Debug for CodestreamMarker {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CodestreamMarker")
      .field("type", &self.0.type_)
      .field("pos", &self.0.pos)
      .field("len", &self.0.len)
      .finish()
  }
}

pub struct TileCodingParamInfo(ptr::NonNull<sys::opj_tccp_info_t>);

impl std::fmt::Debug for TileCodingParamInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let info = self.as_ref();
    f.debug_struct("TileCodingParamInfo")
      .field("compno", &info.compno)
      .field("csty", &info.csty)
      .field("numresolutions", &info.numresolutions)
      .field("cblkw", &info.cblkw)
      .field("cblkh", &info.cblkh)
      .field("cblksty", &info.cblksty)
      .field("qmfbid", &info.qmfbid)
      .field("qntsty", &info.qntsty)
      .field("stepsizes_mant", &info.stepsizes_mant)
      .field("stepsizes_expn", &info.stepsizes_expn)
      .field("numgbits", &info.numgbits)
      .field("roishift", &info.roishift)
      .field("prcw", &info.prcw)
      .field("prch", &info.prch)
      .finish()
  }
}

impl TileCodingParamInfo {
  fn as_ref(&self) -> &sys::opj_tccp_info_t {
    unsafe { &(*self.0.as_ref()) }
  }
}

pub struct TileInfo(pub(crate) sys::opj_tile_info_v2_t);

impl std::fmt::Debug for TileInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TileInfo")
      .field("tileno", &self.0.tileno)
      .field("csty", &self.0.csty)
      .field("prg", &self.0.prg)
      .field("numlayers", &self.0.numlayers)
      .field("mct", &self.0.mct)
      .field("tccp_info", &self.tccp_info())
      .finish()
  }
}

impl TileInfo {
  fn tccp_info(&self) -> Option<TileCodingParamInfo> {
    ptr::NonNull::new(self.0.tccp_info)
      .map(|info| TileCodingParamInfo(info))
  }
}

pub struct CodestreamTileIndex(pub(crate) sys::opj_tile_index_t);

impl std::fmt::Debug for CodestreamTileIndex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CodestreamTileIndex")
      .field("tileno", &self.0.tileno)
      .field("nb_tps", &self.0.nb_tps)
      .field("current_nb_tps", &self.0.current_nb_tps)
      .field("current_tpsno", &self.0.current_tpsno)
      .field("tp_index", &self.tile_parts())
      .field("marknum", &self.0.marknum)
      .field("marker", &self.markers())
      .field("maxmarknum", &self.0.maxmarknum)
      .field("nb_packet", &self.0.nb_packet)
      .field("packet_info", &self.packets())
      .finish()
  }
}

impl CodestreamTileIndex {
  /// Tile part index.
  pub fn tile_parts(&self) -> &[CodestreamTilePartIndex] {
    let num = self.0.nb_tps;
    unsafe { std::slice::from_raw_parts(self.0.tp_index as *mut CodestreamTilePartIndex, num as usize) }
  }

  /// Tile markers.
  pub fn markers(&self) -> &[CodestreamMarker] {
    let num = self.0.marknum;
    unsafe { std::slice::from_raw_parts(self.0.marker as *mut CodestreamMarker, num as usize) }
  }

  /// Codestream packet info.
  pub fn packets(&self) -> &[CodestreamPacketInfo] {
    let num = self.0.nb_packet;
    unsafe { std::slice::from_raw_parts(self.0.packet_index as *mut CodestreamPacketInfo, num as usize) }
  }
}

pub struct CodestreamIndex(ptr::NonNull<sys::opj_codestream_index_t>);

impl Drop for CodestreamIndex {
  fn drop(&mut self) {
    unsafe {
      sys::opj_destroy_cstr_index(&mut self.0.as_ptr());
    }
  }
}

impl std::fmt::Debug for CodestreamIndex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let idx = self.as_ref();
    f.debug_struct("CodestreamIndex")
      .field("main_head_start", &idx.main_head_start)
      .field("main_head_end", &idx.main_head_end)
      .field("codestream_size", &idx.codestream_size)
      .field("marknum", &idx.marknum)
      .field("marker", &self.markers())
      .field("maxmarknum", &idx.maxmarknum)
      .field("nb_of_tiles", &idx.nb_of_tiles)
      .field("tile_index", &self.tile_indices())
      .finish()
  }
}

impl CodestreamIndex {
  fn as_ref(&self) -> &sys::opj_codestream_index_t {
    unsafe { &(*self.0.as_ref()) }
  }

  /// Codestream markers.
  pub fn markers(&self) -> &[CodestreamMarker] {
    let idx = self.as_ref();
    let num = idx.marknum;
    unsafe { std::slice::from_raw_parts(idx.marker as *mut CodestreamMarker, num as usize) }
  }

  /// Codestream tile indices.
  pub fn tile_indices(&self) -> &[CodestreamTileIndex] {
    let idx = self.as_ref();
    let num = idx.nb_of_tiles;
    unsafe { std::slice::from_raw_parts(idx.tile_index as *mut CodestreamTileIndex, num as usize) }
  }
}

pub struct CodestreamInfo(ptr::NonNull<sys::opj_codestream_info_v2_t>);

impl Drop for CodestreamInfo {
  fn drop(&mut self) {
    unsafe {
      sys::opj_destroy_cstr_info(&mut self.0.as_ptr());
    }
  }
}

impl std::fmt::Debug for CodestreamInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let info = self.as_ref();
    let tile_info = if info.tile_info.is_null() {
      TileInfo(info.m_default_tile_info)
    } else {
      TileInfo(unsafe { *info.tile_info })
    };
    f.debug_struct("CodestreamInfo")
      .field("tx0", &info.tx0)
      .field("ty0", &info.ty0)
      .field("tdx", &info.tdx)
      .field("tdy", &info.tdy)
      .field("tw", &info.tw)
      .field("th", &info.th)
      .field("nbcomps", &info.nbcomps)
      .field("tile_info", &tile_info)
      .finish()
  }
}

impl CodestreamInfo {
  fn as_ref(&self) -> &sys::opj_codestream_info_v2_t {
    unsafe { &(*self.0.as_ref()) }
  }
}

pub(crate) struct Codec {
  codec: ptr::NonNull<sys::opj_codec_t>,
}

impl Drop for Codec {
  fn drop(&mut self) {
    unsafe {
      sys::opj_destroy_codec(self.codec.as_ptr());
    }
  }
}

extern "C" fn log_info(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    log::info!("{:?}", CStr::from_ptr(msg).to_string_lossy());
  }
}

extern "C" fn log_warn(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    log::warn!("{:?}", CStr::from_ptr(msg).to_string_lossy());
  }
}

extern "C" fn log_error(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    log::error!("{:?}", CStr::from_ptr(msg).to_string_lossy());
  }
}

impl Codec {
  fn new(fmt: J2KFormat, is_decoder: bool) -> Result<Self> {
    let format: sys::CODEC_FORMAT = fmt.into();
    let ptr = unsafe {
      if is_decoder {
        ptr::NonNull::new(sys::opj_create_decompress(format))
      } else {
        ptr::NonNull::new(sys::opj_create_compress(format))
      }
    };
    if let Some(ptr) = ptr {
      let null = ptr::null_mut();
      unsafe {
        if log_enabled!(Level::Info) {
          sys::opj_set_info_handler(ptr.as_ptr(), Some(log_info), null);
        }
        if log_enabled!(Level::Warn) {
          sys::opj_set_warning_handler(ptr.as_ptr(), Some(log_warn), null);
        }
        sys::opj_set_error_handler(ptr.as_ptr(), Some(log_error), null);

        #[cfg(feature = "threads")]
        if sys::opj_has_thread_support() == 1{
          let num_cpus = sys::opj_get_num_cpus();
          if sys::opj_codec_set_threads(ptr.as_ptr(), num_cpus) != 1 {
            log::warn!("Failed to set number of threads: {:?}", num_cpus);
          }
        }
      }

      Ok(Self {
        codec: ptr,
      })
    } else {
      Err(Error::CreateCodecError(format!("Codec not supported: {:?}", fmt)))
    }
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_codec_t {
    self.codec.as_ptr()
  }
}

pub(crate) struct Decoder<'a> {
  codec: Codec,
  stream: Stream<'a>,
}

impl<'a> Decoder<'a> {
  pub(crate) fn new(stream: Stream<'a>) -> Result<Self> {
    assert!(stream.is_input());
    let fmt = stream.format();
    let codec = Codec::new(fmt, true)?;
    Ok(Self {
      codec,
      stream,
    })
  }

  pub(crate) fn setup(&self, params: &mut DecodeParameters) -> Result<()> {
    let res = unsafe {
      sys::opj_setup_decoder(self.as_ptr(), params.as_ptr()) == 1
        && sys::opj_decoder_set_strict_mode(self.as_ptr(), params.strict as i32) == 1
    };
    if res {
      Ok(())
    } else {
      Err(Error::CreateCodecError(format!("Failed to setup decoder with parameters.")))
    }
  }

  pub(crate) fn read_header(&self) -> Result<Image> {
    let mut img: *mut sys::opj_image_t = ptr::null_mut();

    let res = unsafe { sys::opj_read_header(self.stream.as_ptr(), self.as_ptr(), &mut img)};
    // Try wrapping the image pointer before handling any errors.
    // Since the read header function might have allocated the image structure.
    let img = Image::new(img)?;
    if res == 1 {
      Ok(img)
    } else {
      Err(Error::CodecError("Failed to read header".into()))
    }
  }

  pub(crate) fn get_codestream_index(&self) -> Result<CodestreamIndex> {
    let index = ptr::NonNull::new(unsafe {
      sys::opj_get_cstr_index(self.as_ptr())
    }).ok_or_else(|| Error::CodecError("Failed to get codestream index".into()))?;
    Ok(CodestreamIndex(index))
  }

  pub(crate) fn get_codestream_info(&self) -> Result<CodestreamInfo> {
    let info = ptr::NonNull::new(unsafe {
      sys::opj_get_cstr_info(self.as_ptr())
    }).ok_or_else(|| Error::CodecError("Failed to get codestream info".into()))?;
    Ok(CodestreamInfo(info))
  }

  pub(crate) fn set_decode_area(&self, img: &Image, params: &DecodeParameters) -> Result<()> {
    if let Some(area) = &params.area {
      let res = unsafe {
        sys::opj_set_decode_area(self.as_ptr(), img.as_ptr(),
          area.start_x as i32,
          area.start_y as i32,
          area.end_x as i32,
          area.end_y as i32)
      };
      if res != 1 {
        return Err(Error::CreateCodecError(format!("Failed to set decode area.")));
      }
    }
    Ok(())
  }

  pub(crate) fn decode(&self, img: &Image) -> Result<()> {
    let res = unsafe {
      sys::opj_decode(self.as_ptr(), self.stream.as_ptr(), img.as_ptr()) == 1 &&
      sys::opj_end_decompress(self.as_ptr(), self.stream.as_ptr()) == 1
    };
    if res {
      Ok(())
    } else {
      Err(Error::CodecError("Failed to decode image".into()))
    }
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_codec_t {
    self.codec.as_ptr()
  }
}

pub(crate) struct Encoder<'a> {
  codec: Codec,
  stream: Stream<'a>,
}

impl<'a> Encoder<'a> {
  pub(crate) fn new(stream: Stream<'a>) -> Result<Self> {
    assert!(!stream.is_input());
    let fmt = stream.format();
    let codec = Codec::new(fmt, false)?;
    Ok(Self {
      codec,
      stream
    })
  }

  pub(crate) fn setup(&self, mut params: EncodeParameters, img: &Image) -> Result<()> {
    let res = unsafe {
      sys::opj_setup_encoder(self.as_ptr(), &mut params.0, img.as_ptr())
    };
    if res == 1 {
      Ok(())
    } else {
      Err(Error::CreateCodecError(format!("Failed to setup encoder with parameters.")))
    }
  }

  pub(crate) fn encode(&self, img: &Image) -> Result<()> {
    let res = unsafe {
      sys::opj_start_compress(self.as_ptr(), img.as_ptr(), self.stream.as_ptr()) == 1 &&
      sys::opj_encode(self.as_ptr(), self.stream.as_ptr()) == 1 &&
      sys::opj_end_compress(self.as_ptr(), self.stream.as_ptr()) == 1
    };
    if res {
      Ok(())
    } else {
      Err(Error::CodecError("Failed to encode image".into()))
    }
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_codec_t {
    self.codec.as_ptr()
  }
}

