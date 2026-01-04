use crate::common::{settings_path, Loadable, Rollable};
use crate::dbgprintln;
use rand::Rng;
use rand::distr::Uniform;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(from = "RawCritDice")]
pub struct CritDice {
	pub value: u8,
	pub values: [u8; 6],
	#[serde(skip)]
	pub distribution: Uniform<usize>
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct RawCritDice {
	pub value: u8,
	pub values: [u8; 6]
}

impl From<RawCritDice> for CritDice {
	fn from(raw: RawCritDice) -> Self {
		Self {
			value: raw.value,
			values: raw.values,
			distribution: Uniform::<usize>::new_inclusive(0, raw.values.len() - 1).expect("Failed to create uniform distribution for crit dices"),
		}
	}
}


#[derive(PartialEq, Serialize, Deserialize, Debug, PartialOrd, Copy, Clone)]
pub struct Level {
	pub lower: u8,
	pub upper: Option<u8>,
	pub percentage: f32,
}

impl Display for Level {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let up = self.upper
			.and_then(|val| Some(val.to_string()))
			.unwrap_or_else(|| "Keine Begrenzung".to_string());
		write!(f, "Level {} - {}", self.lower, up)
	}
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct CritDices {
	pub dices: Vec<CritDice>,
	pub level: Vec<Level>,
	pub s: u8,
}

impl Default for CritDices {
	fn default() -> Self {
		CritDices {
			level: vec![
				Level {
					lower: 0,
					upper: Some(9),
					percentage: 50f32,
				},
				Level {
					lower: 10,
					upper: Some(19),
					percentage: 66.666,
				},
				Level {
					lower: 20,
					upper: None,
					percentage: 83.333,
				},
			],
			dices: vec![
				RawCritDice {
					value: 1,
					values: [0, 0, 0, 0, 0, 1],
				}.into(),
				RawCritDice {
					value: 2,
					values: [0, 0, 0, 0, 1, 2],
				}.into(),
				RawCritDice {
					value: 3,
					values: [0, 0, 0, 0, 3, 2],
				}.into(),
				RawCritDice {
					value: 4,
					values: [0, 0, 1, 2, 3, 4],
				}.into(),
			],
			s: 4,
		}
	}
}

impl Level {
	fn works(&self, rng: &mut impl Rng) -> bool {
		let uniform = Uniform::new_inclusive(0f32, 100f32).expect("Failed to create uniform distribution for crits");
		let random_value = rng.sample(uniform);
		self.percentage > random_value
	}
}

impl Rollable<u8> for CritDice {
    fn roll(&self, rng: &mut impl Rng) -> u8 {
		let random_value = rng.sample(&self.distribution);
        self.values[random_value]
    }
}

impl Loadable<CritDices> for CritDices {
	fn load(file: Option<&str>) -> CritDices {
		let alt = settings_path("crits.yaml");
		let file_name = file.unwrap_or(alt.to_str().unwrap());
		if Path::new(file_name).exists() {
			let file = File::open(file_name).unwrap();
			let buf_reader = BufReader::new(file);
			match serde_yaml::from_reader::<BufReader<File>, CritDices>(buf_reader) {
				Ok(spells) => spells,
				Err(err) => {
					eprintln!("{}", err);
					let file = OpenOptions::new()
						.write(true)
						.truncate(true)
						.open(file_name)
						.unwrap();
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer(writer, &CritDices::default()) {
						Ok(_) => {}
						Err(err) => {
							eprintln!("Couldn't default values to file");
							eprintln!("{}", err);
						}
					}
					CritDices::default()
				}
			}
		} else {
			CritDices::default()
		}
	}
}

impl CritDices {
	pub fn roll(&self, value: i16, rng: &mut impl Rng) {
		let levels: Vec<(&Level, bool)> = self
			.level
			.iter()
			.map(|x| (x, x.works(rng)))
			.filter(|x| x.1)
			.collect();
		let mut stack: Vec<&CritDice> = Vec::with_capacity(10);
		let mut counter: i16 = value;
		while counter != 0 {
			for item in self.dices.iter() {
				if counter - item.value as i16 > -1 {
					stack.push(item);
					counter -= item.value as i16;
					break;
				}
			}
		}
		let results: Vec<u8> = stack.iter_mut().map(|x| x.roll(rng)).collect();
		// How many "S" where found in the "rolled" dices
		let s_counter = results.iter().filter(|x| **x == self.s).count();
		let counter: u8 = results.into_iter().filter(|x| *x != self.s).sum();
		dbgprintln!("Folgende Level haben crits:");
		levels.iter().for_each(|x| {
			if x.1 {
				dbgprintln!("Level: {}", x.0);
			}
		});
		dbgprintln!("S: {}", s_counter);
		dbgprintln!("Blitze: {}", counter);
	}
}
