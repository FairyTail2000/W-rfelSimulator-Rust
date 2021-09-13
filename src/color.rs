use ansi_term::Colour;

pub fn get_color(hexval: &str) -> Colour {
    let new_hexval = hexval.replace("#", "");
    let parse: Vec<u8> = new_hexval
        .chars()
        .map(|val| u8::from_str_radix(&*val.to_string(), 16).unwrap())
        .collect();
    let r: u8;
    let g: u8;
    let b: u8;

    if parse.len() == 3 {
        r = parse[0] << 4 | parse[0];
        g = parse[1] << 4 | parse[1];
        b = parse[2] << 4 | parse[2];
    } else {
        r = parse[0] << 4 | parse[1];
        g = parse[2] << 4 | parse[3];
        b = parse[4] << 4 | parse[5];
    }

    return Colour::RGB(r, g, b);
}
