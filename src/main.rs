extern crate blake3;
#[macro_use]
extern crate clap;
extern crate image;
extern crate rand;
extern crate rand_xoshiro;

mod regions;

use image::{RgbaImage, DynamicImage, ColorType};
use image::codecs::png::{
  CompressionType, FilterType,
  PngDecoder, PngEncoder
};

use std::path::Path;
use std::fs::{self, File};
use std::io::{self, Write};

fn main() {
  let mut app = clap_app!(skembed =>
    (version: crate_version!())
    (author: crate_authors!())
    (about: crate_description!())
    (@subcommand extract =>
      (about: "Extract data from a Minecraft skin")
      (@arg INPUT: +required "The skin to extract data from")
      (@arg OUTPUT: -o --output +takes_value "Indicates the file to write extracted data to, omit to output to console")
    )
    (@subcommand embed =>
      (about: "Embed data within a Minecraft skin")
      (@arg INPUT: +required "The skin to embed data in")
      (@arg DATA: -d --data +takes_value "Indicates the file which should be embedded")
      (@arg CLEAR: -c --clear "Indicates that the embedded data should be cleared")
      (@arg OUTPUT: -o --output +takes_value "Indicates the location to write the skin file to")
      (@arg OVERWRITE: -w --overwrite "Indicates that the input skin should be overwritten")
    )
  );

  let mut long_help: Vec<u8> = Vec::new();
  app.write_long_help(&mut long_help).unwrap();

  let matches = app.get_matches();

  match matches.subcommand() {
    ("extract", Some(matches)) => {
      let input = Path::new(matches.value_of_os("INPUT").unwrap());
      let output = matches.value_of_os("OUTPUT").map(Path::new);
      extract(input, output);
    },
    ("embed", Some(matches)) => {
      let input = Path::new(matches.value_of_os("INPUT").unwrap());

      let data = matches.value_of_os("DATA").map(Path::new);
      let clear = matches.is_present("CLEAR");
      // Some: write data found in file, None: clear
      let data_mode = get_data_mode(data, clear);

      let output = matches.value_of_os("OUTPUT").map(Path::new);
      let overwrite = matches.is_present("OVERWRITE");
      // Some: output to path, None: overwrite
      let output_mode = get_output_mode(output, overwrite);

      embed(input, data_mode, output_mode);
    },
    (_, Some(_)) => unreachable!("no other subcommands"),
    (_, None) => {
      io::stdout().write_all(&long_help).unwrap();
    }
  };
}

fn extract(input: &Path, path: Option<&Path>) {
  let image = read_image(input);
  let extracted_data = regions::extract_data(&image);

  match path {
    Some(path) => {
      match String::from_utf8(extracted_data) {
        Ok(extracted_data) => fs::write(path, extracted_data).unwrap(),
        Err(_) => println!("error: Extracted data is not valid UTF-8")
      };
    },
    None => {
      let extracted_data = String::from_utf8_lossy(&extracted_data);
      println!("{}", extracted_data);
    }
  };
}

fn embed(input: &Path, data_mode: Option<&Path>, output_mode: Option<&Path>) {
  let mut image = read_image(input);
  let output = if let Some(p) = output_mode { p } else { input };

  if let Some(path) = data_mode {
    let data = fs::read(path).unwrap();
    regions::embed_data(&mut image, data);
  } else {
    regions::embed_data_clear(&mut image);
  };

  println!("{:?}", regions::get_hash(image.as_raw()));

  write_image(output, &image);
}

fn read_image(path: &Path) -> RgbaImage {
  let img_file = File::open(path).unwrap();
  let img_file = PngDecoder::new(img_file).unwrap();
  let img = DynamicImage::from_decoder(img_file).unwrap();
  img.into_rgba8()
}

fn write_image(path: &Path, image: &RgbaImage) {
  let img_file = File::create(path).unwrap();
  let img_file = PngEncoder::new_with_quality(img_file, CompressionType::Best, FilterType::default());
  let (width, height) = image.dimensions();
  img_file.encode(image.as_raw(), width, height, ColorType::Rgba8).unwrap();
}

fn get_data_mode(data: Option<&Path>, clear: bool) -> Option<&Path> {
  if let Some(data) = data {
    Some(data)
  } else if clear {
    None
  } else {
    produce_error("One of 'data' or 'clear' must be provided");
  }
}

fn get_output_mode(output: Option<&Path>, overwrite: bool) -> Option<&Path> {
  if let Some(output) = output {
    Some(output)
  } else if overwrite {
    None
  } else {
    produce_error("One of 'output' or 'overwrite' must be provided");
  }
}

fn produce_error(msg: &str) -> ! {
  println!("error: {}\n\nFor more information try --help", msg);
  std::process::exit(1)
}
