use crate::common::{settings_path, Loadable, Rollable};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use rand::distr::Uniform;
use rand::Rng;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Hash, Debug)]
pub struct Spells {
	pub spells: Vec<String>,
	pub name: String,
}

impl Spells {
	pub fn defaults() -> Vec<Spells> {
		vec![
			Spells {
				name: "Projektionszauber".to_string(),
				spells: vec![
					"Astrale Ohren".to_string(),
					"Astrale Projektion".to_string(),
					"Astrales Rauschen".to_string(),
					"Erleuchten".to_string(),
					"Erschaffen".to_string(),
					"Extraktor".to_string(),
					"Hologramm".to_string(),
					"Kunst der Natur".to_string(),
					"Leuchte".to_string(),
					"Molekularsicht".to_string(),
					"Trugbild".to_string(),
					"Visionär".to_string(),
					"Wetterkontrolle".to_string(),
					"Zeichnen".to_string(),
				],
			},
			Spells {
				name: "Heil- und Schutzzauber".to_string(),
				spells: vec![
					"Attributsschub".to_string(),
					"Desinfektor".to_string(),
					"Elementarschild".to_string(),
					"Flächenwirkung".to_string(),
					"Göttlicher Segen".to_string(),
					"Heilkreis".to_string(),
					"Heilung".to_string(),
					"Manablockade".to_string(),
					"Projektilschild".to_string(),
					"Reparator".to_string(),
				],
			},
			Spells {
				name: "Geistermagie".to_string(),
				spells: vec![
					"Binden".to_string(),
					"Heilige Erlösung".to_string(),
					"Segnen".to_string(),
					"Späher".to_string(),
					"Verbannen".to_string(),
				],
			},
			Spells {
				name: "Manipulationszauber".to_string(),
				spells: vec![
					"Brecher".to_string(),
					"Feger".to_string(),
					"Freie Zunge".to_string(),
					"Humanzauber".to_string(),
					"Mobbewusstsein".to_string(),
					"Molekularbearbeitung".to_string(),
					"Morpher".to_string(),
					"Offenes Buch".to_string(),
					"Reinigen".to_string(),
					"Schrumpfen".to_string(),
					"Statusbrecher".to_string(),
					"Vergrößern".to_string(),
					"Zeitkontrolle".to_string(),
				],
			},
			Spells {
				name: "Kampfzauber".to_string(),
				spells: vec![
					"Dampfgarer".to_string(),
					"Elementar-Emitter".to_string(),
					"Energieball".to_string(),
					"Feuerball".to_string(),
					"Flamme".to_string(),
					"Höllenschrei".to_string(),
					"Inferno".to_string(),
					"Komprimator".to_string(),
					"Manablitz".to_string(),
					"Parasit".to_string(),
					"Sandsturm".to_string(),
					"Schnee".to_string(),
					"Todesfluch".to_string(),
					"Sprengball".to_string(),
					"Toxin".to_string(),
					"Trance".to_string(),
					"Windhose".to_string(),
				],
			},
		]
	}
}

impl Loadable<Vec<Spells>> for Spells {
	fn load(file: Option<&str>) -> Vec<Spells> {
		let alt = settings_path("spell.yaml");
		let file_name = file.unwrap_or(alt.to_str().unwrap());
		if Path::new(file_name).exists() {
			let file = File::open(file_name).unwrap();
			let buf_reader = BufReader::new(file);
			match serde_yaml::from_reader::<BufReader<File>, Vec<Spells>>(buf_reader) {
				Ok(spells) => spells,
				Err(err) => {
					eprintln!("{}", err);
					let file = OpenOptions::new()
						.write(true)
						.truncate(true)
						.open(file_name)
						.unwrap();
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer(writer, &Spells::defaults()) {
						Ok(_) => {}
						Err(err) => {
							eprintln!("Couldn't default values to file");
							eprintln!("{}", err);
						}
					}
					Spells::defaults()
				}
			}
		} else {
			Spells::defaults()
		}
	}
}

impl Rollable<String> for Spells  {
    fn roll(&self, rng: &mut impl Rng) -> String {
		let distribution = Uniform::new_inclusive(0, self.spells.len() - 1).expect("Failed to create distribution for Spells");
        self.spells[rng.sample(distribution)].clone()
    }
}