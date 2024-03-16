#![no_std]
#![doc = include_str!("../README.md")]


/// Number of bytes required to encode the BMP header.
///
/// The buffer passed to write_bmp() will need to be the size of your pixel data + this many bytes
pub const BMP_HEADER_SIZE: usize = 54;


/// Write a valid BMP file into the provided buffer, returning the number of bytes written.
///
/// The buffer can be longer than required. Extra space will remain untouched.
/// See documentation on the [Error] enum for possible errors.
pub fn write_bmp(buffer: &mut [u8], width: u16, height: u16, pixels: &[u8]) -> Result<usize, Error> {
	let width = width as usize;
	let height = height as usize;

	let row_stride = (width * 3).next_multiple_of(4);
	let pixel_data_size = height * row_stride;
	let file_length = BMP_HEADER_SIZE + pixel_data_size;

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
	/// Returned if the provided pixel data is not exactly (width * height * 3) bytes in length
	BadPixelDataLength { expected: usize, was: usize },

	/// The BMP format stores the file length in a u32.
	/// This error is returned if the provided width & height would produce a file too large to store the length in the BMP header.
	FileLengthTooLong { max: usize, would_be: usize },

	/// Returned if the given buffer is too small to contain the BMP File
	BufferTooSmall { required: usize, was: usize },
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bad_pixel_data_length() {
		const WIDTH: usize = 100;
		const HEIGHT: usize = 100;
		const LENGTH: usize = WIDTH * HEIGHT * 3;

		let mut buffer = [0u8; LENGTH + BMP_HEADER_SIZE];
		let pixels = [0u8; LENGTH - 1];
		let result = write_bmp(&mut buffer, WIDTH as u16, HEIGHT as u16, &pixels);
		assert_eq!(result, Err(Error::BadPixelDataLength { expected: LENGTH, was: LENGTH - 1 }));

		let mut buffer = [0u8; LENGTH + BMP_HEADER_SIZE];
		let pixels = [0u8; LENGTH + 1];
		let result = write_bmp(&mut buffer, WIDTH as u16, HEIGHT as u16, &pixels);
		assert_eq!(result, Err(Error::BadPixelDataLength { expected: LENGTH, was: LENGTH + 1 }));
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
