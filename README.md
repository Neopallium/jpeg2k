# jpeg2k

JPEG 2000 image loader.

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
