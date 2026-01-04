use std::error::Error;
use ansi_term::Colour;
use crate::common::{settings_path, Loadable};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::vec::Vec;
use rand::Rng;
use rand::distr::Uniform;
use serde::{Deserialize, Serialize};
use crate::dbgprintln;

const NORMAL_DICES_FILE: &str = "normal.yaml";

/**
Contains the information of one result
 */
pub struct Results {
	counts: Option<Vec<u64>>,
	data: Option<Vec<u8>>,
	sides: u8,
	count: u64,
}

fn calculate_variance(counts: &Vec<u64>, total_samples: u32, sides: u32) -> f64 {
	let mean = total_samples as f64 / sides as f64;
	let variance: f64 = counts.iter()
		.map(|&count| {
			let diff = count as f64 - mean;
			diff * diff
		})
		.sum::<f64>() / sides as f64;
	variance
}

impl Results {
	pub fn print_results(&self, old_style: bool, no_summary: bool) {
		println!("\n");
		if let Some(counts) = &self.counts {
			if self.sides == 6 {
				dbgprintln!("Misserfolge: {}", counts[0]); // 1
				dbgprintln!(
				"Misserfolge (improvisert): {}",
				counts[0] + counts[1]
			); // 1 + 2
				dbgprintln!(
				"Misserfolge (Pechphiole): {}",
				counts[0] + counts[1] + counts[2]
			); // 1 + 2 + 3
				dbgprintln!(
				"Erfolge (Wealthphiole): {}",
				counts[2] + counts[3] + counts[4] + counts[5]
			); // 3 + 4 + 5 + 6
				dbgprintln!(
				"Erfolge (Glücksphiole): {}",
				counts[3] + counts[4] + counts[5]
			); // 4 + 5 + 6
				dbgprintln!("Erfolge: {}", counts[4] + counts[5]); // 5 + 6

			} else {
				let sum: u64 = counts.iter().enumerate()
					.map(|(index, &count)| (index + 1) as u64 * count)
					.sum();

				if old_style {
					for (index, result) in counts.iter().enumerate() {
						if *result != 0 {
							dbgprintln!("Augenzahl: {}\tErgebnis: {}", index + 1, result);
						}
					}
					println!();
				}
				dbgprintln!("Summe: {}", sum);
			}
		} else if let Some(data) = &self.data {
			if self.sides == 6 {
				let mut accumulated: [u64; 6] = [0, 0, 0, 0, 0, 0];
				for result in data {
					debug_assert!(*result <= 6);
					accumulated[(*result - 1) as usize] += 1;
				}

				if old_style {
					for (index, result) in data.iter().enumerate() {
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
				let sum: u64 = data.iter().map(|x| *x as u64).sum();

				if old_style {
					for (index, result) in data.iter().enumerate() {
						if *result != 0 {
							dbgprintln!("Augenzahl: {}\tErgebnis: {}", index + 1, result);
						}
					}
					println!();
				}
				dbgprintln!("Summe: {}", sum);
			}
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
		if cfg!(debug_assertions) {
			if let Some(counts) = &self.counts {
				dbgprintln!("Varianz: {}", calculate_variance(counts, counts.iter().sum::<u64>() as u32, self.sides as u32));
			} else if let Some(data) = &self.data {
				let mut counts = vec![0u64; self.sides as usize];
				for &value in data {
					if value > 0 && value <= self.sides {
						counts[(value - 1) as usize] += 1;
					}
				}
				dbgprintln!("Varianz: {}", calculate_variance(&counts, data.len() as u32, self.sides as u32));
			}
		}

	}
}

pub fn roll(amount: usize, sides: u8, old_style: bool, rng: &mut impl Rng) -> Results {
	if old_style {
		let distribution = Uniform::<u8>::new_inclusive(1, sides).expect("Failed to create uniform distribution");
		Results {
			counts: None,
			data: Some((0..amount).map(|_| rng.sample(&distribution)).collect()),
			sides,
			count: amount as u64,
		}
	} else {
		// Pre-allocate space for each side (1-indexed for convenience)
		let mut counts = vec![0u64; (sides + 1) as usize];
		let die_range = Uniform::new_inclusive(1, sides as usize).expect("Failed to create uniform distribution for dice rolls");
		for _ in 0..amount {
			let result = rng.sample(die_range);
			// We use unchecked access because the distribution guarantees 1..=sides
			unsafe {
				*counts.get_unchecked_mut(result) += 1;
			}
		}

		Results {
			counts: Some(counts),
			data: None,
			sides,
			count: amount as u64,
		}
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
	fn load_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
		if !path.exists() {
			return Err("File does not exist".into());
		}

		let file = File::open(path)?;
		let buf_reader = BufReader::new(file);
		let dices = serde_yaml::from_reader(buf_reader)?;
		Ok(dices)
	}

	fn save_to_path(&self, path: &Path) -> Result<(), Box<dyn Error>> {
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
