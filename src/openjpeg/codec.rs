use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::ptr;

use log::{Level, log_enabled};

use super::*;

pub(crate) struct DecodeParamers(sys::opj_dparameters);

impl Default for DecodeParamers {
  fn default() -> Self {
    Self(unsafe {
      let mut ptr = std::mem::zeroed::<sys::opj_dparameters>();
      sys::opj_set_default_decoder_parameters(&mut ptr as *mut _);
      ptr
    })
  }
}

pub(crate) struct EncodeParamers(sys::opj_cparameters);

impl Default for EncodeParamers {
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

  pub(crate) fn setup(&self, mut params: DecodeParamers) -> Result<()> {
    let res = unsafe {
      sys::opj_setup_decoder(self.as_ptr(), &mut params.0)
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

  pub(crate) fn setup(&self, mut params: EncodeParamers, img: &Image) -> Result<()> {
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

