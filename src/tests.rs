#[cfg(test)]
mod tests {
    use crate::color::get_color;
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
