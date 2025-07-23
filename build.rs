use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader};
use serde_yaml;

fn generate_disadvantage(src_file: &str, dest_file: &str, name: &str) {
    let file = File::open(src_file).unwrap();
    let buf_reader = BufReader::new(file);
    let stuff: Vec<HashMap<String, String>> = serde_yaml::from_reader::<BufReader<File>, Vec<HashMap<String, String>>>(buf_reader).unwrap_or_else(|_| vec![]);

    let mut buffer = String::new();
    buffer += "vec![";
    for stuf in stuff {
        buffer += &*format!("{} {} name: \"{}\".to_string(), description: \"{}\".to_string() {},", name, "{", stuf.get("name").unwrap().escape_default(), stuf.get("description").unwrap().escape_default(), "}")
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

fn generate_decay_series(src_file: &str, dest_file: &str) {
    let file = File::open(src_file).unwrap();
    let buf_reader = BufReader::new(file);
    let stuff: Vec<HashMap<String, String>> = serde_yaml::from_reader::<BufReader<File>, Vec<HashMap<String, String>>>(buf_reader).unwrap_or_else(|_| vec![]);

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

        buffer += &*format!("Operation {} display: \"{}\".to_string(), electrons: {}, protons: {}, neutrons: {}, {},", "{", stuf.get("display").unwrap().escape_default(), electrons, protons, neutrons, "}")
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
    generate_disadvantage("disadvantage.yaml", "src/default_disadvantage.rs", "Disadvantage");
    generate_decay_series("decay_series.yaml", "src/decay_series/default_operation.rs");
}