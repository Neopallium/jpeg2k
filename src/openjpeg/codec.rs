use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::ptr;

// TODO: Create error type.
use anyhow::Result;

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

pub(crate) struct Codec(ptr::NonNull<sys::opj_codec_t>);

impl Drop for Codec {
  fn drop(&mut self) {
    unsafe {
      sys::opj_destroy_codec(self.0.as_ptr());
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
  pub(crate) fn new_decompress(fmt: J2KFormat, mut params: DecodeParamers) -> Result<Self> {
    let format: sys::CODEC_FORMAT = fmt.into();
    let ptr = unsafe {
      ptr::NonNull::new(sys::opj_create_decompress(format))
    };
    if let Some(ptr) = ptr {
      let null = ptr::null_mut();
      params.0.m_verbose = 1;
      unsafe {
        if params.0.m_verbose != 0 {
          sys::opj_set_info_handler(ptr.as_ptr(), Some(log_info), null);
          sys::opj_set_warning_handler(ptr.as_ptr(), Some(log_warn), null);
        }
        sys::opj_set_error_handler(ptr.as_ptr(), Some(log_error), null);
      }

      let res = unsafe {
        sys::opj_setup_decoder(ptr.as_ptr(), &mut params.0)
      };
      if res != 1 {
        Err(anyhow!("Failed to setup decoder with parameters."))?;
      }
      Ok(Self(ptr))
    } else {
      Err(anyhow!("Codec not supported: {:?}", fmt))
    }
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_codec_t {
    self.0.as_ptr()
  }
}
