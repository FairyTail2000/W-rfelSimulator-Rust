mod preferences;

use color::get_color;
use crate::preferences::Settings;
use ansi_term::Colour;
use clap::{Arg, Command};
use dice::colored_dice::{ColoredDice, ColoredDices};
use common::{Loadable, Rollable};
use dice::crit_dice::CritDices;
use dialoguer::console::Term;
use dialoguer::{Input, MultiSelect, Select};
use common::macros::{dbgprint, dbgprintln};
use nachteil::Disadvantage;
use dice::normal_dice::Dices;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::process::exit;
#[cfg(debug_assertions)]
use std::time::SystemTime;
use zauber::Spells;
use zerfallsreihen::operation::Operation;
use zerfallsreihen::State;

/**
 * Prints basic information's about the usage of the program
 * Also used for help message
 */
fn print_startup_information(allowed_coloured_dices: &ColoredDices, allowed_dice_sites: &Dices) {
	dbgprint!("Erlaubte Würfelseiten:");
	let mut vector: Vec<String> = vec![];
	for (index, site) in allowed_dice_sites.dices.iter().enumerate() {
		vector.push((*format!(" {}", site)).parse().unwrap());
		if allowed_dice_sites.len() != index + 1 {
			vector.push((*format!(",")).parse().unwrap());
		}
	}
	dbgprintln!("{}", vector.join(""));

	vector.clear();

	dbgprint!("Erlaubte farbige Seiten:");

	for (index, site) in allowed_coloured_dices.dices.iter().enumerate() {
		let color: Colour = match get_color(&site.color) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("{}", e.deref());
				exit(-1);
			}
		};

		vector.push(
			(*format!(" {} ({})", color.paint(&site.long), site.short))
				.parse()
				.unwrap(),
		);
		if allowed_coloured_dices.len() != index + 1 {
			vector.push((*format!(",")).parse().unwrap());
		}
	}

	dbgprintln!("{}", vector.join(""));

	let res = io::stdout().flush();
	if let Err(_e) = res {
		exit(-1)
	}
}

fn handle_input(
	input: String,
	old_report_style: bool,
	allowed_colored_dices: &ColoredDices,
	allowed_dice_sites: &Dices,
	error_message: &str,
	no_summary: bool,
) -> bool {
	if input == "exit" || input == "e" {
		true
	} else if input == "help" || input == "h" {
		print_startup_information(allowed_colored_dices, allowed_dice_sites);
		false
	} else {
		let parsed = input.parse::<u8>();
		if let Err(_err) = parsed {
			dbgprintln!("Es muss eine Ganzzahl sein, wie oben beschrieben");
			false
		} else if let Ok(sides) = parsed {
			if allowed_dice_sites.dices.contains(&sides) {
				let amount = ask_for_amount(error_message, "Anzahl");
				let res = dice::normal_dice::roll(amount, sides);
				res.print_results(old_report_style, no_summary);
			} else {
				dbgprintln!("Die ist nicht erlaubt...")
			}
			false
		} else {
			false
		}
	}
}

fn ask_for_amount(error_message: &str, prompt: &str) -> usize {
	let input = Input::new()
		.with_prompt(prompt)
		.validate_with(|input: &String| -> Result<(), &str> {
			if let Ok(_parsed) = input.parse::<u64>() {
				Ok(())
			} else {
				Err(error_message)
			}
		})
		.interact_text();
	return if let Ok(result) = input {
		match result.parse() {
			Ok(res) => res,
			Err(_e) => 0
		}
	} else {
		0
	};
}

fn validator(val: &String) -> Result<(), &'static str> {
	let new_val = val.trim();
	if new_val.is_empty() {
		return Err("Bitte etwas eingeben");
	}
	match i64::from_str_radix(new_val, 10) {
		Ok(_) => Ok(()),
		Err(_) => Err("Bitte eine positive oder negative Ganzzahl eingeben"),
	}
}

