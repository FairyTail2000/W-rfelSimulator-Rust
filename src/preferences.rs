use ansi_term::Colour;
use common::{settings_path, Loadable};
use macros::dbgprintln;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const PREFERENCE_FILE: &str = "settings.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
	pub(crate) old_style: bool,
	pub(crate) no_tutorial: bool,
	pub(crate) no_summary_message: bool,
	pub(crate) no_select_dice_select: bool,
	pub(crate) number_instead: bool,
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			old_style: true,
			no_tutorial: false,
			no_summary_message: false,
			no_select_dice_select: false,
			number_instead: true,
		}
	}
}

impl Loadable<Self> for Settings {
	fn load(file: Option<&str>) -> Self {
		let alt = settings_path(PREFERENCE_FILE);
		let file_name = file.unwrap_or(alt.to_str().unwrap());
		#[cfg(debug_assertions)]
		dbgprintln!("Loading Settings from {}", file_name);
		let exists = Path::new(file_name).exists();
		if exists {
			let file = File::open(file_name).unwrap();
			let buf_reader = BufReader::new(file);
			let parsed = serde_yaml::from_reader::<BufReader<File>, Settings>(buf_reader);
			parsed.unwrap_or(Settings::default())
		} else {
			match File::create(file_name) {
				Ok(file) => {
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer::<BufWriter<File>, Settings>(
						writer,
						&Settings::default(),
					) {
						Ok(_) => {
							dbgprintln!("Neue Einstellungen wurden erzeugt");
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

			Settings::default()
		}
	}
}
