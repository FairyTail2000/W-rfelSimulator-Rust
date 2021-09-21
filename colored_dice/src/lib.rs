use ansi_term::Colour;
use macros::dbgprintln;
use random_integer::random_usize;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const COLORED_DICES_FILE: &str = "colored.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ColoredDice {
    pub long: String,
    pub short: char,
    pub sites: [u8; 6],
    pub value: u8,
    pub color: String,
}

impl Clone for ColoredDice {
    fn clone(&self) -> Self {
        ColoredDice {
            long: self.long.to_string(),
            short: self.short.clone(),
            sites: self.sites.clone(),
            value: self.value.clone(),
            color: self.color.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ColoredDices {
    pub dices: Vec<ColoredDice>,
}

impl Clone for ColoredDices {
    fn clone(&self) -> Self {
        ColoredDices {
            dices: self.dices.to_vec(),
        }
    }
}

impl ColoredDices {
    pub fn load(file: Option<&str>) -> Self {
        let file_name = file.unwrap_or(COLORED_DICES_FILE);
        let exists = Path::new(file_name).exists();
        if exists {
            let file = File::open(file_name).unwrap();
            let buf_reader = BufReader::new(file);
            serde_yaml::from_reader::<BufReader<File>, ColoredDices>(buf_reader)
                .unwrap_or(ColoredDices::default())
        } else {
            match File::create(file_name) {
                Ok(file) => {
                    let writer = BufWriter::new(file);
                    match serde_yaml::to_writer::<BufWriter<File>, ColoredDices>(
                        writer,
                        &ColoredDices::default(),
                    ) {
                        Ok(_) => {
                            dbgprintln!("Neue Farbiger Würfel einstellungsdatei wurden erzeugt");
                        }
                        Err(err) => {
                            dbgprintln!("{}", Colour::RGB(255, 0, 0).paint(err.to_string()));
                        }
                    }
                }
                Err(err) => {
                    dbgprintln!("{}", Colour::RGB(255, 0, 0).paint(err.to_string()));
                }
            }
            ColoredDices::default()
        }
    }

    pub fn len(&self) -> usize {
        self.dices.len()
    }
}

impl ColoredDice {
    pub fn get_random(&self) -> &u8 {
        self.sites
            .get(random_usize(1, self.sites.len() - 1))
            .unwrap()
    }
}
