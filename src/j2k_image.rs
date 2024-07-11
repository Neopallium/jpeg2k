use std::ptr;

#[cfg(feature = "file-io")]
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

  /// Component data scaled to unsigned 8bit.
  pub fn data_u8(&self) -> Box<dyn Iterator<Item = u8>> {
    let len = (self.0.w * self.0.h) as usize;
    if self.is_signed() {
      let data = unsafe { std::slice::from_raw_parts(self.0.data, len) };
      let old_max = (1 << (self.precision() - 1)) as i64;
      const NEW_MAX: i64 = 1 << (8 - 1);
      const ADJUST: u8 = (NEW_MAX - 1) as u8;
      Box::new(
        data
          .iter()
          .map(move |p| (((*p as i64) * NEW_MAX) / old_max) as u8 + ADJUST),
      )
    } else {
      let data = unsafe { std::slice::from_raw_parts(self.0.data as *const u32, len) };
      let old_max = ((1 << self.precision()) - 1) as u64;
      const NEW_MAX: u64 = (1 << 8) - 1;
      Box::new(
        data
          .iter()
          .map(move |p| (((*p as u64) * NEW_MAX) / old_max) as u8),
      )
    }
  }

  /// Component data scaled to unsigned 16bit.
  pub fn data_u16(&self) -> Box<dyn Iterator<Item = u16>> {
    let len = (self.0.w * self.0.h) as usize;
    if self.is_signed() {
      let data = unsafe { std::slice::from_raw_parts(self.0.data, len) };
      let old_max = (1 << (self.precision() - 1)) as i64;
      const NEW_MAX: i64 = 1 << (16 - 1);
      const ADJUST: u16 = (NEW_MAX - 1) as u16;
      Box::new(
        data
          .iter()
          .map(move |p| (((*p as i64) * NEW_MAX) / old_max) as u16 + ADJUST),
      )
    } else {
      let data = unsafe { std::slice::from_raw_parts(self.0.data as *const u32, len) };
      let old_max = ((1 << self.precision()) - 1) as u64;
      const NEW_MAX: u64 = (1 << 16) - 1;
      Box::new(
        data
          .iter()
          .map(move |p| (((*p as u64) * NEW_MAX) / old_max) as u16),
      )
    }
  }
}

/// Image Data.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageFormat {
  L8,
  La8,
  Rgb8,
  Rgba8,
  L16,
  La16,
  Rgb16,
  Rgba16,
}

/// Image Pixel Data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImagePixelData {
  L8(Vec<u8>),
  La8(Vec<u8>),
  Rgb8(Vec<u8>),
  Rgba8(Vec<u8>),
  L16(Vec<u16>),
  La16(Vec<u16>),
  Rgb16(Vec<u16>),
  Rgba16(Vec<u16>),
}

/// Image Data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageData {
  pub width: u32,
  pub height: u32,
  pub format: ImageFormat,
  pub data: ImagePixelData,
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
      .field("has_icc_profile", &self.has_icc_profile())
      .field("numcomps", &img.numcomps)
      .field("comps", &self.components())
      .finish()
  }
}

impl Image {
  pub(crate) fn new(ptr: *mut sys::opj_image_t) -> Result<Self> {
    let img =
      ptr::NonNull::new(ptr).ok_or_else(|| Error::NullPointerError("Image: NULL `opj_image_t`"))?;
    Ok(Self { img })
  }

