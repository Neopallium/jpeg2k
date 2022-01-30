use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use image::DynamicImage;
use jpeg2k::*;

#[inline]
fn components_to_pixels(r: &[i32], g: &[i32], b: &[i32], a: &[i32]) -> Vec<u8> {
  let len = r.len().min(g.len()).min(b.len()).min(a.len());
  let mut pixels = Vec::with_capacity(len * 4);
  for (r, (g, (b, a))) in r.iter().zip(g.iter().zip(b.iter().zip(a.iter()))) {
    pixels.extend_from_slice(&[*r as u8, *g as u8, *b as u8, *a as u8]);
  }
  pixels
}

#[inline]
fn components_to_pixels_flat_map(r: &[i32], g: &[i32], b: &[i32], a: &[i32]) -> Vec<u8> {
  r.iter()
    .zip(g.iter().zip(b.iter().zip(a.iter())))
    .flat_map(|(r, (g, (b, a)))| [*r as u8, *g as u8, *b as u8, *a as u8])
    .collect()
}

fn generate_component(width: u32, height: u32) -> Vec<i32> {
  (0..width)
    .zip(0..height)
    .map(|(x, y)| (x + y) as i32)
    .collect()
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let size = 1024;
  let r = generate_component(size, size);
  let g = generate_component(size, size);
  let b = generate_component(size, size);
  let a = generate_component(size, size);

  c.bench_function("components_to_pixels 1024x1024", |bench| {
    bench.iter_with_large_drop(|| {
      components_to_pixels(r.as_slice(), g.as_slice(), b.as_slice(), a.as_slice())
    })
  });

  c.bench_function("components_to_pixels_flat_map 1024x1024", |bench| {
    bench.iter_with_large_drop(|| {
      components_to_pixels_flat_map(r.as_slice(), g.as_slice(), b.as_slice(), a.as_slice())
    })
  });

  let file_name =
    "samples/Hadley_Crater_provides_deep_insight_into_martian_geology_(7942261196).jp2";
  let jp2_img = Image::from_file(&file_name).expect("Failed to load sample image");
  c.bench_with_input(
    BenchmarkId::new("jp2_to_DynamicImage", &file_name),
    &jp2_img,
    |bench, jp2| {
      bench.iter_with_large_drop(|| {
        let img: DynamicImage = jp2.try_into().expect("Failed to convert image");
        img
      })
    },
  );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
