use ansi_term::Colour;
use raster::Color;

pub fn get_color(hexval: &str) -> Colour {
	let parsed = Color::hex(hexval);
	assert!(parsed.is_ok());

	let color = parsed.unwrap();

	return Colour::RGB(color.r, color.g, color.b);
}