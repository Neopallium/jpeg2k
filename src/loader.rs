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
  type Error = Error;

  fn try_from(img: Image) -> Result<Texture> {
    use bevy::render::texture::*;
    let comps = img.components();
    let (width, height) = comps.get(0).map(|c| (c.width(), c.height()))
      .ok_or_else(|| Error::UnsupportedComponentsError(0))?;
    let format;

    let data = match comps {
      [r] => {
        format = TextureFormat::R8Unorm;
        r.data().iter().map(|r| *r as u8).collect()
      }
      [r, g, b] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 4);

        format = TextureFormat::Rgba8UnormSrgb;
        for (r, (g, b)) in r.data().iter().zip(g.data().iter().zip(b.data().iter())) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8, u8::MAX]);
        }
        pixels
      }
      [r, g, b, a] => {
        let len = (width * height) as usize;
        let mut pixels = Vec::with_capacity(len * 4);

        format = TextureFormat::Rgba8UnormSrgb;
        for (r, (g, (b, a))) in r.data().iter().zip(g.data().iter().zip(b.data().iter().zip(a.data().iter()))) {
          pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8, *a as u8]);
        }
        pixels
      }
      _ => {
        return Err(Error::UnsupportedComponentsError(img.num_components()));
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
