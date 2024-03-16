#![no_std]
#![doc = include_str!("../README.md")]


const BMP_HEADER_SIZE: usize = 54;


/// Calculate the length of a BMP file of the given width and height.
///
/// The BMP format has a header and stores each row padded to 4 byte alignment,
/// so the final length will be larger than a simple (WIDTH * HEIGHT * 3) or (WIDTH * HEIGHT * 3 + BMP_HEADER_LENGTH) calculation.
///
/// This will be the minimum required size of the buffer passed to write_bmp.
/// Is a const function so it can be used as the size of a statically sized array.
///
/// ```rust
/// let mut buffer = [0u8; simple_bmp::buffer_length(100, 100)];
/// ```
pub const fn buffer_length(width: usize, height: usize) -> usize {
	let row_stride = (width * 3).next_multiple_of(4);
	let pixel_data_size = height * row_stride;
	BMP_HEADER_SIZE + pixel_data_size
}

/// Write a valid BMP file into the provided buffer, returning the number of bytes written.
///
/// The buffer can be longer than required. Extra space will remain untouched.
/// See documentation on the [Error] enum for possible errors.
pub fn write_bmp(buffer: &mut [u8], width: usize, height: usize, pixels: &[u8]) -> Result<usize, Error> {
	let row_stride = (width * 3).next_multiple_of(4);
	let pixel_data_size = height * row_stride;
	let file_length = BMP_HEADER_SIZE + pixel_data_size;

	if (i32::MAX as usize) < width {
		return Err(Error::WidthTooLarge { max: i32::MAX as usize, was: width });
	}

	if (i32::MAX as usize) < height {
		return Err(Error::HeightTooLarge { max: i32::MAX as usize, was: height });
	}

	if (u32::MAX as usize) < file_length {
		return Err(Error::FileLengthTooLong { max: u32::MAX as usize, would_be: file_length });
	}

	if pixels.len() != width * height * 3 {
		return Err(Error::BadPixelDataLength { expected: width * height * 3, was: pixels.len() });
	}

	if buffer.len() < file_length {
		return Err(Error::BufferTooSmall { required: file_length, was: buffer.len() });
	}

	// Header
	buffer[0..2].copy_from_slice(b"BM");
	buffer[2..][..4].copy_from_slice(&(file_length as u32).to_le_bytes());
	buffer[6..][..4].fill(0);
	buffer[10..][..4].copy_from_slice(&54u32.to_le_bytes());

	// DIB Header
	buffer[14..][..4].copy_from_slice(&40u32.to_le_bytes());
	buffer[18..][..4].copy_from_slice(&(width as i32).to_le_bytes());
	buffer[22..][..4].copy_from_slice(&(height as i32).to_le_bytes());
	buffer[26..][..2].copy_from_slice(&1u16.to_le_bytes());
	buffer[28..][..2].copy_from_slice(&24u16.to_le_bytes());
	buffer[30..][..4].copy_from_slice(&0u32.to_le_bytes());
	buffer[34..][..4].copy_from_slice(&(pixel_data_size as u32).to_le_bytes());
	buffer[38..][..4].copy_from_slice(&1000u32.to_le_bytes());
	buffer[42..][..4].copy_from_slice(&1000u32.to_le_bytes());
	buffer[46..][..4].copy_from_slice(&0u32.to_le_bytes());
	buffer[50..][..4].copy_from_slice(&0u32.to_le_bytes());

	// Pixel data
	for row in 0..height {
		let dst_begin = 54 + row_stride * row;
		let dst_end = dst_begin + width * 3;
		let src_begin = (height - row - 1) * width * 3;
		let src_end = src_begin + width * 3;
		buffer[dst_begin..dst_end].copy_from_slice(&pixels[src_begin..src_end]);
	}

	Ok(file_length)
}


/// Possible errors that can be returned by the write_bmp function
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
	/// Returned if the provided pixel data is not exactly (width * height * 3) bytes in length.
	BadPixelDataLength { expected: usize, was: usize },

	/// The BMP format stores the file length in a u32.
	/// This error is returned if the provided width & height would produce a file too large to store the length in the BMP header.
	FileLengthTooLong { max: usize, would_be: usize },

	/// Returned if the given buffer is too small to contain the BMP File.
	/// You can use [buffer_length] to determine how large the buffer needs to be.
	BufferTooSmall { required: usize, was: usize },

	/// The BMP file format stores the width in a signed i32.
	/// This error is returned if the given width doesn't fit in the BMP header.
	WidthTooLarge { max: usize, was: usize },

	/// The BMP file format stores the height in a signed i32.
	/// This error is returned if the given height doesn't fit in the BMP header.
	HeightTooLarge { max: usize, was: usize },
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bad_pixel_data_length() {
		const WIDTH: usize = 100;
		const HEIGHT: usize = 100;
		const PIXEL_LENGTH: usize = WIDTH * HEIGHT * 3;

		let mut buffer = [0u8; buffer_length(WIDTH, HEIGHT)];
		let pixels = [0u8; PIXEL_LENGTH - 1];
		let result = write_bmp(&mut buffer, WIDTH, HEIGHT, &pixels);
		assert_eq!(result, Err(Error::BadPixelDataLength { expected: PIXEL_LENGTH, was: PIXEL_LENGTH - 1 }));

		let mut buffer = [0u8; buffer_length(WIDTH, HEIGHT)];
		let pixels = [0u8; PIXEL_LENGTH + 1];
		let result = write_bmp(&mut buffer, WIDTH, HEIGHT, &pixels);
		assert_eq!(result, Err(Error::BadPixelDataLength { expected: PIXEL_LENGTH, was: PIXEL_LENGTH + 1 }));
	}

	#[test]
	fn bad_width() {
		let pixels = [0u8; 100 * 100 * 3];
		let mut buffer = [0u8; buffer_length(100, 100)];
		let width = i32::MAX as usize + 1;
		let result = write_bmp(&mut buffer, width, 100, &pixels);

		match result {
			Err(Error::WidthTooLarge { was, .. }) if was == width => {}
			otherwise => panic!("Width error is incorrect. {:?}", otherwise),
		}
	}

	#[test]
	fn bad_height() {
		let pixels = [0u8; 100 * 100 * 3];
		let mut buffer = [0u8; buffer_length(100, 100)];
		let height = i32::MAX as usize + 1;
		let result = write_bmp(&mut buffer, 100, height, &pixels);

		match result {
			Err(Error::HeightTooLarge { was, .. }) if was == height => {}
			otherwise => panic!("Height error is incorrect. {:?}", otherwise),
		}
	}

	#[test]
	fn bad_file_length() {
		let mut buffer = [0u8; 100];
		let pixels = [0u8; 100];
		let result = write_bmp(&mut buffer, 65535, 65535, &pixels);

		match result {
			Err(Error::FileLengthTooLong { .. }) => {}
			_ => assert!(false),
		}
	}

	#[test]
	fn bad_buffer_length() {
		let mut buffer = [0u8; 100];
		let pixels = [0u8; 100 * 100 * 3];
		let result = write_bmp(&mut buffer, 100, 100, &pixels);

		match result {
			Err(Error::BufferTooSmall { .. }) => {}
			_ => assert!(false),
		}
	}
}
