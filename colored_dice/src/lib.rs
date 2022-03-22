use ansi_term::Colour;
use common::{settings_path, Loadable, Rollable};
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

impl Default for ColoredDices {
	fn default() -> Self {
		return ColoredDices {
			dices: vec![
				ColoredDice {
					long: "Rosa".parse().unwrap(),
					short: "r".parse().unwrap(),
					sites: [0, 0, 0, 1, 1, 2],
					value: 1,
					color: "#FF8B8B".parse().unwrap(),
				},
				ColoredDice {
					long: "Grün".parse().unwrap(),
					short: "g".parse().unwrap(),
					sites: [0, 0, 1, 1, 2, 2],
					value: 2,
					color: "#22FF00".parse().unwrap(),
				},
				ColoredDice {
					long: "Weiß".parse().unwrap(),
					short: "w".parse().unwrap(),
					sites: [0, 1, 2, 2, 2, 3],
					value: 3,
					color: "#FFFFFF".parse().unwrap(),
				},
				ColoredDice {
					long: "Schwarz".parse().unwrap(),
					short: "s".parse().unwrap(),
					sites: [0, 1, 3, 3, 3, 4],
					value: 4,
					color: "#818181".parse().unwrap(),
				},
			],
		};
	}
}

impl Loadable<Self> for ColoredDices {
	fn load(file: Option<&str>) -> Self {
		let alt = settings_path(COLORED_DICES_FILE);
		let file_name = file.unwrap_or(alt.to_str().unwrap());
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
							dbgprintln!("Neue Farbiger Würfel wurden erzeugt");
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
}

impl Rollable<&u8> for ColoredDice {
    fn roll(&self) -> &u8 {
        self.sites
            .get(random_usize(1, self.sites.len() - 1))
            .unwrap()
    }
}

impl ColoredDices {
	pub fn len(&self) -> usize {
		self.dices.len()
	}
}