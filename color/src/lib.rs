#[cfg(test)]
mod tests {
    use crate::get_color;
    use ansi_term::Colour;
    use std::error::Error;

    #[test]
    fn color_decoding() {
        assert_eq!(
            get_color("#FFF").unwrap(),
            Colour::RGB(u8::MAX, u8::MAX, u8::MAX),
            "Decoding simple hex value failed!"
        );
        assert_eq!(
            get_color("#000000").unwrap(),
            Colour::RGB(0, 0, 0),
            "Decoding simple hex value failed!"
        );
    }

    #[test]
    #[should_panic]
    fn color_failing() -> () {
        get_color("#F").expect("Color decoding fail ");
    }

    #[test]
    fn color_not_failing() -> Result<(), Box<dyn Error>> {
        get_color("#FFF")?;
        Ok(())
    }
}

use ansi_term::Colour;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct HexDecodeError {
    msg: String,
}

impl Display for HexDecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for HexDecodeError {}

pub fn get_color(hexval: &str) -> Result<Colour, HexDecodeError> {
    let new_hexval = hexval.replace("#", "");
    if new_hexval.len() != 3 && new_hexval.len() != 6 {
        return Err(HexDecodeError {
            msg: format!(
                "Invalid hexval length: {}, must be 3 or 6",
                new_hexval.len()
            ),
        });
    }

    let values = new_hexval
        .split("")
        .filter(|&s| !s.is_empty())
        .map(|s| u8::from_str_radix(s, 16).map_err(|err| HexDecodeError { msg: err.to_string() }))
        .collect::<Result<Vec<u8>, HexDecodeError>>()?;

    let r: u8;
    let g: u8;
    let b: u8;

    if values.len() == 3 {
        r = values[0] << 4 | values[0];
        g = values[1] << 4 | values[1];
        b = values[2] << 4 | values[2];
    } else {
        r = values[0] << 4 | values[1];
        g = values[2] << 4 | values[3];
        b = values[4] << 4 | values[5];
    }

    Ok(Colour::RGB(r, g, b))
}
