use std::fmt::{Display, Formatter};

#[cfg(test)]
mod tests {
    use crate::State;

    #[test]
    fn test_validation() {
        let invalid = State {
            electrons: -1,
            protons: 10,
            neutrons: -10000,
        };
        let valid = invalid.validate();
        assert_eq!(valid.protons, 10);
        assert_eq!(valid.electrons, 0);
        assert_eq!(valid.neutrons, 0);
    }

    #[test]
    fn test_description() {
        let test_state = State {
            electrons: 0,
            protons: 1,
            neutrons: 0,
        };
        assert_eq!(test_state.get_description(), "Nicht abgedeckt");
    }

    #[test]
    fn test_alpha() {
        let state = State {
            electrons: 10,
            protons: 1,
            neutrons: 10,
        };
        let mutated = state.alpha();
        assert_eq!(mutated.protons, 0);
        assert_eq!(mutated.neutrons, 8);
        assert_eq!(mutated.electrons, 10);
        assert_eq!(mutated.get_description(), "Pflanze der 3. Generation Nr. 0");
    }

    #[test]
    fn test_beta_hin_zerfall() {
        let state = State {
            electrons: 0,
            protons: 0,
            neutrons: 1,
        };
        let mutated = state.beta_hin_zerfall();
        assert_eq!(mutated.protons, 1);
        assert_eq!(mutated.neutrons, 0);
        assert_eq!(mutated.electrons, 1);
        assert_eq!(mutated.get_description(), "Monster Nr. 1");
    }

    #[test]
    fn test_beta_rück_zerfall() {
        let state = State {
            electrons: 1,
            protons: 1,
            neutrons: 1,
        };
        let mutated = state.beta_rück_zerfall();
        assert_eq!(mutated.protons, 0);
        assert_eq!(mutated.neutrons, 2);
        assert_eq!(mutated.electrons, 0);
        assert_eq!(mutated.get_description(), "Pflanze der 2. Generation Nr. 0");
    }

    #[test]
    fn test_beta_hin_minus_zerfall() {
        let state = State {
            electrons: 1,
            protons: 1,
            neutrons: 1,
        };
        let mutated = state.beta_hin_minus_zerfall();
        assert_eq!(mutated.protons, 0);
        assert_eq!(mutated.neutrons, 2);
        assert_eq!(mutated.electrons, 2);
        assert_eq!(mutated.get_description(), "Pflanze der 3. Generation Nr. 0");
    }

    #[test]
    fn test_beta_rück_plus_zerfall() {
        let state = State {
            electrons: 1,
            protons: 1,
            neutrons: 1,
        };
        let mutated = state.beta_rück_plus_zerfall();
        assert_eq!(mutated.protons, 2);
        assert_eq!(mutated.neutrons, 0);
        assert_eq!(mutated.electrons, 0);
        assert_eq!(mutated.get_description(), "Nicht abgedeckt");
    }

    #[test]
    fn test_gamma() {
        let state = State {
            electrons: 1,
            protons: 1,
            neutrons: 1,
        };
        let mutated = state.gamma();
        assert_eq!(mutated.protons, 1);
        assert_eq!(mutated.neutrons, 1);
        assert_eq!(mutated.electrons, 0);
        assert_eq!(mutated.get_description(), "Pflanze der 1. Generation Nr. 1");
    }

    #[test]
    fn test_delta() {
        let state = State {
            electrons: 1,
            protons: 1,
            neutrons: 1,
        };
        let mutated = state.delta();
        assert_eq!(mutated.protons, 0);
        assert_eq!(mutated.neutrons, 1);
        assert_eq!(mutated.electrons, 3);
        assert_eq!(mutated.get_description(), "Pflanze der 3. Generation Nr. 0");
    }

    #[test]
    fn test_froms() {
        assert_eq!(
            State::from("10;-10;0"),
            State {
                electrons: 10,
                protons: 0,
                neutrons: 0
            }
        );
        assert_eq!(
            State::from((10, -10, 0)),
            State {
                electrons: 10,
                protons: 0,
                neutrons: 0
            }
        );
        assert_eq!(
            State::from((10i64, -10, 0)),
            State {
                electrons: 10,
                protons: 0,
                neutrons: 0
            }
        );

        assert_eq!(
            State::from((10i16, -10, 0)),
            State {
                electrons: 10,
                protons: 0,
                neutrons: 0
            }
        );

        assert_eq!(
            State::from((10i8, -10, 0)),
            State {
                electrons: 10,
                protons: 0,
                neutrons: 0
            }
        );
        assert_eq!(
            State::from((10u8, 10, 0)),
            State {
                electrons: 10,
                protons: 10,
                neutrons: 0
            }
        );
    }

    #[test]
    #[should_panic]
    fn panic_from() {
        State::from("22,4,3");
    }

    #[test]
    #[should_panic]
    fn panic_from2() {
        State::from("A;B;C");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
pub struct State {
    pub electrons: i64,
    pub protons: i64,
    pub neutrons: i64,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nElektronen: {}\nProtonen: {}\nNeutronen: {}",
            self.get_description(),
            self.electrons,
            self.protons,
            self.neutrons
        )
    }
}

