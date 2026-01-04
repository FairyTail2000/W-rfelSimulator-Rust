mod preferences;
mod color;
mod dice;
mod spell;
mod disadvantage;
mod decay_series;
mod common;

use color::get_color;
use crate::preferences::Settings;
use clap::{Arg, Command};
use dice::colored_dice::{ColoredDice, ColoredDices};
use common::{Loadable, Rollable};
use dice::crit_dice::CritDices;
use dialoguer::console::Term;
use dialoguer::{Input, MultiSelect, Select};
use disadvantage::Disadvantage;
use dice::normal_dice::Dices;
use std::io;
use std::io::Write;
use std::time::SystemTime;
use spell::Spells;
use decay_series::Operation;
use decay_series::State;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_rdseed64_step;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/**
 * Prints basic information's about the usage of the program
 * Also used for help message
 */
fn print_startup_information(allowed_coloured_dices: &ColoredDices, allowed_dice_sites: &Dices) {
	let normal_dices = allowed_dice_sites.dices
		.iter()
		.map(|site| format!("{}", site))
		.collect::<Vec<_>>()
		.join(", ");
	let colored_dices: String = allowed_coloured_dices.dices
		.iter()
		.filter_map(|site| {
			get_color(&site.color)
				.map(|color| format!("{} ({})", color.paint(&site.long), site.short))
				.map_err(|e| edbgprintln!("{}", e))
				.ok()
		})
		.collect::<Vec<_>>()
		.join(", ");

	dbgprintln!("Erlaubte Würfelseiten:\n{}", normal_dices);
	dbgprintln!("Erlaubte farbige Seiten:\n{}", colored_dices);

	if let Err(_e) = io::stdout().flush() {
		edbgprintln!("Fehler beim flushen von stdout")
	}
}

fn handle_input(
	input: &str,
	old_report_style: bool,
	allowed_colored_dices: &ColoredDices,
	allowed_dice_sites: &Dices,
	error_message: &str,
	no_summary: bool,
	rng: &mut impl Rng
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
		} else if let Ok(sides) = parsed {
			if allowed_dice_sites.dices.contains(&sides) {
				let amount = ask_for_amount(error_message, "Anzahl");
				let res = dice::normal_dice::roll(amount, sides, rng);
				res.print_results(old_report_style, no_summary);
			} else {
				dbgprintln!("Die ist nicht erlaubt...")
			}
		}
		false
	}
}

fn ask_for_amount(error_message: &str, prompt: &str) -> usize {
	let input = Input::new()
		.with_prompt(prompt)
		.validate_with(|input: &String| -> Result<(), &str> {
			input.parse::<u64>()
				.map(|_| ())
				.map_err(|_| error_message)
		})
		.interact_text();
	input.unwrap_or_else(|_e| "0".to_string())
		.parse()
		.unwrap_or_else(|_e| 0)
}

