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
use common::macros::{dbgprint, dbgprintln, edbgprintln};
use disadvantage::Disadvantage;
use dice::normal_dice::Dices;
use std::io;
use std::io::Write;
use std::num::ParseIntError;
use std::ops::Deref;
use std::process::exit;
#[cfg(debug_assertions)]
use std::time::SystemTime;
use spell::Spells;
use decay_series::operation::Operation;
use decay_series::State;

/**
 * Prints basic information's about the usage of the program
 * Also used for help message
 */
fn print_startup_information(allowed_coloured_dices: &ColoredDices, allowed_dice_sites: &Dices) {
	dbgprint!("Erlaubte Würfelseiten:");
	let mut vector: Vec<String> = allowed_dice_sites.dices.iter().enumerate().map(|(index, site)| {
		if allowed_dice_sites.len() != index + 1 {
			format!(" {},", site)
		} else {
			format!(" {}", site)
		}
	}).collect();
	dbgprintln!("{}", vector.join(""));

	vector.clear();

	dbgprint!("Erlaubte farbige Seiten:");

	for (index, site) in allowed_coloured_dices.dices.iter().enumerate() {
		let color = match get_color(&site.color) {
			Ok(c) => c,
			Err(e) => {
				edbgprintln!("{}", e);
				exit(-1);
			}
		};
		vector.push(format!(" {} ({})", color.paint(&site.long), site.short));

		if allowed_coloured_dices.len() != index + 1 {
			vector.push(",".to_string());
		}
	}

	dbgprintln!("{}", vector.join(""));

	if io::stdout().flush().is_err() {
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
	use_hw_rng: bool,
) -> bool {
	if input == "exit" || input == "e" {
		true
	} else if input == "help" || input == "h" {
		print_startup_information(allowed_colored_dices, allowed_dice_sites);
		false
	} else {
		match input.parse::<u8>() {
			Ok(sides) => {
				if allowed_dice_sites.dices.contains(&sides) {
					let amount = ask_for_amount(error_message, "Anzahl");
					let res = if use_hw_rng {
						dice::normal_dice::roll_native(amount, sides).unwrap_or_else(|| {
							eprintln!("Es ist ein Fehler aufgetreten beim Würfeln! Hardware Fehler, software generator wird verwendet!");
							dice::normal_dice::roll(amount, sides)
						})
					} else {
						dice::normal_dice::roll(amount, sides)
					};
					res.print_results(old_report_style, no_summary);
				} else {
					dbgprintln!("Die ist nicht erlaubt...")
				}
				false
			}
			Err(_) => {
				dbgprintln!("Es muss eine Ganzzahl sein, wie oben beschrieben");
				false
			}
		}
	}
}

fn ask_for_amount(error_message: &str, prompt: &str) -> usize {
	Input::new()
		.with_prompt(prompt)
		.validate_with(|input: &String| input.parse::<u64>().map_or(Err(error_message), |_| Ok(())))
		.interact_text()
		.map_or(0, |x| x.parse().unwrap_or(0))
}

fn validator(val: &String) -> Result<(), &'static str> {
	let new_val = val.trim();
	if new_val.is_empty() {
		return Err("Bitte etwas eingeben");
	}
	i64::from_str_radix(new_val, 10).map_or(Err("Bitte eine positive oder negative Ganzzahl eingeben"), |x| {
		if x < 0 {
			Err("Bitte eine positive Ganzzahl eingeben")
		} else {
			Ok(())
		}
	})
}

fn decay_series(stdout: &Term, operation: &Vec<Operation>) {
	let protons = Input::new()
		.with_prompt("Protonen")
		.validate_with(validator)
		.interact_text()
		.map_or(0, |x| i64::from_str_radix(&*x.trim(), 10).unwrap_or(0));
	let neutrons = Input::new()
		.with_prompt("Neutronen")
		.validate_with(validator)
		.interact_text()
		.map_or(0, |x| i64::from_str_radix(&*x.trim(), 10).unwrap_or(0));
	let electrons = Input::new()
		.with_prompt("Elektronen")
		.validate_with(validator)
		.interact_text()
		.map_or(0, |x| i64::from_str_radix(&*x.trim(), 10).unwrap_or(0));
	let _ = stdout.clear_last_lines(3);

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
	stdout.clear_last_lines(1).expect("Failed to clear last line");
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
		.arg(Arg::new("hwrng")
			.short('w')
			.long("hwrng")
			.help("Verwende den HWRNG (langsamer) anstatt des Software Generators")
			.action(clap::ArgAction::SetTrue)
		)
}

