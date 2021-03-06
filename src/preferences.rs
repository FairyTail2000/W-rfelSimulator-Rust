use ansi_term::Colour;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const PREFERENCE_FILE: &str = "settings.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    pub(crate) old_style: bool,
    pub(crate) no_tutorial: bool,
    pub(crate) no_summary_message: bool,
    pub(crate) no_select_dice_select: bool,
    pub(crate) number_instead: bool,
}

impl Settings {
    pub fn load() -> Self {
        dbgprintln!("Loading Settings from {}", PREFERENCE_FILE);
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
            match File::create(PREFERENCE_FILE) {
                Ok(file) => {
                    let writer = BufWriter::new(file);
                    match serde_yaml::to_writer::<BufWriter<File>, Settings>(
                        writer,
                        &Settings::default(),
                    ) {
                        Ok(_) => {
                            dbgprintln!("Neue Einstellungen wurden erzeugt");
                            #[cfg(not(debug_assertions))]
                            println!("Neue Einstellungen wurden erzeugt");
                        }
                        Err(err) => {
                            dbgprintln!("{}", Colour::RGB(255, 0, 0).paint(err.to_string()));
                            #[cfg(not(debug_assertions))]
                            println!("{}", Colour::RGB(255, 0, 0).paint(err.to_string()))
                        }
                    }
                }
                Err(err) => {
                    dbgprintln!("{}", Colour::RGB(255, 0, 0).paint(err.to_string()));
                    #[cfg(not(debug_assertions))]
                    println!("{}", Colour::RGB(255, 0, 0).paint(err.to_string()))
                }
            }

            Settings::default()
        };
    }
}
