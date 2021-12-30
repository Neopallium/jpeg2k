use std::ptr;

use std::path::Path;

use super::*;

/// A Jpeg2000 Image Component.
pub struct ImageComponent(pub(crate) sys::opj_image_comp_t);

impl std::fmt::Debug for ImageComponent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ImageComponent")
      .field("dx", &self.0.dx)
      .field("dy", &self.0.dy)
      .field("w", &self.0.w)
      .field("h", &self.0.h)
      .field("x0", &self.0.x0)
      .field("y0", &self.0.y0)
      .field("prec", &self.0.prec)
      .field("bpp", &self.0.bpp)
      .field("sgnd", &self.0.sgnd)
      .field("resno_decoded", &self.0.resno_decoded)
      .field("factor", &self.0.factor)
      .field("data", &self.0.data)
      .field("alpha", &self.0.alpha)
      .finish()
  }
}

impl ImageComponent {
  /// Component width.
  pub fn width(&self) -> u32 {
    self.0.w
  }

  /// Component height.
  pub fn height(&self) -> u32 {
    self.0.h
  }

  /// Component precision.
  pub fn precision(&self) -> u32 {
    self.0.prec
  }

  /// Image depth in bits.
  pub fn bpp(&self) -> u32 {
    self.0.bpp
  }

  /// Is component an alpha channel.
  pub fn is_alpha(&self) -> bool {
    self.0.alpha == 1
  }

  /// Is component data signed.
  pub fn is_signed(&self) -> bool {
    self.0.sgnd == 1
  }

  /// Component data.
  pub fn data(&self) -> &[i32] {
    let len = (self.0.w * self.0.h) as usize;
    unsafe { std::slice::from_raw_parts(self.0.data, len) }
  }
}

/// A Jpeg2000 Image.
pub struct Image {
  img: ptr::NonNull<sys::opj_image_t>,
}

impl Drop for Image {
  fn drop(&mut self) {
    unsafe {
      sys::opj_image_destroy(self.img.as_ptr());
    }
  }
}

impl std::fmt::Debug for Image {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let img = unsafe { &*self.as_ptr() };
    f.debug_struct("Image")
      .field("x_offset", &self.x_offset())
      .field("y_offset", &self.y_offset())
      .field("width", &self.orig_width())
      .field("height", &self.orig_height())
      .field("color_space", &self.color_space())
      .field("numcomps", &img.numcomps)
      .field("comps", &self.components())
      .finish()
  }
}

impl Image {
  pub(crate) fn new(ptr: *mut sys::opj_image_t) -> Result<Self> {
    let img = ptr::NonNull::new(ptr)
      .ok_or_else(|| Error::NullPointerError("Image: NULL `opj_image_t`"))?;
    Ok(Self{ img })
  }

  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    let stream = Stream::from_bytes(buf)?;
    Self::from_stream(stream, Default::default())
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let stream = Stream::from_file(path)?;
    Self::from_stream(stream, Default::default())
  }

  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let stream = Stream::from_bytes(buf)?;
    Self::from_stream(stream, params)
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  pub fn from_file_with<P: AsRef<Path>>(path: P, params: DecodeParameters) -> Result<Self> {
    let stream = Stream::from_file(path)?;
    Self::from_stream(stream, params)
  }

  /// Save image to Jpeg 2000 file.  It will detect the J2K format.
  pub fn save_as_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let stream = Stream::to_file(path)?;
    self.to_stream(stream, Default::default())
  }

  /// Save image to Jpeg 2000 file.  It will detect the J2K format.
  pub fn save_as_file_with<P: AsRef<Path>>(&self, path: P, params: EncodeParameters) -> Result<()> {
    let stream = Stream::to_file(path)?;
    self.to_stream(stream, params)
  }

  fn from_stream(stream: Stream<'_>, mut params: DecodeParameters) -> Result<Self> {
    let decoder = Decoder::new(stream)?;
    decoder.setup(&mut params)?;

    let img = decoder.read_header()?;

    decoder.set_decode_area(&img, &params)?;

    decoder.decode(&img)?;

    Ok(img)
  }

  fn to_stream(&self, stream: Stream<'_>, params: EncodeParameters) -> Result<()> {
    let encoder = Encoder::new(stream)?;
    encoder.setup(params, &self)?;

    encoder.encode(&self)?;

    Ok(())
  }

  fn image(&self) -> &sys::opj_image_t {
    unsafe { &(*self.img.as_ptr()) }
  }

  pub(crate) fn as_ptr(&self) -> *mut sys::opj_image_t {
    self.img.as_ptr()
  }

  /// Horizontal offset.
  pub fn x_offset(&self) -> u32 {
    let img = self.image();
    img.x0
  }

  /// Vertical offset.
  pub fn y_offset(&self) -> u32 {
    let img = self.image();
    img.y0
  }

  /// Full resolution image width.  Not reduced by the scaling factor.
  pub fn orig_width(&self) -> u32 {
    let img = self.image();
    img.x1 - img.x0
  }

  /// Full resolution image height.  Not reduced by the scaling factor.
  pub fn orig_height(&self) -> u32 {
    let img = self.image();
    img.y1 - img.y0
  }

  /// Decoded image width.  Reduced by the scaling factor.
  pub fn width(&self) -> u32 {
    self.component_dimensions()
      .map(|(w,_)| w).unwrap_or_default()
  }

  /// Decoded image height.  Reduced by the scaling factor.
  pub fn height(&self) -> u32 {
    self.component_dimensions()
      .map(|(_,h)| h).unwrap_or_default()
  }

  /// Color space.
  pub fn color_space(&self) -> ColorSpace {
    let img = self.image();
    img.color_space.into()
  }

  /// Number of components.
  pub fn num_components(&self) -> u32 {
    let img = self.image();
    img.numcomps
  }

  fn component_dimensions(&self) -> Option<(u32, u32)> {
    self.components().get(0)
      .map(|comp| (comp.width(), comp.height()))
  }

  /// Image components.
  pub fn components(&self) -> &[ImageComponent] {
    let img = self.image();
    let numcomps = img.numcomps;
    unsafe { std::slice::from_raw_parts(img.comps as *mut ImageComponent, numcomps as usize) }
  }
}

/// Try to convert a loaded Jpeg 2000 image into a `image::DynamicImage`.
#[cfg(feature = "image")]
impl TryFrom<Image> for ::image::DynamicImage {
  type Error = Error;

  fn try_from(img: Image) -> Result<::image::DynamicImage> {
    use ::image::*;
    let comps = img.components();
    let (width, height) = comps.get(0).map(|c| (c.width(), c.height()))
      .ok_or_else(|| Error::UnsupportedComponentsError(0))?;

    let img = match comps {
      [r] => {
        let pixels = r.data().iter().map(|r| *r as u8).collect();

        let gray = GrayImage::from_vec(width, height, pixels)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        DynamicImage::ImageLuma8(gray)
      }
      [r, g, b] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 3);

        for (r, (g, b)) in r.data().iter().zip(g.data().iter().zip(b.data().iter())) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8]);
        }
        let rgb = RgbImage::from_vec(width, height, pixels)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        DynamicImage::ImageRgb8(rgb)
      }
      [r, g, b, a] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 4);

        for (r, (g, (b, a))) in r.data().iter().zip(g.data().iter().zip(b.data().iter().zip(a.data().iter()))) {
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
