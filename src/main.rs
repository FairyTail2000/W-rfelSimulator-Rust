mod normal_dice;
mod colored_dice;

use std::io;
use std::io::Write;
use std::process::exit;
use clap::{Arg, App};
use crate::normal_dice::PrintResult;

const ALLOWED_DICE_SITES: [u8; 8] = [2, 3, 4, 6, 8, 10, 20, 100];
const ALLOWED_COLOURED_DICES: [(&str, &str); 4] = [("Rosa", "r"), ("Weiß", "w"), ("Grün", "g"), ("Schwarz", "s")];

/**
* Prints basic information's about the usage of the program
* Also used for help message
*/
fn print_startup_informations(allowed_coloured_dices: [(&str, &str); 4], allowed_dice_sites: [u8; 8]) {
	print!("Erlaubte Würfelseiten:");
	// Faster writing to the terminal because values are not waiting to be written to std::io::stdout
	let mut vector: Vec<String> = vec![];
	for ( index, site) in allowed_dice_sites.iter().enumerate() {
		vector.push((*format!(" {}", site)).parse().unwrap());
		if allowed_dice_sites.len() != index + 1 {
			vector.push((*format!(",")).parse().unwrap());
		}
	}
	println!("{}", vector.join(""));
	vector.clear();
	print!("Erlaubte farbige Seiten:");
	for (index, site) in allowed_coloured_dices.iter().enumerate() {
		let (long, short) = site;
		vector.push((*format!(" {} ({})", long, short)).parse().unwrap());
		if allowed_coloured_dices.len() != index + 1 {
			vector.push((*format!(",")).parse().unwrap());
		}
	}
	println!("{}", vector.join(""));
}

fn handle_input(input: String, old_report_style: bool) -> bool {
	return if input == "exit" || input == "e" {
		true
	} else if input == "help" || input == "h" {
		print_startup_informations(ALLOWED_COLOURED_DICES, ALLOWED_DICE_SITES);
		false
	} else {
		let parsed = input.parse::<u8>();
		if let Err(_err) = parsed {
			println!("Es muss eine Ganzzahl sein, wie oben beschrieben");
			false
		} else if let Ok(sides) = parsed {
			if ALLOWED_DICE_SITES.contains(&sides) {
				let amount = ask_for_amount(std::u64::MAX);

				let res = normal_dice::roll(amount, sides);
				res.print_results(old_report_style);
			} else {
				println!("Die ist nicht erlaubt...");
			}
			false
		} else {
			false
		}
	}
}

fn ask_for_amount(max: u64) -> u64 {
	println!("Max: {}", max);

	loop {
		print!("Anzahl: ");

		let mut input = String::new();
		let res = io::stdout().flush();
		if let Err(_e) = res {
			exit(-1)
		}

		match io::stdin().read_line(&mut input) {
			Ok(_n) => {
				let removed = input.replace("\n", "");
				let parsed = removed.parse::<u64>();
				let mut number = 0;
				if let Err(_err) = parsed {
					println!("Es muss eine Ganzzahl sein, maximal {}", max);
					continue;
				} else if let Ok(dice_count) = parsed {
					number = dice_count;
				}
				break number;
			}
			Err(error) => println!("error while reading from stdin: {}", error),
		};
	}
}

fn main() {
	let matches = App::new("Würfeln")
		.version("1.0")
		.author("Rafael Sundorf <developer.rafael.sundorf@gmail.com>")
		.about("Hiermit kann man würfeln!")
		.arg(Arg::new("old_style")
			.short('o')
			.long("old_style")
			.about("Nutzt den alten style um das Ergebnis anzuzeigen")
		)
		.arg(Arg::new("no tutorial")
			.short('n')
			.long("no-tutorial")
			.about("Gives you no tutorial messages")
		)
		.arg(Arg::new("no summary message")
			.short('s')
			.long("no-summary-message")
			.about("Disables ")
		)
		.get_matches();

	let old = matches.is_present("old_style");


	let mut finished = false;
	if !matches.is_present("no tutorial") {
		print_startup_informations(ALLOWED_COLOURED_DICES, ALLOWED_DICE_SITES);
	}

	while !finished {
		print!("Seitenanzahl: ");
		let res = io::stdout().flush();
		if let Err(_e) = res {
			exit(-1)
		}

		let mut input = String::new();
		match io::stdin().read_line(&mut input) {
			Ok(_n) => {
				let removed = input.replace("\n", "");
				// Return value determines continuation of the loop, true ends the loop, false continues it
				finished = handle_input(removed, old);
			}
			Err(error) => println!("error: {}", error),
		}
	}
}
