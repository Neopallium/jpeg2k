use std::env;

use anyhow::Result;

use image::DynamicImage;

use jpeg2k::*;

fn main() -> Result<()> {
  dotenv::dotenv().ok();
  env_logger::init();

  let jp2_filename = env::args().nth(1).unwrap_or_else(|| "test.j2k".to_string());
  let savename = env::args().nth(2).unwrap_or_else(|| "test.jpg".to_string());
  let reduce = env::args()
    .nth(3)
    .unwrap_or_else(|| "0".to_string())
    .parse::<u32>()
    .expect("Reduce must be an integer.");
  let layers = env::args()
    .nth(4)
    .unwrap_or_else(|| "0".to_string())
    .parse::<u32>()
    .expect("Layers must be an integer.");
  let decode_area = env::args()
    .nth(5)
    .and_then(|area| area.parse::<DecodeArea>().ok());

  // Decode parameters.
  let params = DecodeParameters::new()
    .reduce(reduce)
    .layers(layers)
    .decode_area(decode_area);

  /*
  let mut file = File::open(jp2_filename)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  let jp2_image = Image::from_bytes_with(&mut buf, params)?;
  */
  let jp2_image = Image::from_file_with(jp2_filename, params)?;

  println!("dump image: {:#?}", jp2_image);

  println!(
    "jp2_image: width={:?}, height={:?}",
    jp2_image.width(),
    jp2_image.height()
  );

  let img: DynamicImage = jp2_image.try_into()?;
  img.save(&savename)?;

  println!("Saved to: {}", savename);
  Ok(())
}
