use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::State;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default, Serialize, Deserialize)]
pub struct Op {
    pub electrons: Option<i64>,
    pub protons: Option<i64>,
    pub neutrons: Option<i64>
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default, Serialize, Deserialize)]
pub struct Operation {
    pub display: String,
    pub operation: Op
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

impl Operation {
    pub fn apply(&self, state: State) -> State {
        let mut new_state = state.clone();
        match self.operation.electrons {
            None => {}
            Some(val) => {
                new_state.electrons += val;
            }
        }
        match self.operation.protons {
            None => {}
            Some(val) => {
                new_state.protons += val;
            }
        }
        match self.operation.neutrons {
            None => {}
            Some(val) => {
                new_state.neutrons += val;
            }
        }
        new_state.validate()
    }
}