use random_integer::random_u8;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::vec::Vec;

const NORMAL_DICES_FILE: &str = "normal.yaml";

/**
Contains the information of one result
*/
pub struct Results {
    data: Vec<u8>,
    sides: u8,
    count: u64,
}

pub trait PrintResult {
    fn print_results(&self, old_style: bool, no_summary: bool);
}

impl PrintResult for Results {
    fn print_results(&self, old_style: bool, no_summary: bool) {
        println!("\n");
        if self.sides == 6 {
            let mut accumulated: [u64; 6] = [0, 0, 0, 0, 0, 0];
            for result in &self.data {
                debug_assert!(*result <= 6);
                accumulated[(*result - 1) as usize] += 1;
            }

            if old_style {
                for (index, result) in self.data.iter().enumerate() {
                    dbgprintln!("{}: {}", index + 1, result);
                    #[cfg(not(debug_assertions))]
                    println!("{}: {}", index + 1, result);
                }
                dbgprintln!("\n");
                #[cfg(not(debug_assertions))]
                println!("\n")
            }

            for (index, datapoint) in accumulated.iter().enumerate() {
                dbgprintln!("{}: {}", index + 1, datapoint);
                #[cfg(not(debug_assertions))]
                println!("{}: {}", index + 1, datapoint);
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

            #[cfg(not(debug_assertions))]
            {
                println!("Misserfolge: {}", accumulated[0]); // 1
                println!(
                    "Misserfolge (improvisert): {}",
                    accumulated[0] + accumulated[1]
                ); // 1 + 2
                println!(
                    "Misserfolge (Pechphiole): {}",
                    accumulated[0] + accumulated[1] + accumulated[2]
                ); // 1 + 2 + 3
                println!(
                    "Erfolge (Wealthphiole): {}",
                    accumulated[2] + accumulated[3] + accumulated[4] + accumulated[5]
                ); // 3 + 4 + 5 + 6
                println!(
                    "Erfolge (Glücksphiole): {}",
                    accumulated[3] + accumulated[4] + accumulated[5]
                ); // 4 + 5 + 6
                println!("Erfolge: {}", accumulated[4] + accumulated[5]); // 5 + 6
            }
        } else {
            let mut sum: u64 = 0;
            for number in &self.data {
                sum += *number as u64;
            }

            if old_style {
                for (index, result) in self.data.iter().enumerate() {
                    if *result != 0 {
                        dbgprintln!("Augenzahl: {}\tErgebnis: {}", index + 1, result);
                        #[cfg(not(debug_assertions))]
                        println!("Augenzahl: {}\tErgebnis: {}", index + 1, result)
                    }
                }
                println!();
            }
            dbgprintln!("Summe: {}", sum);
            #[cfg(not(debug_assertions))]
            println!("Summe: {}", sum);
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
            #[cfg(not(debug_assertions))]
            println!(
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
    }
}

pub fn roll(amount: u64, sides: u8) -> Results {
    let mut results = Results {
        data: vec![],
        sides,
        count: amount,
    };

    for _ in 0..amount {
        results.data.push(random_u8(1, sides))
    }

    results
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Dice {
    pub sides: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Dices {
    pub dices: Vec<Dice>,
}

impl Dices {
    pub fn load() -> Self {
        let exists = Path::new(NORMAL_DICES_FILE).exists();
        return if exists {
            let file = File::open(NORMAL_DICES_FILE).unwrap();
            let buf_reader = BufReader::new(file);
            let parsed = serde_yaml::from_reader::<BufReader<File>, Dices>(buf_reader);
            if let Ok(result) = parsed {
                result
            } else {
                Dices::default()
            }
        } else {
            Dices::default()
        };
    }

    pub fn len(&self) -> usize {
        return self.dices.len();
    }
}