fn roll_colored_dice(
	colored_dice: &ColoredDices,
	error_message: &str,
	number_instead: bool,
	stderr: &Term,
	use_hw_rng: bool,
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
				result += *dice.roll(use_hw_rng) as u64;
			}
			let value = (result, dice.long);
			dices.push(value);
			remaining %= dice.value as usize;
		}

		let mut accumulated_result: u64 = dices.iter().fold(0, |acc, x| acc + x.0);
		for result in dices {
			dbgprintln!("{}: {}", result.1, result.0);
		}

		dbgprintln!(
			"Insgesamt: {} ({})",
			accumulated_result,
			accumulated_result * 10
		);
	} else {
		// Use multiselect...
		possibilities = colored_dice.dices.iter().map(|x| x.long.as_str()).collect();
		let selection = MultiSelect::new()
				.items(&possibilities)
				.with_prompt("Wähle deine farbigen Würfel (Mit der Leertaste auswählen und Enter bestätigen)")
				.interact_on(stderr)
			.unwrap_or_else(|_err| vec![]);
		if selection.is_empty() {
			dbgprintln!("Nichts gewählt.");
			return Ok(());
		}

		let mut result: Vec<(&String, u64)> = Vec::with_capacity(selection.len());
		let mut accumulated_amount: u64 = 0;
		for select in selection {
			let sel = match colored_dice.dices.get(select) {
				Some(val) => val,
				None => continue
			};
			let amount = ask_for_amount(error_message, &*format!("Anzahl {}", sel.long));
			// Amount but shorter
			let mut am: u64 = 0;
			for _ in 0..amount {
				am += *(sel.roll(use_hw_rng)) as u64;
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
	match start.elapsed() {
		Ok(elapsed) => dbgprintln!("Loading Configuration finished, took {} ms", elapsed.as_millis()),
		Err(err) => edbgprintln!("{}", err)
	}

	let old = matches.get_flag("old_style") || preferences.old_style;
	let no_dice_select = matches.get_flag("no select dice select") || preferences.no_select_dice_select;
	let number_instead = matches.get_flag("number instead") || preferences.number_instead;
	let no_tutorial = matches.get_flag("no tutorial") || preferences.no_tutorial;
	let no_summary_message = matches.get_flag("no summary message") || preferences.no_summary_message;
	let use_hw_rng = matches.get_flag("hwrng");
	
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
		let selection = Select::new().items(&items).default(1).interact_opt().unwrap_or_else(|_| None);

		if selection == None {
			finished = true;
			continue;
		}

		let answer = match selection.and_then(|sel| items.get(sel)) {
			None => {
				continue
			}
			Some(sel) => {
				*sel
			}
		};

		if answer == "Farbiger Würfel" {
			roll_colored_dice(
				&colored_dice,
				error_message.as_str(),
				number_instead,
				&stderr,
				use_hw_rng,
			).unwrap_or_else(|e| eprintln!("{}", e));
		} else if answer == "Hilfe" {
			let h = match "h".parse() {
				Ok(res) => res,
				Err(e) => {
					edbgprintln!("{}", e);
					continue
				}
			};
			finished = handle_input(
				h,
				old,
				&colored_dice,
				&normal_dices,
				error_message.as_str(),
				no_summary_message,
				use_hw_rng
			);
		} else if answer == "Crit" {
			let input = Input::new()
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
				.interact_text();

			let count = match input {
				Ok(inp) => {
					match i16::from_str_radix(&*inp, 10) {
						Ok(c) => c,
						Err(e) => {
							eprintln!("{}", e);
							continue;
						}
					}
				}
				Err(err) => {
					eprintln!("{}", err);
					continue;
				}
			};
			crits.roll(count, use_hw_rng);
		} else if answer == "Verlassen" {
			finished = true;
		} else if answer == "Zerfallsreihen" {
			decay_series(&stdout, &operation);
		} else if answer == "Random Zauber" {
			let string: String = "Kampfzauber".into();
			let items: Vec<String> = spells.iter().map(|x| x.name.clone()).collect();
			let default = items.iter().position(|x| x.clone() == string).unwrap_or(0);

			let selection = Select::new()
					.items(&items)
					.default(default)
					.interact_opt()
					.unwrap_or(None);

			match selection	{
				None => continue,
				Some(index) => dbgprintln!("{}", spells[index].roll(use_hw_rng)),
			}
		} else if answer == "Random Nachteil" {
			let rando = disadvantage::get_random(&disadvantages);
			dbgprintln!("{}", rando);
		} else {
			if no_dice_select {
				dbgprint!("Seitenanzahl: ");
				if stdout.flush().is_err() {
					exit(-1)
				}
				
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
							use_hw_rng
						);
					}
					Err(error) => edbgprintln!("error: {}", error),
				}
			} else {
				let mut dice_items: Vec<String> = vec![];
				for allowed_dice_site in normal_dices.dices.iter() {
					dice_items.push(allowed_dice_site.to_string())
				}

				let selection = Select::new()
						.items(&dice_items)
						.default(3)
						.interact_on_opt(&Term::stderr())
						.unwrap_or(None);
				if selection == None {
					continue;
				}
				let input: &str = match selection {
					None => continue,
					Some(sec) => match dice_items.get(sec) {
						None => continue,
						Some(item) => &item
					}
				};

				finished = handle_input(
					input.to_string(),
					old,
					&colored_dice,
					&normal_dices,
					error_message.as_str(),
					no_summary_message,
					use_hw_rng
				);
			}
		}
	}
	Ok(())
}