fn decay_series(stdout: &Term, operation: &Vec<Operation>) {
	let protons_input = Input::new()
		.with_prompt("Protonen")
		.validate_with(validator)
		.interact_text();
	let neutrons_input = Input::new()
		.with_prompt("Neutronen")
		.validate_with(validator)
		.interact_text();
	let electrons_input = Input::new()
		.with_prompt("Elektronen")
		.validate_with(validator)
		.interact_text();
	let _ = stdout.clear_last_lines(3);

	let protons = match protons_input {
		Ok(inp) => {
			match i64::from_str_radix(&*inp.trim(), 10) {
				Ok(val) => val,
				Err(_) => 0,
			}
		}
		Err(_err) => 0
	};

	let neutrons = match neutrons_input {
		Ok(inp) => {
			match i64::from_str_radix(&*inp.trim(), 10) {
				Ok(val) => val,
				Err(_) => 0,
			}
		}
		Err(_err) => 0
	};

	let electrons = match electrons_input {
		Ok(inp) => {
			match i64::from_str_radix(&*inp.trim(), 10) {
				Ok(val) => val,
				Err(_) => 0,
			}
		}
		Err(_err) => 0
	};

	let mut state = State::from((electrons, protons, neutrons));
	let mut options: Vec<String> = operation.iter().map(|x| x.display.clone()).collect();
	options.push(String::from("Aufhören"));
	#[cfg(debug_assertions)]
	dbgprintln!("{:?}", operation);
	loop {
		let selection = Select::new()
			.with_prompt("Operation")
			.items(&options)
			.default(0)
			.interact();
		match selection {
			Ok(i) => {
				if i >= operation.len() {
					break;
				}
				let new = operation[i].apply(state);
				if new.protons < 0 || new.electrons < 0 || new.neutrons < 0 {
					dbgprintln!("Nicht möglich!");
				} else {
					state = new;
					dbgprintln!("{}", state);
				}
			}
			Err(_) => break,
		}
	}
	let _ = stdout.clear_last_lines(1);
}

fn get_app() -> Command {
	Command::new("Würfeln")
		.version("1.0.0")
		.author("Rafael Sundorf <developer.rafael.sundorf@gmail.com>")
		.about("Hiermit kann man würfeln!")
		.arg(Arg::new("old_style")
			.short('o')
			.long("old_style")
			.help("Nutzt den alten style um das Ergebnis anzuzeigen")
			.action(clap::ArgAction::SetTrue)
		)
		.arg(Arg::new("no tutorial")
			.short('n')
			.long("no-tutorial")
			.help("Unterdrückt die Start Nachricht")
			.action(clap::ArgAction::SetTrue)
		)
		.arg(Arg::new("no summary message")
			.short('s')
			.long("no-summary-message")
			.help("Unterdrückt die kurze information nachdem das Würfelergebnis ausgegeben wurde")
			.action(clap::ArgAction::SetTrue)
		)
		.arg(Arg::new("no select dice select")
			.short('d')
			.long("no-select-dice-select")
			.help("Verwendet die standard Eingabe anstatt einer Auswahl")
			.action(clap::ArgAction::SetTrue)
		)
		.arg(Arg::new("number instead")
			.short('i')
			.long("number-instead")
			.help("Verwendet eine Zahlen eingabe anstatt einer Auswahl und Anzahl von farbigen würfeln")
			.action(clap::ArgAction::SetTrue)
		)
}

