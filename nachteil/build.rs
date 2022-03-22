use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader};
use serde_yaml;

fn generate (src_file: &str, dest_file: &str, name: &str) {
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
        buffer += &*format!("{} {} name: \"{}\".parse().unwrap(), description: \"{}\".parse().unwrap() {},", name, "{", stuf.get("name").unwrap().escape_default(), stuf.get("description").unwrap().escape_default(), "}")
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
    generate("disadvantage.yaml", "src/default_disadvantage.rs", "Disadvantage");
    generate("advantage.yaml", "src/default_advantage.rs", "Advantage");
}