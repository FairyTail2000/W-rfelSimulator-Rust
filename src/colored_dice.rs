use random_integer::random_usize;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

const COLORED_DICES_FILE: &str = "colored.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ColoredDice {
    pub(crate) long: String,
    pub(crate) short: char,
    pub(crate) sites: [u8; 6],
    pub(crate) value: u8,
    pub(crate) color: String,
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
    pub(crate) dices: Vec<ColoredDice>,
}

impl Clone for ColoredDices {
    fn clone(&self) -> Self {
        ColoredDices {
            dices: self.dices.to_vec(),
        }
    }
}

impl ColoredDices {
    pub fn load() -> Self {
        let exists = Path::new(COLORED_DICES_FILE).exists();
        return if exists {
            let file = File::open(COLORED_DICES_FILE).unwrap();
            let buf_reader = BufReader::new(file);
            let parsed = serde_yaml::from_reader::<BufReader<File>, ColoredDices>(buf_reader);
            if let Ok(result) = parsed {
                result
            } else {
                ColoredDices::default()
            }
        } else {
            ColoredDices::default()
        };
    }

    pub fn len(&self) -> usize {
        return self.dices.len();
    }
}

impl ColoredDice {
    pub fn get_random(&self) -> &u8 {
        self.sites
            .get(random_usize(1, self.sites.len() - 1))
            .unwrap()
    }
}
