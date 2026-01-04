use ansi_term::Colour;
use crate::common::{settings_path, Loadable, Rollable};
use crate::dbgprintln;
use rand::Rng;
use rand::distr::Uniform;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const COLORED_DICES_FILE: &str = "colored.yaml";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawColoredDice")]
pub struct ColoredDice {
	pub long: String,
	pub short: char,
	pub sites: [u8; 6],
	pub value: u8,
	pub color: String,
	#[serde(skip)]
	range: Uniform<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawColoredDice {
	pub long: String,
	pub short: char,
	pub sites: [u8; 6],
	pub value: u8,
	pub color: String,
}

impl From<RawColoredDice> for ColoredDice {
	fn from(raw: RawColoredDice) -> Self {
		Self {
			long: raw.long,
			short: raw.short,
			sites: raw.sites,
			value: raw.value,
			color: raw.color,
			range: Uniform::<usize>::new_inclusive(0, raw.sites.len() - 1).expect("Failed to create uniform distribution for colored dice"),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColoredDices {
	pub dices: Vec<ColoredDice>,
}

impl Default for ColoredDices {
	fn default() -> Self {
		ColoredDices {
			dices: vec![
				ColoredDice {
					long: "Rosa".to_string(),
					short: 'r',
					sites: [0, 0, 0, 1, 1, 2],
					value: 1,
					color: "#FF8B8B".to_string(),
					range: Uniform::new_inclusive(0, 5).expect("Failed to create uniform distribution for colored dice")
				},
				ColoredDice {
					long: "Grün".to_string(),
					short: 'g',
					sites: [0, 0, 1, 1, 2, 2],
					value: 2,
					color: "#22FF00".to_string(),
					range: Uniform::new_inclusive(0, 5).expect("Failed to create uniform distribution for colored dice")
				},
				ColoredDice {
					long: "Weiß".to_string(),
					short: 'w',
					sites: [0, 1, 2, 2, 2, 3],
					value: 3,
					color: "#FFFFFF".to_string(),
					range: Uniform::new_inclusive(0, 5).expect("Failed to create uniform distribution for colored dice")
				},
				ColoredDice {
					long: "Schwarz".to_string(),
					short: 's',
					sites: [0, 1, 3, 3, 3, 4],
					value: 4,
					color: "#818181".to_string(),
					range: Uniform::new_inclusive(0, 5).expect("Failed to create uniform distribution for colored dice")
				},
			],
		}
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

impl Rollable<u8> for ColoredDice {
    fn roll(&self, rng: &mut impl Rng) -> u8 {
        if self.sites.is_empty() {
            return 0;
        }
        self.sites[rng.sample(&self.range)]
    }
}
