use std::env;

use anyhow::Result;

use jpeg2k::*;

fn main() -> Result<()> {
  dotenv::dotenv().ok();
  env_logger::init();

  let jp2_filename = env::args().nth(1)
    .unwrap_or_else(|| "test.j2k".to_string());
  let reduce = env::args().nth(2)
    .unwrap_or_else(|| "0".to_string())
    .parse::<u32>().expect("Reduce must be an integer.");
  let layers = env::args().nth(3)
    .unwrap_or_else(|| "0".to_string())
    .parse::<u32>().expect("Layers must be an integer.");

  // Decode parameters.
  let params = DecodeParameters::new()
    .reduce(reduce)
    .layers(layers)
    ;

  /*
  let mut file = File::open(jp2_filename)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  let jp2_image = DumpImage::from_bytes_with(&mut buf, params)?;
  */
  let jp2_image = DumpImage::from_file_with(jp2_filename, params)?;

  println!("dump image: {:#?}", jp2_image.img);
  println!("dump index: {:#?}", jp2_image.get_codestream_index());
  println!("dump info: {:#?}", jp2_image.get_codestream_info());

  // Start decoding the image.  Decoding needs to be started to get
  // the codestream index, even if `start_decode()` returns an error.
  let res = jp2_image.start_decode();
  println!("image decoded = {:?}", res);

  println!("dump index: {:#?}", jp2_image.get_codestream_index());

  Ok(())
}
