# jpeg2k
Safe wrapper for `openjpeg-sys` with Bevy asset support.

## Example: Bevy asset loader

```rust
use std::fs::File;
use std::io::prelude::*;
use std::env;

use anyhow::Result;

use image::DynamicImage;

use jpeg2k::*;

fn main() -> Result<()> {
  let jp2_filename = env::args().nth(1)
    .unwrap_or_else(|| "test.j2k".to_string());
  let savename = env::args().nth(2)
    .unwrap_or_else(|| "test.jpg".to_string());

	// Load file bytes.
  let mut file = File::open(jp2_filename)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

	// Load jpeg 2000 file from bytes.
  let jp2_image = Image::from_bytes(&mut buf)?;

  println!("jp2_image: width={:?}, height={:?}", jp2_image.width(), jp2_image.height());

	// Convert to a `image::DynamicImage`
  let img: DynamicImage = jp2_image.try_into()?;

	// Using `image` crate to save image to another format: png, jpg, etc...
  img.save(&savename)?;

  println!("Saved to: {}", savename);
  Ok(())
}
```

## Example: Bevy asset loader

```rust
use bevy::prelude::*;

use jpeg2k::*;

fn main() {
  App::build()
    .add_plugins(DefaultPlugins)

		// Load the Jpeg 2000 asset loader plugin.
    .add_plugin(Jpeg2KPlugin)

    .add_startup_system(setup.system())
    .run();
}

fn setup(
  asset_server: Res<AssetServer>,
) {
	// Load j2k, jp2, j2c, images.
  let texture_handle = asset_server.load("example.j2k");
	// <Use the texture handle>
}

```
