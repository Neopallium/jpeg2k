use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::ptr;

use log::{Level, log_enabled};

use super::*;

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
    }
  }
}

impl DecodeParameters {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn reduce(mut self, reduce: u32) -> Self {
    self.params.cp_reduce = reduce;
    self
  }

  pub fn layers(mut self, layers: u32) -> Self {
    self.params.cp_layer = layers;
    self
  }

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
      sys::opj_setup_decoder(self.as_ptr(), params.as_ptr())
    };
    if res == 1 {
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