fn roll_colored_dice(
	colored_dice: &ColoredDices,
	error_message: &str,
	number_instead: bool,
	stderr: &Term,
) -> io::Result<()> {
	let mut possibilities: Vec<&str> = vec![];
	if number_instead {
		//Input a number and auto compute values
		let amount = ask_for_amount(error_message, "Farbiger Würfel Wert");
		//Tuple of value, amount and result
		let mut dices: Vec<(u64, String)> = Vec::with_capacity(amount);
		let mut remaining = amount;

		let mut copy: Vec<ColoredDice> = colored_dice.dices.to_vec();

		copy.sort_by(|a, b| a.value.cmp(&b.value).reverse());
		for dice in copy {
			let mut result: u64 = 0;
			for _ in 0..remaining / dice.value as usize {
				result += *dice.roll() as u64;
			}
			let value = (result, dice.long);
			dices.push(value);
			remaining %= dice.value as usize;
		}

		let mut accumulated_result: u64 = 0;
		for result in dices {
			dbgprintln!("{}: {}", result.1, result.0);
			accumulated_result += result.0;
		}

		dbgprintln!(
			"Insgesamt: {} ({})",
			accumulated_result,
			accumulated_result * 10
		);
	} else {
		// Use multiselect...
		for (_index, dice) in colored_dice.dices.iter().enumerate() {
			possibilities.push(&*dice.long);
		}



		let selection = {
			let sel = MultiSelect::new()
				.items(&possibilities)
				.with_prompt(
					"Wähle deine farbigen Würfel (Mit der Leertaste auswählen und Enter bestätigen)",
				)
				.interact_on(stderr);

			match sel {
				Ok(sel) => sel,
				Err(_err) => vec![]
			}
		};

		if selection.is_empty() {
			dbgprintln!("Nichts gewählt.");
			return Ok(());
		}


		let mut result: Vec<(&String, u64)> = Vec::with_capacity(selection.len());
		let mut accumulated_amount: u64 = 0;
		for select in selection {
			let sel = colored_dice.dices.get(select).unwrap();
			let amount = ask_for_amount(error_message, &*format!("Anzahl {}", sel.long));
			// Amount but shorter
			let mut am: u64 = 0;
			for _ in 0..amount {
				am += *(sel.roll()) as u64;
			}
			accumulated_amount += am;
			result.push((&sel.long, am))
		}

		for res in result {
			dbgprintln!("{}: {}", res.0, res.1);
			dbgprintln!(
				"Insgesamt: {} ({})",
				accumulated_amount,
				accumulated_amount * 10
			);
		}
	}
	Ok(())
}