impl From<(i64, i64, i64)> for State {
    fn from(parts: (i64, i64, i64)) -> Self {
        State {
            electrons: parts.0,
            protons: parts.1,
            neutrons: parts.2,
        }
        .validate()
    }
}

impl From<(i32, i32, i32)> for State {
    fn from(parts: (i32, i32, i32)) -> Self {
        State {
            electrons: parts.0 as i64,
            protons: parts.1 as i64,
            neutrons: parts.2 as i64,
        }
        .validate()
    }
}

impl From<(i16, i16, i16)> for State {
    fn from(parts: (i16, i16, i16)) -> Self {
        State {
            electrons: parts.0 as i64,
            protons: parts.1 as i64,
            neutrons: parts.2 as i64,
        }
        .validate()
    }
}

impl From<(i8, i8, i8)> for State {
    fn from(parts: (i8, i8, i8)) -> Self {
        State {
            electrons: parts.0 as i64,
            protons: parts.1 as i64,
            neutrons: parts.2 as i64,
        }
        .validate()
    }
}

impl From<(u32, u32, u32)> for State {
    fn from(parts: (u32, u32, u32)) -> Self {
        State {
            electrons: parts.0 as i64,
            protons: parts.1 as i64,
            neutrons: parts.2 as i64,
        }
        .validate()
    }
}

impl From<(u16, u16, u16)> for State {
    fn from(parts: (u16, u16, u16)) -> Self {
        State {
            electrons: parts.0 as i64,
            protons: parts.1 as i64,
            neutrons: parts.2 as i64,
        }
        .validate()
    }
}

impl From<(u8, u8, u8)> for State {
    fn from(parts: (u8, u8, u8)) -> Self {
        State {
            electrons: parts.0 as i64,
            protons: parts.1 as i64,
            neutrons: parts.2 as i64,
        }
        .validate()
    }
}

impl From<String> for State {
    fn from(string: String) -> Self {
        let parts: Vec<i64> = string
            .split(";")
            .into_iter()
            .map(|val| i64::from_str_radix(val, 10).unwrap())
            .collect();
        if parts.len() != 3 {
            panic!("Invalid number of parts: {}", parts.len());
        }
        State {
            electrons: parts[0],
            protons: parts[1],
            neutrons: parts[2],
        }
        .validate()
    }
}

impl From<&str> for State {
    fn from(string: &str) -> Self {
        let parts: Vec<i64> = string
            .split(";")
            .into_iter()
            .map(|val| i64::from_str_radix(val, 10).unwrap())
            .collect();
        if parts.len() != 3 {
            panic!("Invalid number of parts: {}", parts.len());
        }
        State {
            electrons: parts[0],
            protons: parts[1],
            neutrons: parts[2],
        }
        .validate()
    }
}

impl State {
    pub fn get_description(&self) -> String {
        if self.neutrons == 0 && self.electrons == self.protons {
            format!("Monster Nr. {}", self.protons)
        } else if self.neutrons != 0 {
            if self.protons == self.electrons {
                format!("Pflanze der 2. Generation Nr. {}", self.protons)
            } else if self.protons < self.electrons {
                format!("Pflanze der 3. Generation Nr. {}", self.protons)
            } else if self.electrons < self.protons {
                format!("Pflanze der 1. Generation Nr. {}", self.protons)
            } else {
                String::from("Nicht abgedeckt")
            }
        } else {
            String::from("Nicht abgedeckt")
        }
    }

    pub fn alpha(&self) -> Self {
        State {
            electrons: self.electrons,
            protons: self.protons - 2,
            neutrons: self.neutrons - 2,
        }
        .validate()
    }

    pub fn beta_hin_zerfall(&self) -> Self {
        State {
            electrons: self.electrons + 1,
            protons: self.protons + 1,
            neutrons: self.neutrons - 1,
        }
        .validate()
    }

    pub fn beta_rück_zerfall(&self) -> Self {
        State {
            electrons: self.electrons - 1,
            protons: self.protons - 1,
            neutrons: self.neutrons + 1,
        }
        .validate()
    }

    pub fn beta_hin_minus_zerfall(&self) -> Self {
        State {
            electrons: self.electrons + 1,
            protons: self.protons - 1,
            neutrons: self.neutrons + 1,
        }
        .validate()
    }

    pub fn beta_rück_plus_zerfall(&self) -> Self {
        State {
            electrons: self.electrons - 1,
            protons: self.protons + 1,
            neutrons: self.neutrons - 1,
        }
        .validate()
    }

    pub fn gamma(&self) -> Self {
        State {
            electrons: self.electrons - 1,
            protons: self.protons,
            neutrons: self.neutrons,
        }
        .validate()
    }

    pub fn delta(&self) -> Self {
        State {
            electrons: self.electrons + 2,
            protons: self.protons - 1,
            neutrons: self.neutrons,
        }
        .validate()
    }

    fn validate(&self) -> Self {
        State {
            electrons: if self.electrons >= 0 {
                self.electrons
            } else {
                0
            },
            protons: if self.protons >= 0 { self.protons } else { 0 },
            neutrons: if self.neutrons >= 0 { self.neutrons } else { 0 },
        }
    }
}
