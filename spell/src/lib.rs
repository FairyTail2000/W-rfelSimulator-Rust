use common::{settings_path, Loadable, Rollable};
use random_integer::random_usize;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Hash, Debug)]
pub struct Spells {
	pub spells: Vec<String>,
	pub name: String,
}

impl Spells {
	pub fn defaults() -> Vec<Spells> {
		return vec![
			Spells {
				name: "Projektionszauber".parse().unwrap(),
				spells: vec![
					"Astrale Ohren".parse().unwrap(),
					"Astrale Projektion".parse().unwrap(),
					"Astrales Rauschen".parse().unwrap(),
					"Erleuchten".parse().unwrap(),
					"Erschaffen".parse().unwrap(),
					"Extraktor".parse().unwrap(),
					"Hologramm".parse().unwrap(),
					"Kunst der Natur".parse().unwrap(),
					"Leuchte".parse().unwrap(),
					"Molekularsicht".parse().unwrap(),
					"Trugbild".parse().unwrap(),
					"Visionär".parse().unwrap(),
					"Wetterkontrolle".parse().unwrap(),
					"Zeichnen".parse().unwrap(),
				],
			},
			Spells {
				name: "Heil- und Schutzzauber".parse().unwrap(),
				spells: vec![
					"Attributsschub".parse().unwrap(),
					"Desinfektor".parse().unwrap(),
					"Elementarschild".parse().unwrap(),
					"Flächenwirkung".parse().unwrap(),
					"Göttlicher Segen".parse().unwrap(),
					"Heilkreis".parse().unwrap(),
					"Heilung".parse().unwrap(),
					"Manablockade".parse().unwrap(),
					"Projektilschild".parse().unwrap(),
					"Reparator".parse().unwrap(),
				],
			},
			Spells {
				name: "Geistermagie".parse().unwrap(),
				spells: vec![
					"Binden".parse().unwrap(),
					"Heilige Erlösung".parse().unwrap(),
					"Segnen".parse().unwrap(),
					"Späher".parse().unwrap(),
					"Verbannen".parse().unwrap(),
				],
			},
			Spells {
				name: "Manipulationszauber".parse().unwrap(),
				spells: vec![
					"Brecher".parse().unwrap(),
					"Feger".parse().unwrap(),
					"Freie Zunge".parse().unwrap(),
					"Humanzauber".parse().unwrap(),
					"Mobbewusstsein".parse().unwrap(),
					"Molekularbearbeitung".parse().unwrap(),
					"Morpher".parse().unwrap(),
					"Offenes Buch".parse().unwrap(),
					"Reinigen".parse().unwrap(),
					"Schrumpfen".parse().unwrap(),
					"Statusbrecher".parse().unwrap(),
					"Vergrößern".parse().unwrap(),
					"Zeitkontrolle".parse().unwrap(),
				],
			},
			Spells {
				name: "Kampfzauber".parse().unwrap(),
				spells: vec![
					"Dampfgarer".parse().unwrap(),
					"Elementar-Emitter".parse().unwrap(),
					"Energieball".parse().unwrap(),
					"Feuerball".parse().unwrap(),
					"Flamme".parse().unwrap(),
					"Höllenschrei".parse().unwrap(),
					"Inferno".parse().unwrap(),
					"Komprimator".parse().unwrap(),
					"Manablitz".parse().unwrap(),
					"Parasit".parse().unwrap(),
					"Sandsturm".parse().unwrap(),
					"Schnee".parse().unwrap(),
					"Todesfluch".parse().unwrap(),
					"Sprengball".parse().unwrap(),
					"Toxin".parse().unwrap(),
					"Trance".parse().unwrap(),
					"Windhose".parse().unwrap(),
				],
			},
		];
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
    fn roll(&self) -> &String {
        &self.spells[random_usize(0, self.spells.len() - 1)]
    }
}