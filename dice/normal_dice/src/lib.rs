use ansi_term::Colour;
use common::{settings_path, Loadable, random, ToByteArray};
use common::macros::dbgprintln;
use random_integer::random_u8;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::vec::Vec;

const NORMAL_DICES_FILE: &str = "normal.yaml";

/**
Contains the information of one result
 */
pub struct Results {
	data: Vec<u8>,
	sides: u8,
	count: u64,
}

impl Results {
	pub fn print_results(&self, old_style: bool, no_summary: bool) {
		println!("\n");
		if self.sides == 6 {
			let mut accumulated: [u64; 6] = [0, 0, 0, 0, 0, 0];
			for result in &self.data {
				debug_assert!(*result <= 6);
				accumulated[(*result - 1) as usize] += 1;
			}

			if old_style {
				for (index, result) in self.data.iter().enumerate() {
					dbgprintln!("{}: {}", index + 1, result);
				}
				dbgprintln!("\n");
			}

			for (index, datapoint) in accumulated.iter().enumerate() {
				dbgprintln!("{}: {}", index + 1, datapoint);
			}

			dbgprintln!("Misserfolge: {}", accumulated[0]); // 1
			dbgprintln!(
				"Misserfolge (improvisert): {}",
				accumulated[0] + accumulated[1]
			); // 1 + 2
			dbgprintln!(
				"Misserfolge (Pechphiole): {}",
				accumulated[0] + accumulated[1] + accumulated[2]
			); // 1 + 2 + 3
			dbgprintln!(
				"Erfolge (Wealthphiole): {}",
				accumulated[2] + accumulated[3] + accumulated[4] + accumulated[5]
			); // 3 + 4 + 5 + 6
			dbgprintln!(
				"Erfolge (Glücksphiole): {}",
				accumulated[3] + accumulated[4] + accumulated[5]
			); // 4 + 5 + 6
			dbgprintln!("Erfolge: {}", accumulated[4] + accumulated[5]); // 5 + 6
		} else {
			let mut sum: u64 = 0;
			for number in &self.data {
				sum += *number as u64;
			}

			if old_style {
				for (index, result) in self.data.iter().enumerate() {
					if *result != 0 {
						dbgprintln!("Augenzahl: {}\tErgebnis: {}", index + 1, result);
					}
				}
				println!();
			}
			dbgprintln!("Summe: {}", sum);
		}

		if !no_summary {
			dbgprintln!(
				"Es wurde mit {} {} gewürfelt {} {} {} {}\n",
				self.count,
				if self.count == 1 {
					"Würfel"
				} else {
					"Würfeln"
				},
				if self.count == 1 { "welcher" } else { "welche" },
				self.sides,
				if self.sides == 1 { "Seite" } else { "Seiten" },
				if self.sides == 1 { "hatte" } else { "hatten" }
			);
		}
	}
}

pub fn roll(amount: usize, sides: u8) -> Results {
	let mut results = Results {
		data: Vec::with_capacity(amount),
		sides,
		count: amount as u64,
	};
	for _ in 0..amount {
		results.data.push(random_u8(1, sides))
	}
	results
}

pub fn roll_native(amount: usize, sides: u8) -> Option<Results> {
	let mut results = Results {
		data: Vec::with_capacity(amount),
		sides,
		count: amount as u64,
	};
	// 8 is the maximum amount of random numbers that can be generated at once
	// 1 more is added to make sure that the amount of random numbers is enough
	for _ in 0..(amount/8+1) {
		// Collect the random numbers until the amount is reached
		match random() {
			Some(value) => {
				results.data.extend(value.to_byte_array())
			},
			None => {
				return None;
			}
		};
		if results.data.len() >= amount {
			results.data.truncate(amount);
			break;
		}
	}
	// Map the random numbers to the dice sides
	results.data = results.data.iter().map(|x| x % sides + 1).collect();

	Some(results)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dices {
	pub dices: Vec<u8>,
}

impl Default for Dices {
	fn default() -> Self {
		return Dices {
			dices: vec![2, 3, 4, 6, 8, 10, 20, 100],
		};
	}
}

impl Loadable<Self> for Dices {
	fn load(file: Option<&str>) -> Self {
		let alt = settings_path(NORMAL_DICES_FILE);
		let file_name = file.unwrap_or(alt.to_str().unwrap());

		if Path::new(file_name).exists() {
			let file = File::open(file_name).unwrap();
			let buf_reader = BufReader::new(file);
			serde_yaml::from_reader::<BufReader<File>, Dices>(buf_reader)
				.unwrap_or(Dices::default())
		} else {
			match File::create(file_name) {
				Ok(file) => {
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer::<BufWriter<File>, Dices>(writer, &Dices::default())
					{
						Ok(_) => {
							dbgprintln!("Neue Würfel wurden erzeugt");
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
			Dices::default()
		}
	}
}

impl Dices {
	pub fn len(&self) -> usize {
		return self.dices.len();
	}
}

#[cfg(test)]
mod tests {
	use super::{roll, roll_native};

	#[test]
	fn test_roll() {
		let results = roll(100, 6);
		assert_eq!(results.data.len(), 100);
		for result in results.data {
			assert!(result >= 1 && result <= 6);
		}
	}

	#[test]
	fn test_roll_native() {
		let results = roll_native(100, 6);
		assert!(results.is_some(), "Failed to generate random numbers");
		let results = results.unwrap();
		assert_eq!(results.data.len(), 100);
		for result in results.data {
			assert!(result >= 1 && result <= 6);
		}
	}
}
