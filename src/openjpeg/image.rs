use std::ptr;

use std::path::Path;

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
      .ok_or_else(|| Error::NullPointerError("Image: NULL `opj_image_t`"))?;
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

/// A Jpeg2000 Image.
pub struct Image {
  pub(crate) img: WrappedImage,
}

impl Image {
  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    let stream = Stream::from_bytes(buf)?;

    let decoder = Decoder::new(stream)?;
    let params = DecodeParamers::default();
    decoder.setup(params)?;

    let img = decoder.read_header()?;

    decoder.decode(&img)?;

    Ok(Self{
      img,
    })
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let stream = Stream::from_file(path)?;

    let decoder = Decoder::new(stream)?;
    let params = DecodeParamers::default();
    decoder.setup(params)?;

    let img = decoder.read_header()?;

    decoder.decode(&img)?;

    Ok(Self{
      img,
    })
  }

  /// Save image to Jpeg 2000 file.  It will detect the J2K format.
  pub fn save_as_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let stream = Stream::to_file(path)?;

    let encoder = Encoder::new(stream)?;
    let params = EncodeParamers::default();
    encoder.setup(params, &self.img)?;

    encoder.encode(&self.img)?;

    Ok(())
  }

  /// Image width.
  pub fn width(&self) -> u32 {
    self.img.width()
  }

  /// Image height.
  pub fn height(&self) -> u32 {
    self.img.height()
  }
}

/// Try to convert a loaded Jpeg 2000 image into a `image::DynamicImage`.
#[cfg(feature = "image")]
impl TryFrom<Image> for ::image::DynamicImage {
  type Error = Error;

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
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

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
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

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
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        DynamicImage::ImageRgba8(rgba)
      }
      _ => {
        return Err(Error::UnsupportedComponentsError(img.num_components()));
      }
    };
    Ok(img)
  }
}

