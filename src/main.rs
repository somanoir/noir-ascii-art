use image::{GenericImageView, GrayImage, Pixel};
use std::env;
use std::fs::File;
use std::io::{self, Write};

/// Braille Unicode characters map 2×4 dot cells to code points 0x2800‑0x28FF.
/// The bits are ordered as:
/// 0 3
/// 1 4
/// 2 5
/// 6 7
fn cell_to_char(dots: u8) -> char {
  // Unicode Braille pattern offset
  std::char::from_u32(0x2800 + dots as u32).unwrap_or(' ')
}

/// Convert a grayscale image to a binary (black/white) matrix.
/// `threshold` is the luminance value (0‑255) above which a pixel is considered white.
fn binarize(img: &GrayImage, threshold: u8) -> Vec<Vec<bool>> {
  let (w, h) = img.dimensions();
  let mut rows = Vec::with_capacity(h as usize);
  for y in 0..h {
    let mut row = Vec::with_capacity(w as usize);
    for x in 0..w {
      let luma = img.get_pixel(x, y).0[0];
      row.push(luma < threshold);
    }
    rows.push(row);
  }
  rows
}

/// Render the binary matrix as braille art.
/// Each braille cell covers 2 columns × 4 rows of pixels.
fn render_braille(matrix: &[Vec<bool>]) -> String {
  let height = matrix.len();
  let width = matrix[0].len();

  let mut output = String::new();

  // step through the image in 4‑pixel‑high blocks
  for y in (0..height).step_by(4) {
    // each line of output corresponds to a row of braille cells
    for x in (0..width).step_by(2) {
      let mut dots: u8 = 0;
      for dy in 0..4 {
        for dx in 0..2 {
          let py = y + dy;
          let px = x + dx;
          if py < height && px < width && matrix[py][px] {
            // map (dx,dy) to the correct bit index
            let bit = match (dx, dy) {
              (0, 0) => 0,
              (0, 1) => 1,
              (0, 2) => 2,
              (0, 3) => 6,
              (1, 0) => 3,
              (1, 1) => 4,
              (1, 2) => 5,
              (1, 3) => 7,
              _ => unreachable!(),
            };
            dots |= 1 << bit;
          }
        }
      }
      output.push(cell_to_char(dots));
    }
    output.push('\n');
  }
  output
}

fn main() -> io::Result<()> {
  // Expect: cargo run -- <path> [threshold] [scale]
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!(
      "Usage: {} <image> [threshold 0‑255] [scale 0.1‑10.0]",
      args[0]
    );
    std::process::exit(1);
  }

  let path = &args[1];
  let threshold: u8 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(128);
  let scale: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1.0);

  // Load image, convert to grayscale, optionally resize
  let img = image::open(path)
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    .to_luma8();
  let (w, h) = img.dimensions();
  let resized = if (scale - 1.0).abs() > f32::EPSILON {
    let new_w = (w as f32 * scale) as u32;
    let new_h = (h as f32 * scale) as u32;
    image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Lanczos3)
  } else {
    img
  };

  let binary = binarize(&resized, threshold);
  let art = render_braille(&binary);

  // Write to stdout (or a file if you prefer)
  let stdout = io::stdout();
  let mut handle = stdout.lock();
  handle.write_all(art.as_bytes())?;
  Ok(())
}
