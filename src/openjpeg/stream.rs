use std::ffi::CString;
use std::os::raw::c_void;

use std::path::Path;

use super::*;

struct WrappedSlice<'a> {
  offset: usize,
  buf: &'a [u8],
}

impl<'a> WrappedSlice<'a> {
  fn new(buf: &'a [u8]) -> Box<Self> {
    Box::new(Self {
      offset: 0,
      buf,
    })
  }

  fn remaining(&self) -> usize {
    self.buf.len() - self.offset
  }

  fn seek(&mut self, new_offset: usize) -> usize {
    // Make sure `new_offset <= buf.len()`
    self.offset = std::cmp::min(self.buf.len(), new_offset);
    self.offset
  }

  fn consume(&mut self, n_bytes: usize) -> usize {
    let offset = self.offset.saturating_add(n_bytes);
    // Make sure `offset <= buf.len()`
    self.offset = std::cmp::min(self.buf.len(), offset);
    self.offset
  }

  fn read_into(&mut self, out_buffer: &mut [u8]) -> usize {
    // Get number of remaining bytes.
    let remaining = self.remaining();
    if remaining == 0 {
      // No more bytes.
      return 0;
    }

    // Try to fill the output buffer.
    let n_read = std::cmp::min(remaining, out_buffer.len());
    let offset = self.offset;
    let end_off = self.consume(n_read);
    out_buffer[0..n_read].copy_from_slice(&self.buf[offset..end_off]);

    n_read
  }
}

pub(crate) struct Stream<'a> {
  stream: *mut sys::opj_stream_t,
  format: J2KFormat,
  is_input: bool,
  buf: Option<&'a [u8]>,
}

impl Drop for Stream<'_> {
  fn drop(&mut self) {
    unsafe {
      sys::opj_stream_destroy(self.stream);
    }
  }
}

impl std::fmt::Debug for Stream<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(slice) = &self.buf {
      f.write_fmt(format_args!("BufStream: len={}", slice.len()))
    } else {
      f.write_fmt(format_args!("FileStream"))
    }
  }
}

extern "C" fn buf_read_stream_free_fn(p_data: *mut c_void) {
  let ptr = p_data as *mut WrappedSlice;
  drop(unsafe {
    Box::from_raw(ptr)
  })
}

extern "C" fn buf_read_stream_read_fn(p_buffer: *mut c_void, nb_bytes: usize, p_data: *mut c_void) -> usize {
  if p_buffer.is_null() || nb_bytes == 0 {
    return 0;
  }

  let slice = unsafe { &mut *(p_data as *mut WrappedSlice) };
  let out_buf = unsafe {
    std::slice::from_raw_parts_mut(p_buffer as *mut u8, nb_bytes)
  };
  slice.read_into(out_buf)
}

extern "C" fn buf_read_stream_skip_fn(nb_bytes: i64, p_data: *mut c_void) -> i64 {
  let slice = unsafe { &mut *(p_data as *mut WrappedSlice) };
  slice.consume(nb_bytes as usize) as i64
}

extern "C" fn buf_read_stream_seek_fn(nb_bytes: i64, p_data: *mut c_void) -> i32 {
  let slice = unsafe { &mut *(p_data as *mut WrappedSlice) };
  let seek_offset = nb_bytes as usize;
  let new_offset = slice.seek(seek_offset);

  // Return true if the seek worked.
  if seek_offset == new_offset { 1 } else { 0 }
}

impl<'a> Stream<'a> {
  pub(crate) fn from_bytes(buf: &'a [u8]) -> Result<Self> {
    let format = j2k_detect_format(buf)?;
    let len = buf.len();
    let data = WrappedSlice::new(buf);
    unsafe {
      let p_data = Box::into_raw(data) as *mut c_void;
      let stream = sys::opj_stream_default_create(1);
      sys::opj_stream_set_read_function(stream, Some(buf_read_stream_read_fn));
      sys::opj_stream_set_skip_function(stream, Some(buf_read_stream_skip_fn));
      sys::opj_stream_set_seek_function(stream, Some(buf_read_stream_seek_fn));
      sys::opj_stream_set_user_data_length(stream, len as u64);
      sys::opj_stream_set_user_data(
        stream,
        p_data,
        Some(buf_read_stream_free_fn));

      Ok(Self {
        stream,
        format,
        is_input: true,
        buf: Some(buf),
      })
    }
  }

  pub(crate) fn new_file<P: AsRef<Path>>(path: P, is_input: bool) -> Result<Self> {
    let path = path.as_ref();
    if !path.exists() {
      return Err(Error::FileNotFoundError(format!("{:?}", path)));
    }
    let format = j2k_detect_format_from_extension(path.extension())?;
    let c_path = path.to_str()
      .and_then(|p| CString::new(p.as_bytes()).ok())
      .ok_or_else(|| Error::BadFilenameError(format!("Can't pass filename to openjpeg-sys: {:?}", path)))?;

    let c_input = if is_input { 1 } else { 0 };
    let stream = unsafe {
      sys::opj_stream_create_default_file_stream(c_path.as_ptr(), c_input)
    };
    if stream.is_null() {
      return Err(Error::NullPointerError("Failed to create file stream: NULL opj_stream_t"));
    }
    Ok(Self {
      stream,
      format,
      is_input,
      buf: None,
    })
  }

  pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    Self::new_file(path, true)
  }

  pub(crate) fn to_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    Self::new_file(path, false)
  }

  pub(crate) fn format(&self) -> J2KFormat {
    self.format
  }

  pub(crate) fn is_input(&self) -> bool {
    self.is_input
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_stream_t {
    self.stream
  }
}
