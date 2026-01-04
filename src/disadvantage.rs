use crate::common::{settings_path, Loadable};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use rand::distr::Uniform;
use rand::Rng;

#[derive(Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, Default, Hash, Debug, Clone)]
pub struct Disadvantage {
	name: String,
	description: String,
}

impl Loadable<Vec<Disadvantage>> for Disadvantage {
	fn load(file: Option<&str>) -> Vec<Disadvantage> {
		let alt = settings_path("disadvantage.yaml");
		let file_name = file.unwrap_or(alt.to_str().unwrap());
		if Path::new(file_name).exists() {
			let file = File::open(file_name).unwrap();
			let buf_reader = BufReader::new(file);
			match serde_yaml::from_reader::<BufReader<File>, Vec<Disadvantage>>(buf_reader) {
				Ok(disadvantages) => disadvantages,
				Err(err) => {
					eprintln!("{}", err);
					let file = OpenOptions::new()
						.write(true)
						.truncate(true)
						.open(file_name)
						.unwrap();
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer(writer, &Disadvantage::defaults()) {
						Ok(_) => {}
						Err(err) => {
							eprintln!("Couldn't write default values to file!");
							eprintln!("{}", err);
						}
					}
					Disadvantage::defaults()
				}
			}
		} else {
			match File::create(file_name) {
				Ok(file) => {
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer::<BufWriter<File>, Vec<Disadvantage>>(
						writer,
						&Disadvantage::defaults(),
					) {
						Ok(_) => {
							println!("Neue Nachteile wurden erzeugt");
						}
						Err(err) => {
							eprintln!("{}", err)
						}
					}
				}
				Err(err) => {
					eprintln!("{}", err);
				}
			}
			Disadvantage::defaults()
		}
	}
}

impl Disadvantage {
	pub fn defaults() -> Vec<Disadvantage> {
		return include!("default_disadvantage.rs");
	}
}

impl Display for Disadvantage {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}\n{}", self.name, self.description)
	}
}

pub fn get_random<T: Clone>(adv: &Vec<T>, rng: &mut impl Rng) -> T {
	let uniform = Uniform::new_inclusive(0, adv.len() - 1).expect("Failed to create uniform distribution for advantages");
	adv[rng.sample(uniform)].clone()
}
