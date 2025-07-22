use std::error::Error;
use ansi_term::Colour;
use common::{settings_path, Loadable};
use common::macros::dbgprintln;
use random_integer::random_u8;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::vec::Vec;
use serde::{Deserialize, Serialize};

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
			let sum: u64 = self.data.iter().map(|x| *x as u64).sum();

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
	Results {
		data: (0..amount).map(|_| random_u8(1, sides)).collect(),
		sides,
		count: amount as u64,
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dices {
	pub dices: Vec<u8>,
}

impl Default for Dices {
	fn default() -> Self {
		Dices {
			dices: vec![2, 3, 4, 6, 8, 10, 20, 100],
		}
	}
}

impl Loadable<Self> for Dices {
	fn load(file: Option<&str>) -> Self {
		let default_path = settings_path(NORMAL_DICES_FILE);
		let file_name = file
			.map_or(default_path, |s| Path::new(s).to_path_buf())
			.to_string_lossy()
			.to_string();
		let path = Path::new(&file_name);
		Dices::load_from_path(path)
			.unwrap_or_else(|e| {
				dbgprintln!("{}", Colour::RGB(255, 0, 0).paint(format!("Error loading dices: {}. Attempting to create default.", e)));
				let default_dices = Dices::default();
				let save_result = default_dices.save_to_path(path);
				match save_result {
					Ok(_) => dbgprintln!("New default dices created at: {}", path.display()),
					Err(err) => dbgprintln!("{}", Colour::RGB(255, 0, 0).paint(format!("Error creating or saving default dices: {}", err)))
				}
				default_dices
			})
	}
}

impl Dices {
	pub fn len(&self) -> usize {
		self.dices.len()
	}

	fn load_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
		if !path.exists() {
			return Err("File does not exist".into());
		}

		let file = File::open(path)?;
		let buf_reader = BufReader::new(file);
		let dices = serde_yaml::from_reader(buf_reader)?;
		Ok(dices)
	}

	fn save_to_path(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		let file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)?;
		let writer = BufWriter::new(file);
		serde_yaml::to_writer(writer, self)?;
		Ok(())
	}
}
