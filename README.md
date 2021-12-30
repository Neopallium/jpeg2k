# jpeg2k

Safe wrapper for `openjpeg-sys` with Bevy asset support.

## Example: Convert a Jpeg 2000 image to a png image.

```rust
use image::DynamicImage;

use jpeg2k::*;

fn main() {
  // Load jpeg 2000 file from file.
  let jp2_image = Image::from_file("./assets/example.j2k")
		.expect("Failed to load j2k file.");

  // Convert to a `image::DynamicImage`
  let img: DynamicImage = jp2_image.try_into()?;

  // Save as png file.
  img.save("out.png")?;
}
```

## Example: Bevy asset loader

```rust
use bevy::prelude::*;

use jpeg2k::loader::*;

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
