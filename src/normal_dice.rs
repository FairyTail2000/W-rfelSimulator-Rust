use random_integer::random_u8;
/**
Contains the information of one result
*/
struct Results {
	data: Vec<u8>,
	sides: u8,
	count: u64
}

trait PrintResult {
	fn print_results(&self, old_style: bool);
}

impl PrintResult for Results {
	fn print_results(&self, old_style: bool) {
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
			}

			for (index, datapoint) in accumulated.iter().enumerate() {
				println!("{}: {}", index + 1, datapoint);
			}
		} else {
			for result in &self.data {

			}
		}

		unimplemented!()
	}
}

fn roll(amount: u64, sides: u8) -> Results {
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