use crate::State;
use ansi_term::Colour;
use common::settings_path;
use macros::dbgprintln;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default, Serialize, Deserialize)]
pub struct Operation {
    pub display: String,
    pub electrons: Option<i64>,
    pub protons: Option<i64>,
    pub neutrons: Option<i64>,
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

impl Operation {
    fn defaults() -> Vec<Self> {
        return vec![
            Operation {
                display: "Alpha".parse().unwrap(),
                electrons: None,
                protons: Some(-2),
                neutrons: Some(-2),
            },
            Operation {
                display: "Beta hin".parse().unwrap(),
                electrons: Some(1),
                protons: Some(1),
                neutrons: Some(-1),
            },
            Operation {
                display: "Beta rück".parse().unwrap(),
                electrons: Some(-1),
                protons: Some(-1),
                neutrons: Some(1),
            },
            Operation {
                display: "Beta hin minus".parse().unwrap(),
                electrons: Some(1),
                protons: Some(-1),
                neutrons: Some(1),
            },
            Operation {
                display: "Beta rück plus".parse().unwrap(),
                electrons: Some(-1),
                protons: Some(1),
                neutrons: Some(-1),
            },
            Operation {
                display: "Gamma".parse().unwrap(),
                electrons: Some(-1),
                protons: None,
                neutrons: None,
            },
            Operation {
                display: "Delta".parse().unwrap(),
                electrons: Some(2),
                protons: Some(-1),
                neutrons: None,
            },
        ];
    }

    pub fn apply(&self, state: State) -> State {
        let mut new_state = state.clone();
        match self.electrons {
            None => {}
            Some(val) => {
                new_state.electrons += val;
            }
        }
        match self.protons {
            None => {}
            Some(val) => {
                new_state.protons += val;
            }
        }
        match self.neutrons {
            None => {}
            Some(val) => {
                new_state.neutrons += val;
            }
        }
        new_state.validate()
    }

    pub fn load(file: Option<&str>) -> Vec<Self> {
        let alt = settings_path("zerfallsreihe.yaml");
        let file_name = file.unwrap_or(alt.to_str().unwrap());

        if Path::new(file_name).exists() {
            let file = File::open(file_name).unwrap();
            let buf_reader = BufReader::new(file);
            serde_yaml::from_reader::<BufReader<File>, Vec<Operation>>(buf_reader)
                .unwrap_or(Operation::defaults())
        } else {
            match File::create(file_name) {
                Ok(file) => {
                    let writer = BufWriter::new(file);
                    match serde_yaml::to_writer::<BufWriter<File>, Vec<Operation>>(
                        writer,
                        &Operation::defaults(),
                    ) {
                        Ok(_) => {
                            dbgprintln!("Neue Zerfallsreihen wurden erzeugt");
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
            Operation::defaults()
        }
    }
}
