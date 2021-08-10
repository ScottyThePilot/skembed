use image::{RgbaImage, Rgba};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

pub fn extract_data(image: &RgbaImage) -> Vec<u8> {
  if image.dimensions() != (64, 64) {
    panic!("Input is not 64x64");
  };

  // Extract the length of the data from the marker pixel
  let data_len = {
    let [mx, my] = MARKER_PIXEL;
    let marker_pixel = image.get_pixel(mx, my);
    u16::from_le_bytes([marker_pixel[0], marker_pixel[1]])
  } as usize;

  if data_len == 0 {
    return Vec::new();
  };

  let mut bytes = Vec::with_capacity(REGIONS_BYTES as usize);

  // Read all of the data (including garbage) into a buffer
  for [x, y] in void_pixels(false) {
    let pixel = image.get_pixel(x, y).0;
    bytes.extend_from_slice(&pixel[0..3]);
  };

  // Erase all of the garbage data
  bytes.truncate(data_len);

  bytes
}

pub fn embed_data(image: &mut RgbaImage, mut bytes: Vec<u8>) {
  if image.dimensions() != (64, 64) {
    panic!("Input is not 64x64");
  };

  if bytes.len() > REGIONS_BYTES as usize {
    panic!("Data is too large to be embedded");
  };

  let data_len = bytes.len() as u16;

  // Clear void areas
  for [x, y] in void_pixels(true) {
    image.put_pixel(x, y, Rgba([0; 4]));
  };

  // Create an RNG from the actual skin data
  let hash = get_hash(image.as_raw());
  let mut rng = Xoshiro256PlusPlus::from_seed(hash);

  // Place the marker pixel indicating length of the data
  let [mx, my] = MARKER_PIXEL;
  image.put_pixel(mx, my, {
    let [a, b] = data_len.to_le_bytes();
    Rgba([a, b, rng.gen::<u8>(), 0xff])
  });

  while bytes.len() % 3 != 0 {
    bytes.push(rng.gen::<u8>());
  };

  // Place bytes within the image
  for (bytes, [x, y]) in bytes.chunks_exact(3).zip(void_pixels(false)) {
    image.put_pixel(x, y, Rgba([bytes[0], bytes[1], bytes[2], 0xff]));
  };
}

pub fn embed_data_clear(image: &mut RgbaImage) {
  assert_eq!(image.dimensions(), (64, 64), "Input is not 64x64");

  for [x, y] in void_pixels(true) {
    image.put_pixel(x, y, Rgba([0; 4]));
  };
}

pub fn get_hash(data: &[u8]) -> [u8; 32] {
  *blake3::hash(data).as_bytes()
}



fn void_pixels(include_marker: bool) -> Vec<[u32; 2]> {
  let mut pixels = Vec::with_capacity(REGIONS_PIXEL_COUNT as usize + 1);

  for region in REGIONS {
    for y in region.min[1]..region.max[1] {
      for x in region.min[0]..region.max[0] {
        if !include_marker && [x, y] != MARKER_PIXEL {
          pixels.push([x, y]);
        };
      };
    };
  };

  pixels
}

pub struct Region {
  min: [u32; 2],
  max: [u32; 2]
}

const MARKER_PIXEL: [u32; 2] = [0, 0];

const REGIONS_BYTES: u32 = REGIONS_PIXEL_COUNT * 3;
const REGIONS_PIXEL_COUNT: u32 = 832 - 1;
const REGIONS: &'static [Region] = &[
  Region { min: [0, 0], max: [8, 8] },
  Region { min: [24, 0], max: [40, 8] },
  Region { min: [56, 0], max: [64, 8] },

  Region { min: [0, 16], max: [4, 20] },
  Region { min: [12, 16], max: [20, 20] },
  Region { min: [36, 16], max: [44, 20] },
  Region { min: [52, 16], max: [56, 20] },

  Region { min: [0, 32], max: [4, 36] },
  Region { min: [12, 32], max: [20, 36] },
  Region { min: [36, 32], max: [44, 36] },
  Region { min: [52, 32], max: [56, 36] },

  Region { min: [0, 48], max: [4, 52] },
  Region { min: [12, 48], max: [20, 52] },
  Region { min: [28, 48], max: [36, 52] },
  Region { min: [44, 48], max: [52, 52] },
  Region { min: [60, 48], max: [64, 52] },

  Region { min: [56, 16], max: [64, 48] }
];