  /// Load a Jpeg 2000 image from bytes.  It will detect the J2K format.
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
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
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let stream = Stream::from_bytes(buf)?;
    Self::from_stream(stream, params)
  }

  /// Load a Jpeg 2000 image from file.  It will detect the J2K format.
  #[cfg(feature = "file-io")]
  pub fn from_file_with<P: AsRef<Path>>(path: P, params: DecodeParameters) -> Result<Self> {
    let stream = Stream::from_file(path)?;
    Self::from_stream(stream, params)
  }

  /// Save image to Jpeg 2000 file.  It will detect the J2K format.
  #[cfg(feature = "file-io")]
  pub fn save_as_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let stream = Stream::to_file(path)?;
    self.to_stream(stream, Default::default())
  }

  /// Save image to Jpeg 2000 file.  It will detect the J2K format.
  #[cfg(feature = "file-io")]
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

  #[cfg(feature = "file-io")]
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
    self
      .component_dimensions()
      .map(|(w, _)| w)
      .unwrap_or_default()
  }

  /// Decoded image height.  Reduced by the scaling factor.
  pub fn height(&self) -> u32 {
    self
      .component_dimensions()
      .map(|(_, h)| h)
      .unwrap_or_default()
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

  /// Has ICC Profile.
  pub fn has_icc_profile(&self) -> bool {
    let img = self.image();
    !img.icc_profile_buf.is_null()
  }

  fn component_dimensions(&self) -> Option<(u32, u32)> {
    self
      .components()
      .get(0)
      .map(|comp| (comp.width(), comp.height()))
  }

  /// Image components.
  pub fn components(&self) -> &[ImageComponent] {
    let img = self.image();
    let numcomps = img.numcomps;
    unsafe { std::slice::from_raw_parts(img.comps as *mut ImageComponent, numcomps as usize) }
  }

  /// Convert image components into pixels.
  ///
  /// `alpha_default` - The default value for the alpha channel if there is no alpha component.
  pub fn get_pixels(&self, alpha_default: Option<u32>) -> Result<ImageData> {
    let comps = self.components();
    let (width, height) = comps
      .get(0)
      .map(|c| (c.width(), c.height()))
      .ok_or_else(|| Error::UnsupportedComponentsError(0))?;
    let max_prec = comps
      .iter()
      .fold(std::u32::MIN, |max, c| max.max(c.precision()));
    let has_alpha = comps.iter().any(|c| c.is_alpha());
    let format;

    // Check for support color space.
    match self.color_space() {
      ColorSpace::Unknown | ColorSpace::Unspecified => {
        // Assume either Grey/RGB/RGBA based on number of components.
      }
      ColorSpace::SRGB | ColorSpace::Gray => (),
      cs => {
        return Err(Error::UnsupportedColorSpaceError(cs));
      }
    }

    let data = match (comps, has_alpha, max_prec) {
      ([r], _, 1..=8) => {
        if let Some(alpha) = alpha_default {
          format = ImageFormat::La8;
          ImagePixelData::La8(r.data_u8().flat_map(|r| [r, alpha as u8]).collect())
        } else {
          format = ImageFormat::L8;
          ImagePixelData::L8(r.data_u8().map(|r| r).collect())
        }
      }
      ([r], _, 9..=16) => {
        if let Some(alpha) = alpha_default {
          format = ImageFormat::La16;
          ImagePixelData::La16(r.data_u16().flat_map(|r| [r, alpha as u16]).collect())
        } else {
          format = ImageFormat::L16;
          ImagePixelData::L16(r.data_u16().collect())
        }
      }
      ([r, a], true, 1..=8) => {
        format = ImageFormat::La8;
        ImagePixelData::La8(
          r.data_u8()
            .zip(a.data_u8())
            .flat_map(|(r, a)| [r, a])
            .collect(),
        )
      }
      ([r, a], true, 9..=16) => {
        format = ImageFormat::La16;
        ImagePixelData::La16(
          r.data_u16()
            .zip(a.data_u16())
            .flat_map(|(r, a)| [r, a])
            .collect(),
        )
      }
      ([r, g, b], false, 1..=8) => {
        if let Some(alpha) = alpha_default {
          format = ImageFormat::Rgba8;
          ImagePixelData::Rgba8(
            r.data_u8()
              .zip(g.data_u8().zip(b.data_u8()))
              .flat_map(|(r, (g, b))| [r, g, b, alpha as u8])
              .collect(),
          )
        } else {
          format = ImageFormat::Rgb8;
          ImagePixelData::Rgb8(
            r.data_u8()
              .zip(g.data_u8().zip(b.data_u8()))
              .flat_map(|(r, (g, b))| [r, g, b])
              .collect(),
          )
        }
      }
      ([r, g, b], false, 9..=16) => {
        if let Some(alpha) = alpha_default {
          format = ImageFormat::Rgba16;
          ImagePixelData::Rgba16(
            r.data_u16()
              .zip(g.data_u16().zip(b.data_u16()))
              .flat_map(|(r, (g, b))| [r, g, b, alpha as u16])
              .collect(),
          )
        } else {
          format = ImageFormat::Rgb16;
          ImagePixelData::Rgb16(
            r.data_u16()
              .zip(g.data_u16().zip(b.data_u16()))
              .flat_map(|(r, (g, b))| [r, g, b])
              .collect(),
          )
        }
      }
      ([r, g, b, a], _, 1..=8) => {
        format = ImageFormat::Rgba8;
        ImagePixelData::Rgba8(
          r.data_u8()
            .zip(g.data_u8().zip(b.data_u8().zip(a.data_u8())))
            .flat_map(|(r, (g, (b, a)))| [r, g, b, a])
            .collect(),
        )
      }
      ([r, g, b, a], _, 9..=16) => {
        format = ImageFormat::Rgba16;
        ImagePixelData::Rgba16(
          r.data_u16()
            .zip(g.data_u16().zip(b.data_u16().zip(a.data_u16())))
            .flat_map(|(r, (g, (b, a)))| [r, g, b, a])
            .collect(),
        )
      }
      _ => {
        return Err(Error::UnsupportedComponentsError(self.num_components()));
      }
    };
    Ok(ImageData {
      width,
      height,
      format,
      data,
    })
  }
}

/// Try to convert a loaded Jpeg 2000 image into a `image::DynamicImage`.
#[cfg(feature = "image")]
impl TryFrom<&Image> for ::image::DynamicImage {
  type Error = Error;

  fn try_from(img: &Image) -> Result<::image::DynamicImage> {
    use image::*;
    let ImageData {
      width,
      height,
      data,
      ..
    } = img.get_pixels(None)?;
    match data {
      crate::ImagePixelData::L8(data) => {
        let gray = GrayImage::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageLuma8(gray))
      }
      crate::ImagePixelData::La8(data) => {
        let gray = GrayAlphaImage::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageLumaA8(gray))
      }
      crate::ImagePixelData::Rgb8(data) => {
        let rgb = RgbImage::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageRgb8(rgb))
      }
      crate::ImagePixelData::Rgba8(data) => {
        let rgba = RgbaImage::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageRgba8(rgba))
      }
      crate::ImagePixelData::L16(data) => {
        let gray = ImageBuffer::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageLuma16(gray))
      }
      crate::ImagePixelData::La16(data) => {
        let gray = ImageBuffer::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageLumaA16(gray))
      }
      crate::ImagePixelData::Rgb16(data) => {
        let rgb = ImageBuffer::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageRgb16(rgb))
      }
      crate::ImagePixelData::Rgba16(data) => {
        let rgba = ImageBuffer::from_vec(width, height, data)
          .expect("Shouldn't happen.  Report to jpeg2k if you see this.");

        Ok(DynamicImage::ImageRgba16(rgba))
      }
    }
  }
}