fn main() -> io::Result<()> {
	let matches = get_app().get_matches();

	#[cfg(debug_assertions)]
	let start: SystemTime = SystemTime::now();
	#[cfg(debug_assertions)]
	dbgprintln!("Loading Configuration");

	let preferences = Settings::load(None);
	let colored_dice = ColoredDices::load(None);
	let normal_dices = Dices::load(None);
	let operation = Operation::load(None);
	let spells = Spells::load(None);
	let disadvantages: Vec<Disadvantage> = Disadvantage::load(None);
	let crits = CritDices::load(None);

	#[cfg(debug_assertions)]
	dbgprintln!(
		"Loading Configuration finished, took {} ms",
		start.elapsed().unwrap().as_millis()
	);

	let old = matches.get_flag("old_style") || preferences.old_style;
	let no_dice_select = matches.get_flag("no select dice select") || preferences.no_select_dice_select;
	let number_instead = matches.get_flag("number instead") || preferences.number_instead;
	let no_tutorial = matches.get_flag("no tutorial") || preferences.no_tutorial;
	let no_summary_message = matches.get_flag("no summary message") || preferences.no_summary_message;

	#[cfg(debug_assertions)]
	let error_message = format!(
		"{} {}: Nur Zahlen sind erlaubt! Maximal {}",
		file!(),
		line!(),
		u64::MAX
	);
	#[cfg(not(debug_assertions))]
	let error_message = format!("Nur Zahlen sind erlaubt! Maximal {}", u64::MAX);

	#[cfg(debug_assertions)]
	dbgprintln!("{:?}\n", preferences);
	#[cfg(debug_assertions)]
	dbgprintln!("{:?}\n", colored_dice);
	#[cfg(debug_assertions)]
	dbgprintln!("{:?}\n", normal_dices);
	#[cfg(debug_assertions)]
	dbgprintln!("{:?}\n", operation);
	#[cfg(debug_assertions)]
	dbgprintln!("{:?}\n", spells);
	#[cfg(debug_assertions)]
	dbgprintln!("{:?}\n", disadvantages);

	let mut finished = false;
	if !no_tutorial {
		print_startup_information(&colored_dice, &normal_dices);
	}

	let stdout = Term::stdout();
	let stderr = Term::stderr();
	let stdin = io::stdin();
	let items = vec![
		"Farbiger Würfel",
		"Normaler Würfel",
		"Crit",
		"Zerfallsreihen",
		"Random Zauber",
		"Random Nachteil",
		"Hilfe",
		"Verlassen",
	];

	while !finished {
		let selection = {
			let sel = Select::new().items(&items).default(1).interact_opt();
			match sel {
				Ok(s) => s,
				Err(_) => None
			}
		};

		if selection == None {
			finished = true;
			continue;
		}

		let answer = *items.get(selection.unwrap()).unwrap();

		if answer == "Farbiger Würfel" {
			match roll_colored_dice(
				&colored_dice,
				error_message.as_str(),
				number_instead,
				&stderr,
			) {
				Ok(_) => {}
				Err(err) => {
					eprintln!("{}", err);
					exit(-1);
				}
			}
		} else if answer == "Hilfe" {
			finished = handle_input(
				"h".parse().unwrap(),
				old,
				&colored_dice,
				&normal_dices,
				error_message.as_str(),
				no_summary_message,
			);
		} else if answer == "Crit" {
			let input: String = Input::new()
				.with_prompt("Anzahl")
				.validate_with(|input: &String| -> Result<(), &str> {
					let new_val = input.trim();
					if new_val.is_empty() {
						return Err("Bitte etwas eingeben");
					}
					match i16::from_str_radix(new_val, 10) {
						Ok(val) => {
							if val < 0 {
								Err("Bitte eine positive Ganzzahl eingeben")
							} else {
								Ok(())
							}
						}
						Err(_) => Err("Bitte eine positive Ganzzahl eingeben"),
					}
				})
				.interact_text()
				.unwrap();
			let count = match i16::from_str_radix(&*input, 10) {
				Ok(c) => c,
				Err(e) => {
					eprintln!("{}", e);
					continue;
				}
			};
			crits.roll(count);
		} else if answer == "Verlassen" {
			finished = true;
		} else if answer == "Zerfallsreihen" {
			decay_series(&stdout, &operation);
		} else if answer == "Random Zauber" {
			let string: String = "Kampfzauber".parse().unwrap();
			let items: Vec<String> = spells.iter().map(|x| x.name.clone()).collect();
			let default = match items.iter().position(|x| x.clone() == string) {
				None => 0,
				Some(index) => index,
			};

			let selection = {
				let interact_res = Select::new()
					.items(&items)
					.default(default)
					.interact_opt();

				match interact_res {
					Ok(res) => res,
					Err(_) => None
				}
			};

			match selection	{
				None => continue,
				Some(index) => dbgprintln!("{}", spells[index].roll()),
			}
		} else if answer == "Random Nachteil" {
			let rando = nachteil::get_random(&disadvantages);
			dbgprintln!("{}", rando);
		} else {
			dbgprint!("Seitenanzahl: ");
			if stdout.flush().is_err() {
				exit(-1)
			}

			if no_dice_select {
				let mut input = String::new();
				match stdin.read_line(&mut input) {
					Ok(_n) => {
						let removed = input.replace("\n", "");
						// Return value determines continuation of the loop, true ends the loop, false continues it
						finished = handle_input(
							removed,
							old,
							&colored_dice,
							&normal_dices,
							error_message.as_str(),
							no_summary_message,
						);
					}
					Err(error) => println!("error: {}", error),
				}
			} else {
				let mut dice_items: Vec<String> = vec![];
				for allowed_dice_site in normal_dices.dices.iter() {
					dice_items.push(allowed_dice_site.to_string())
				}

				let selection = {
					let interact_res = Select::new()
						.items(&dice_items)
						.default(3)
						.interact_on_opt(&Term::stderr());
					match interact_res {
						Ok(res) => res,
						Err(_) => None
					}
				};
				if selection == None {
					continue;
				}
				let input: &str = match selection {
					None => continue,
					Some(sec) => &dice_items.get(sec).unwrap(),
				};

				finished = handle_input(
					input.to_string(),
					old,
					&colored_dice,
					&normal_dices,
					error_message.as_str(),
					no_summary_message,
				);
			}
		}
	}
	Ok(())
}
