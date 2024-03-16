A simple library for writing RGB pixels as a valid BMP file.

Sometimes, mostly when debugging, all you want to do is write some data as an image so you can look at it.
Every crate I could find was Way Too Complicated (TM).
What I wanted was a function to write some pixels into an image file that can be opened by an image viewer.
What I got were impossible-to-read types generic over the bit-depth, numeric type used to store the bits,
color channel order, and the image format; crates requiring 15 function calls before writing any data; APIs
forcing you to use stupid set_pixel functions instead of taking a simple slice of pixel data; and the kitchen sink.

This crate is the one function I wanted. Also it's no_std and no_alloc because requiring the standard library
when the core purpose of the crate doesn't require it is annoying.

# Example

```rust
const WIDTH: usize = 500;
const HEIGHT: usize = 500;

let mut pixels = [0u8; WIDTH * HEIGHT * 3];

// Draw a simple gradient
for y in 0..HEIGHT {
   for x in 0..WIDTH {
      pixels[y * WIDTH * 3 + x * 3 + 0] = 240;
      pixels[y * WIDTH * 3 + x * 3 + 1] = 100;
      pixels[y * WIDTH * 3 + x * 3 + 2] = y as u8;
   }
}

// Buffer for the BMP data
let mut image = [0u8; WIDTH * HEIGHT * 3 + simple_bmp::BMP_HEADER_SIZE];

// Write the pixels into the BMP buffer
simple_bmp::write_bmp(&mut image, WIDTH as u16, HEIGHT as u16, &pixels).unwrap();

// Maybe you want to store the BMP on disk
// std::fs::write("./image.bmp", &image).unwrap();
```