use std::ptr;

// TODO: Create error type.
use anyhow::Result;

use super::*;

pub(crate) struct WrappedImage(ptr::NonNull<sys::opj_image_t>);

impl Drop for WrappedImage {
  fn drop(&mut self) {
    unsafe {
      sys::opj_image_destroy(self.0.as_ptr());
    }
  }
}

impl WrappedImage {
  pub(crate) fn new(ptr: *mut sys::opj_image_t) -> Result<Self> {
    let ptr = ptr::NonNull::new(ptr)
      .ok_or_else(|| anyhow!("Null pointer."))?;
    Ok(Self(ptr))
  }

  fn image(&self) -> &sys::opj_image_t {
    unsafe { &(*self.0.as_ptr()) }
  }

  pub(crate) fn width(&self) -> u32 {
    let img = self.image();
    img.x1 - img.x0
  }

  pub(crate) fn height(&self) -> u32 {
    let img = self.image();
    img.y1 - img.y0
  }

  pub(crate) fn num_components(&self) -> u32 {
    let img = self.image();
    img.numcomps
  }

  pub(crate) fn components(&self) -> &[sys::opj_image_comp_t] {
    let img = self.image();
    let numcomps = img.numcomps;
    unsafe { std::slice::from_raw_parts(img.comps, numcomps as usize) }
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_image_t {
    self.0.as_ptr()
  }
}

pub struct Image {
  pub(crate) img: WrappedImage,
}

impl Image {
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    let format = j2k_detect_format(buf)?;

    let stream = Stream::from_bytes(buf);

    let params = DecodeParamers::default();
    let codec = Codec::new_decompress(format, params)?;

    let img = stream.read_header(&codec)?;

    stream.decode(&codec, &img)?;

    Ok(Self{
      img,
    })
  }

  pub fn width(&self) -> u32 {
    self.img.width()
  }

  pub fn height(&self) -> u32 {
    self.img.height()
  }
}

#[cfg(feature = "image")]
impl TryFrom<Image> for ::image::DynamicImage {
  type Error = anyhow::Error;

  fn try_from(img: Image) -> Result<::image::DynamicImage> {
    use ::image::*;
    let img = &img.img;
    let width = img.width();
    let height = img.height();

    let img = match img.components() {
      [r] => {
        let r = unsafe {
          std::slice::from_raw_parts(r.data, (width * height) as usize)
        };
        let pixels = r.iter().map(|r| *r as u8).collect();

        let gray = GrayImage::from_vec(width, height, pixels)
          .ok_or_else(|| anyhow!("Not enough pixels."))?;

        DynamicImage::ImageLuma8(gray)
      }
      [r, g, b] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 3);
        let (r, g, b) = unsafe {
          let r = std::slice::from_raw_parts(r.data, (width * height) as usize);
          let g = std::slice::from_raw_parts(g.data, (width * height) as usize);
          let b = std::slice::from_raw_parts(b.data, (width * height) as usize);
          (r, g, b)
        };

        for (r, (g, b)) in r.iter().zip(g.iter().zip(b.iter())) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8]);
        }
        let rgb = RgbImage::from_vec(width, height, pixels)
          .ok_or_else(|| anyhow!("Not enough pixels."))?;

        DynamicImage::ImageRgb8(rgb)
      }
      [r, g, b, a] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 4);
        let (r, g, b, a) = unsafe {
          let r = std::slice::from_raw_parts(r.data, (width * height) as usize);
          let g = std::slice::from_raw_parts(g.data, (width * height) as usize);
          let b = std::slice::from_raw_parts(b.data, (width * height) as usize);
          let a = std::slice::from_raw_parts(a.data, (width * height) as usize);
          (r, g, b, a)
        };

        for (r, (g, (b, a))) in r.iter().zip(g.iter().zip(b.iter().zip(a.iter()))) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8, *a as u8]);
        }
        let rgba = RgbaImage::from_vec(width, height, pixels)
          .ok_or_else(|| anyhow!("Not enough pixels."))?;

        DynamicImage::ImageRgba8(rgba)
      }
      _ => {
        Err(anyhow!("Unsupported number of components: {:?}", img.num_components()))?
      }
    };
    Ok(img)
  }
}

