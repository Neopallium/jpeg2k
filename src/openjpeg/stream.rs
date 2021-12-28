use std::os::raw::c_void;
use std::ptr;

// TODO: Create error type.
use anyhow::Result;

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
    if let Some(boxed) = &self.buf {
      f.write_fmt(format_args!("BufStream: buf.len {}", boxed.len()))
    } else {
      f.write_fmt(format_args!("FileStream"))
    }
  }
}

extern "C" fn buf_stream_free_fn(p_data: *mut c_void) {
  unsafe {
    let ptr = p_data as *mut WrappedSlice;
    drop(Box::from_raw(ptr))
  }
}

extern "C" fn buf_stream_read_fn(p_buffer: *mut c_void, nb_bytes: usize, p_data: *mut c_void) -> usize {
  if p_buffer.is_null() || nb_bytes == 0 {
    return 0;
  }

  let slice = unsafe { &mut *(p_data as *mut WrappedSlice) };
  let out_buf = unsafe {
    std::slice::from_raw_parts_mut(p_buffer as *mut u8, nb_bytes)
  };
  slice.read_into(out_buf)
}

extern "C" fn buf_stream_skip_fn(nb_bytes: i64, p_data: *mut c_void) -> i64 {
  let slice = unsafe { &mut *(p_data as *mut WrappedSlice) };
  slice.consume(nb_bytes as usize) as i64
}

extern "C" fn buf_stream_seek_fn(nb_bytes: i64, p_data: *mut c_void) -> i32 {
  let slice = unsafe { &mut *(p_data as *mut WrappedSlice) };
  let seek_offset = nb_bytes as usize;
  let new_offset = slice.seek(seek_offset);

  // Return true if the seek worked.
  if seek_offset == new_offset { 1 } else { 0 }
}

impl<'a> Stream<'a> {
  pub(crate) fn from_bytes(buf: &'a [u8]) -> Self {
    let len = buf.len();
    let data = WrappedSlice::new(buf);
    unsafe {
      let p_data = Box::into_raw(data) as *mut c_void;
      let stream = sys::opj_stream_default_create(1);
      sys::opj_stream_set_read_function(stream, Some(buf_stream_read_fn));
      sys::opj_stream_set_skip_function(stream, Some(buf_stream_skip_fn));
      sys::opj_stream_set_seek_function(stream, Some(buf_stream_seek_fn));
      sys::opj_stream_set_user_data_length(stream, len as u64);
      sys::opj_stream_set_user_data(
        stream,
        p_data,
        Some(buf_stream_free_fn));

      Self {
        stream,
        buf: Some(buf),
      }
    }
  }

  pub(crate) fn read_header(&self, codec: &Codec) -> Result<WrappedImage> {
    let mut img: *mut sys::opj_image_t = ptr::null_mut();

    let res = unsafe { sys::opj_read_header(self.stream, codec.as_ptr(), &mut img)};
    // Try wrapping the image pointer before handling any errors.
    // Since the read header function might have allocated the image structure.
    let img = WrappedImage::new(img);
    if res != 1 {
      Err(anyhow!("Failed to read header."))?;
    }
    img
  }

  pub(crate) fn decode(&self, codec: &Codec, img: &WrappedImage) -> Result<()> {
    let res = unsafe {
      sys::opj_decode(codec.as_ptr(), self.stream, img.as_ptr())
    };
    if res != 1 {
      Err(anyhow!("Failed to decode image."))
    } else {
      Ok(())
    }
  }
}


