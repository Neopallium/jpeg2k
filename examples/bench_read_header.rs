use std::fs::File;
use std::io::Read;
use std::env;

use rayon::prelude::*;

use anyhow::Result;

use jpeg2k::*;

fn main() -> Result<()> {
  dotenv::dotenv().ok();
  env_logger::init();

  let jp2_filename = env::args().nth(1).unwrap_or_else(|| "test.j2k".to_string());
  let reduce = env::args()
    .nth(2)
    .unwrap_or_else(|| "0".to_string())
    .parse::<u32>()
    .expect("Reduce must be an integer.");
  let layers = env::args()
    .nth(3)
    .unwrap_or_else(|| "0".to_string())
    .parse::<u32>()
    .expect("Layers must be an integer.");

  let repeat = env::args()
    .nth(4)
    .unwrap_or_else(|| "100".to_string())
    .parse::<u64>()
    .expect("Repeat must be an integer.");

  // Decode parameters.
  let params = DecodeParameters::new().reduce(reduce).layers(layers);

  let mut file = File::open(jp2_filename)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  let imgs = (0..repeat).into_par_iter().map(|_i| {
    let jp2 = DumpImage::from_bytes_with(buf.as_slice(), params.clone())
      .expect("Image read header.");

    (jp2.img.num_components(), jp2.img.width(), jp2.img.height())
  }).collect::<Vec<_>>();

  let mut total_components = 0;
  let mut is_first = true;
  let mut width = 0;
  let mut height = 0;
  for (i_numcomps, i_width, i_height) in imgs {
    if is_first {
      is_first = false;
      width = i_width;
      height = i_height;
    }
    total_components += i_numcomps;
    assert_eq!(i_width, width);
    assert_eq!(i_height, height);
  }
  println!("Total components: {total_components}");

  Ok(())
}
