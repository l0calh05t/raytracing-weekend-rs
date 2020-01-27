use nd::prelude::*;
use ndarray as nd;

use crate::vec::*;

pub fn write_bmp(file_name: &str, image: ArrayView2<Vec3>) -> std::io::Result<()> {
	use byteorder::{LittleEndian, WriteBytesExt};
	use std::io::Write;

	let line_data_bytes = 3 * image.shape()[1];
	let line_bytes = 4 * ((line_data_bytes + 3) / 4);
	let padding = &b"\0\0\0"[..(line_bytes - line_data_bytes)];
	let data_offset: u32 = 14 + 40;
	let total_bytes: u32 = data_offset + (image.shape()[0] * line_bytes) as u32;

	let mut output = std::io::BufWriter::with_capacity(4 << 20, std::fs::File::create(file_name)?);

	// main bitmap header
	output.write_all(b"BM")?;
	output.write_u32::<LittleEndian>(total_bytes)?;
	output.write_all(b"\0\0\0\0")?;
	output.write_u32::<LittleEndian>(data_offset)?;

	// BITMAPINFOHEADER
	output.write_u32::<LittleEndian>(40)?; // header size
	output.write_i32::<LittleEndian>(image.shape()[1] as i32)?; // width
	output.write_i32::<LittleEndian>(image.shape()[0] as i32)?; // height
	output.write_u16::<LittleEndian>(1)?; // number of color planes (always 1)
	output.write_u16::<LittleEndian>(24)?; // bits per pixel
	output.write_u32::<LittleEndian>(0)?; // uncompressed rgb
	output.write_u32::<LittleEndian>(0)?; // image data size (0 for uncompressed)
	output.write_i32::<LittleEndian>(2835)?; // 72 DPI in pixels per meter
	output.write_i32::<LittleEndian>(2835)?; // 72 DPI in pixels per meter
	output.write_u32::<LittleEndian>(0)?; // no palette
	output.write_u32::<LittleEndian>(0)?; // no important colors

	// image data
	for row in image.outer_iter() {
		for color in row.iter() {
			output.write_all(&[
				(color.z.max(0.0).min(1.0).sqrt() * 255.0).round() as u8,
				(color.y.max(0.0).min(1.0).sqrt() * 255.0).round() as u8,
				(color.x.max(0.0).min(1.0).sqrt() * 255.0).round() as u8,
			])?;
		}
		output.write_all(padding)?;
	}

	output.flush()?;
	Ok(())
}
