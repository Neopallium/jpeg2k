use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    utils::BoxedFuture,
};

use super::*;

/// Jpeg 2000 asset loader for Bevy.
#[derive(Default)]
pub struct Jpeg2KAssetLoader;

impl AssetLoader for Jpeg2KAssetLoader {
  fn load<'a>(
    &'a self,
    bytes: &'a [u8],
    load_context: &'a mut LoadContext,
  ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
    Box::pin(async move {
      let image = Image::from_bytes(bytes)?;
      let txt: Texture = image.try_into()?;
      load_context.set_default_asset(LoadedAsset::new(txt));
      Ok(())
    })
  }

  fn extensions(&self) -> &[&str] {
    &["j2k", "jp2", "j2c", "jpc"]
  }
}

/// Try to convert a loaded Jpeg 2000 image into a Bevy `Texture`.
impl TryFrom<Image> for Texture {
  type Error = anyhow::Error;

  fn try_from(img: Image) -> Result<Texture> {
    use bevy::render::texture::*;
    // Get ref to inner image.
    let img = &img.img;
    let width = img.width();
    let height = img.height();
    let format;

    let data = match img.components() {
      [r] => {
        let r = unsafe {
          std::slice::from_raw_parts(r.data, (width * height) as usize)
        };
        format = TextureFormat::R8Unorm;
        r.iter().map(|r| *r as u8).collect()
      }
      [r, g, b] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 4);
        let (r, g, b) = unsafe {
          let r = std::slice::from_raw_parts(r.data, (width * height) as usize);
          let g = std::slice::from_raw_parts(g.data, (width * height) as usize);
          let b = std::slice::from_raw_parts(b.data, (width * height) as usize);
          (r, g, b)
        };

        format = TextureFormat::Rgba8UnormSrgb;
        for (r, (g, b)) in r.iter().zip(g.iter().zip(b.iter())) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8, u8::MAX]);
        }
        pixels
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

        format = TextureFormat::Rgba8UnormSrgb;
        for (r, (g, (b, a))) in r.iter().zip(g.iter().zip(b.iter().zip(a.iter()))) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8, *a as u8]);
        }
        pixels
      }
      _ => {
        return Err(anyhow!("Unsupported number of components: {:?}", img.num_components()));
      }
    };

    Ok(Texture::new(
      Extent3d::new(width, height, 1),
      TextureDimension::D2,
      data, format,
    ))
  }
}

/// Jpeg 2000 asset plugin for Bevy.
#[derive(Default, Clone, Debug)]
pub struct Jpeg2KPlugin;

impl Plugin for Jpeg2KPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_asset_loader::<Jpeg2KAssetLoader>()
      ;
  }
}
