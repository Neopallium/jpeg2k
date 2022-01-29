use std::env;

use anyhow::Result;

use jpeg2k::*;

fn main() -> Result<()> {
  dotenv::dotenv().ok();
  env_logger::init();

  let jp2_filename = env::args().nth(1).unwrap_or_else(|| "test.j2k".to_string());
  let savename = env::args().nth(2).unwrap_or_else(|| "test.jp2".to_string());

  let jp2_image = Image::from_file(jp2_filename)?;

  println!(
    "jp2_image: width={:?}, height={:?}",
    jp2_image.width(),
    jp2_image.height()
  );

  jp2_image.save_as_file(&savename)?;

  println!("Saved to: {}", savename);
  Ok(())
}
