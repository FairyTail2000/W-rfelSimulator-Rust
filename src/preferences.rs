use serde::{Serialize, Deserialize};
use random_integer::random_usize;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;

const PREFERENCE_FILE: &str = "settings.yaml";
const COLORED_DICES_FILE: &str = "colored.yaml";
const NORMAL_DICES_FILE: &str = "normal.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
	old_style: bool,
	no_tutorial: bool,
	no_summary_message: bool,
	no_select_dice_select: bool
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ColoredDice {
	pub(crate) long: String,
	pub(crate) short: char,
	pub(crate) sites: [u8; 6],
	pub(crate) value: u8
}

impl Clone for ColoredDice {
	fn clone(&self) -> Self {
		ColoredDice {
			long: self.long.to_string(),
			short: self.short.clone(),
			sites: self.sites.clone(),
			value: self.value.clone()
		}
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ColoredDices {
	pub(crate) dices: Vec<ColoredDice>
}

impl Clone for ColoredDices {
	fn clone(&self) -> Self {
		ColoredDices {
			dices: self.dices.to_vec()
		}
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Dice {
	pub sides: u8
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Dices {
	pub dices: Vec<Dice>
}

impl Dices {
	pub fn load() -> Self {
		let exists = Path::new(NORMAL_DICES_FILE).exists();
		return if exists {
			let file = File::open(NORMAL_DICES_FILE).unwrap();
			let buf_reader = BufReader::new(file);
			let parsed = serde_yaml::from_reader::<BufReader<File>, Dices>(buf_reader);
			if let Ok(result) = parsed {
				result
			} else {
				Dices::default()
			}
		} else {
			Dices::default()
		}
	}

	pub fn len(&self) -> usize {
		return self.dices.len();
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
		}
	}

	pub fn len(&self) -> usize {
		return self.dices.len();
	}
}

impl ColoredDice {
	pub fn get_random(&self) -> &u8 {
		self.sites.get(random_usize(1, self.sites.len() - 1)).unwrap()
	}
}

impl Settings {
	pub fn load() -> Self {
		let exists = Path::new(PREFERENCE_FILE).exists();
		return if exists {
			let file = File::open(PREFERENCE_FILE).unwrap();
			let buf_reader = BufReader::new(file);
			let parsed = serde_yaml::from_reader::<BufReader<File>, Settings>(buf_reader);
			if let Ok(result) = parsed {
				result
			} else {
				Settings::default()
			}
		} else {
			Settings::default()
		}
	}
}

// TODO actually use preferences
// TODO define a format for the preferences