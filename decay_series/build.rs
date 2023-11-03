use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;

fn generate (src_file: &str, dest_file: &str) {
    let file = File::open(src_file).unwrap();
    let buf_reader = BufReader::new(file);
    let stuff: Vec<HashMap<String, String>> = match serde_yaml::from_reader::<BufReader<File>, Vec<HashMap<String, String>>>(buf_reader) {
        Ok(disadvantages) => disadvantages,
        Err(_) => {
            [].to_vec()
        }
    };

    let mut buffer = String::new();
    buffer += "vec![";
    for stuf in stuff {
        let mut electrons = stuf.get("electrons").unwrap().to_owned();
        let mut protons = stuf.get("protons").unwrap().to_owned();
        let mut neutrons = stuf.get("neutrons").unwrap().to_owned();

        if electrons == "~" {
            electrons = String::from("None");
        } else {
            electrons = String::from(format!("Some({})", electrons));
        }

        if protons == "~" {
            protons = String::from("None");
        } else {
            protons = String::from(format!("Some({})", protons));
        }

        if neutrons == "~" {
            neutrons = String::from("None");
        } else {
            neutrons = String::from(format!("Some({})", neutrons));
        }

        buffer += &*format!("Operation {} display: \"{}\".parse().unwrap(), electrons: {}, protons: {}, neutrons: {}, {},", "{", stuf.get("display").unwrap().escape_default(), electrons, protons, neutrons, "}")
    }

    buffer += "]";
    let mut f = File::create(dest_file).unwrap();
    match write!(f, "{}", buffer) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn main() {
    generate("decay_series.yaml", "src/default_operation.rs");
}