fn validator(val: &String) -> Result<(), &'static str> {
	let new_val = val.trim();
	if new_val.is_empty() {
		return Err("Bitte etwas eingeben");
	}

	i64::from_str_radix(new_val, 10)
		.map(|_| ())
		.map_err(|_| "Bitte eine positive oder negative Ganzzahl eingeben")
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
	if let Err(e) = stdout.clear_last_lines(3) {
		edbgprintln!("Terminal Fehler: {}", e);
	}

	let protons = protons_input
		.map(|inp| i64::from_str_radix(inp.trim(), 10).unwrap_or_else(|_| 0))
		.unwrap_or_else(|_| 0);

	let neutrons = neutrons_input
		.map(|inp| i64::from_str_radix(inp.trim(), 10).unwrap_or_else(|_| 0))
		.unwrap_or_else(|_| 0);

	let electrons = electrons_input
		.map(|inp| i64::from_str_radix(inp.trim(), 10).unwrap_or_else(|_| 0))
		.unwrap_or_else(|_| 0);

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
		let i = match selection {
			Ok(i) if i < operation.len() => i,
			_ => break,
		};

		let new = operation[i].apply(state);
		if new.protons < 0 || new.electrons < 0 || new.neutrons < 0 {
			dbgprintln!("Nicht möglich!");
		} else {
			state = new;
			dbgprintln!("{}", state);
		}
	}
	if let Err(e) = stdout.clear_last_lines(3) {
		edbgprintln!("Terminal Fehler: {}", e);
	}
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
	rng: &mut impl Rng
) -> io::Result<()> {
	if number_instead {
		//Input a number and auto compute values
		let amount = ask_for_amount(error_message, "Farbiger Würfel Wert");
		//Tuple of value, amount and result
		let mut dices: Vec<(u64, String)> = Vec::with_capacity(amount);
		let mut remaining = amount;

		let mut copied_dices: Vec<ColoredDice> = colored_dice.dices.to_vec();

		copied_dices.sort_by(|a, b| a.value.cmp(&b.value).reverse());
		for dice in copied_dices {
			let result: u64 = (0..remaining / dice.value as usize)
				.map(|_| dice.roll(rng) as u64)
				.sum();
			dices.push((result, dice.long));
			remaining %= dice.value as usize;
		}

		let accumulated_result: u64 = dices.iter()
			.map(|x| x.0)
			.sum();
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
		let possibilities: Vec<&str> = colored_dice.dices.iter().map(|dice| &*dice.long).collect();
		let selection = MultiSelect::new()
			.items(&possibilities)
			.with_prompt(
				"Wähle deine farbigen Würfel (Mit der Leertaste auswählen und Enter bestätigen)",
			)
			.interact_on(stderr)
			.unwrap_or_else(|_err| vec![]);

		if selection.is_empty() {
			dbgprintln!("Nichts gewählt.");
			return Ok(());
		}

		let mut result: Vec<(&String, u64)> = Vec::with_capacity(selection.len());
		let mut accumulated_amount: u64 = 0;
		let selection = selection.into_iter()
			.filter_map(|select| colored_dice.dices.get(select))
			.map(|dice| (dice, ask_for_amount(error_message, &*format!("Anzahl {}", dice.long))));
		for (dice, amount) in selection {
			let amount = (0..amount)
				.map(|_| dice.roll(rng) as u64)
				.sum();
			accumulated_amount += amount;
			result.push((&dice.long, amount))
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

	let mut rng = {
		let mut seed_value= 0;
		#[cfg(target_arch = "x86_64")]
		if is_x86_feature_detected!("rdseed") {
			unsafe { _rdseed64_step(&mut seed_value); }
		}

		// rdseed failed (or this is not an x86-64 system, result is 0 also in bad systems that return success on failure (AMD)
		if seed_value == 0 {
			// C style initialization
			seed_value = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
		}
		SmallRng::seed_from_u64(seed_value)
	};

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
	{
		dbgprintln!("{:?}\n", preferences);
		dbgprintln!("{:?}\n", colored_dice);
		dbgprintln!("{:?}\n", normal_dices);
		dbgprintln!("{:?}\n", operation);
		dbgprintln!("{:?}\n", spells);
		dbgprintln!("{:?}\n", disadvantages);

	}

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
		let selection = Select::new()
				.items(&items)
				.default(1)
				.interact_opt()
				.ok()
				.flatten()
				.and_then(|sel| items.get(sel))
				.copied();

		let answer = match selection {
			Some(answer) => answer,
			None => {
				finished = true;
				continue;
			}
		};

		match answer {
			"Farbiger Würfel" => {
				if let Err(err) = roll_colored_dice(
					&colored_dice,
					&error_message,
					number_instead,
					&stderr,
					&mut rng,
				) {
					edbgprintln!("{}", err);
					continue
				}
			},
			"Hilfe" => {
				finished = handle_input(
					"h",
					old,
					&colored_dice,
					&normal_dices,
					&error_message,
					no_summary_message,
					&mut rng,
				);
			},
			"Crit" => {
				let input = Input::new()
					.with_prompt("Anzahl")
					.validate_with(|input: &String| -> Result<(), &str> {
						let new_val = input.trim();
						if new_val.is_empty() {
							return Err("Bitte etwas eingeben");
						}

						i16::from_str_radix(new_val, 10)
							.ok()
							.filter(|&val| val >= 0)
							.map(|_| ())
							.ok_or("Bitte eine positive Ganzzahl eingeben")
					})
					.interact_text()
					.map_err(|err| err.to_string())
					.and_then(|inp| inp.parse::<i16>().map_err(|err| err.to_string()));

				match input {
					Ok(count) => crits.roll(count, &mut rng),
					Err(err) => eprintln!("{}", err),
				}
			},
			"Verlassen" => {
				finished = true;
			},
			"Zerfallsreihen" => {
				decay_series(&stdout, &operation);
			},
			"Random Zauber" => {
				let search_string = "Kampfzauber";
				let items: Vec<&str> = spells.iter().map(|x| &*x.name).collect();
				let default = items.iter().position(|x| *x == search_string).unwrap_or_else(|| 0);

				let selection = Select::new()
					.items(&items)
					.default(default)
					.interact_opt()
					.unwrap_or_else(|_| None);

				if let Some(index) = selection {
					dbgprintln!("{}", spells[index].roll(&mut rng))
				}
			},
			"Random Nachteil" => {
				let rando = disadvantage::get_random(&disadvantages, &mut rng);
				dbgprintln!("{}", rando);
			},
			_ => {
				dbgprint!("Seitenanzahl: ");
				if let Err(err) = stdout.flush() {
					edbgprintln!("Fehler bei der Terminal interaktion: {}", err)
				}

				if no_dice_select {
					let mut input = String::new();
					match stdin.read_line(&mut input) {
						Ok(_) => {
							// Return value determines continuation of the loop, true ends the loop, false continues it
							finished = handle_input(
								&*input.replace("\n", ""),
								old,
								&colored_dice,
								&normal_dices,
								&error_message,
								no_summary_message,
								&mut rng,
							);
						}
						Err(error) => edbgprintln!("error: {}", error),
					}
				} else {
					let dice_items: Vec<String> = normal_dices.dices.iter().map(|x| x.to_string()).collect();
					let selection = Select::new()
						.items(&dice_items)
						.default(3)
						.interact_on_opt(&Term::stderr())
						.unwrap_or_else(|_| None)
						.and_then(|index| dice_items.get(index));

					if let Some(input) = selection {
						finished = handle_input(
							input,
							old,
							&colored_dice,
							&normal_dices,
							error_message.as_str(),
							no_summary_message,
							&mut rng,
						);
					}
				}
			}
		}
	}
	Ok(())
}
