use ansi_term::Colour;
use crate::common::{settings_path, Loadable};
use crate::dbgprintln;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use crate::decay_series::state::State;

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

impl Loadable<Vec<Self>> for Operation {
    fn load(file: Option<&str>) -> Vec<Self> {
        let alt = settings_path("decay_series.yaml");
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

impl Operation {
    fn defaults() -> Vec<Self> {
        include!("default_operation.rs")
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
        new_state
    }
}