use random_integer::random_u8;
use std::vec::Vec;
/**
Contains the information of one result
*/
pub struct Results {
	data: Vec<u8>,
	sides: u8,
	count: u64
}

pub trait PrintResult {
	fn print_results(&self, old_style: bool);
}

impl PrintResult for Results {
	fn print_results(&self, old_style: bool) {
		println!("\n");
		if self.sides == 6 {
			let mut accumulated: [u64; 6] = [0, 0, 0, 0, 0, 0 ];
			for result in &self.data {
				debug_assert!(*result <= 6);
				accumulated[(*result - 1) as usize] += 1;
			}

			if old_style {
				for (index, result) in self.data.iter().enumerate() {
					println!("{}: {}", index + 1, result);
				}
				println!("\n")
			}

			for (index, datapoint) in accumulated.iter().enumerate() {
				println!("{}: {}", index + 1, datapoint);
			}

			println!("Misserfolge: {}", accumulated[0]); // 1
			println!("Misserfolge (improvisert): {}", accumulated[0] + accumulated[1]); // 1 + 2
			println!("Misserfolge (Pechphiole): {}", accumulated[0] + accumulated[1] + accumulated[2]); // 1 + 2 + 3
			println!("Erfolge (Wealthphiole): {}", accumulated[2] + accumulated[3] + accumulated[4] + accumulated[5]); // 3 + 4 + 5 + 6
			println!("Erfolge (Glücksphiole): {}", accumulated[3] + accumulated[4] + accumulated[5]); // 4 + 5 + 6
			println!("Erfolge: {}", accumulated[4] + accumulated[5]); // 5 + 6
		} else {
			let mut sum: u64 = 0;
			for number in &self.data {
				sum += *number as u64;
			}

			if old_style {
				for (index, result) in self.data.iter().enumerate() {
					if result != 0 {
						println!("Augenzahl: {}\tErgebnis: {}", index + 1, result)
					}
				}
				println!();
			}
			println!("Summe: {}", sum);

		}
		println!("Es wurde mit {} {} gewürfelt {} {} {} {}\n", self.count, if self.count == 1 {"Würfel"} else {"Würfeln"}, if self.count == 1 {"welcher"} else {"welche"}, self.sides, if self.sides == 1 {"Seite"} else {"Seiten"}, if self.sides == 1 {"hatte"} else {"hatten"});
	}
}

pub fn roll(amount: u64, sides: u8) -> Results {
	let mut results = Results {
		data: vec![],
		sides,
		count: amount
	};

	for _ in 0..amount {
		results.data.push(random_u8(1, sides))
	}

	results
}