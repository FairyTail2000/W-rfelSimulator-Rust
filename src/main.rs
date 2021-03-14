#[macro_use]
mod macros;
mod color;
mod colored_dice;
mod normal_dice;
mod preferences;

use crate::color::get_color;
use crate::colored_dice::{ColoredDice, ColoredDices};
use crate::normal_dice::{Dice, Dices, PrintResult};
use crate::preferences::Settings;
use ansi_term::Colour;
use clap::{App, Arg};
use dialoguer::console::Term;
use dialoguer::{Input, MultiSelect, Select};
use std::borrow::Borrow;
use std::io;
use std::io::Error;
use std::io::Write;
use std::process::exit;
#[cfg(debug_assertions)]
use std::time::SystemTime;

/**
* Prints basic information's about the usage of the program
* Also used for help message
*/
fn print_startup_information(allowed_coloured_dices: &ColoredDices, allowed_dice_sites: &Dices) {
    #[cfg(not(debug_assertions))]
    print!("Erlaubte Würfelseiten:");
    dbgprint!("Erlaubte Würfelseiten:");
    let mut vector: Vec<String> = vec![];
    for (index, site) in allowed_dice_sites.dices.iter().enumerate() {
        vector.push((*format!(" {}", site.sides)).parse().unwrap());
        if allowed_dice_sites.len() != index + 1 {
            vector.push((*format!(",")).parse().unwrap());
        }
    }
    #[cfg(not(debug_assertions))]
    println!("{}", vector.join(""));
    dbgprintln!("{}", vector.join(""));

    vector.clear();

    #[cfg(not(debug_assertions))]
    print!("Erlaubte farbige Seiten:");
    dbgprint!("Erlaubte farbige Seiten:");

    for (index, site) in allowed_coloured_dices.dices.iter().enumerate() {
        let color: Colour = get_color(&site.color);

        vector.push(
            (*format!(" {} ({})", color.paint(&site.long), site.short))
                .parse()
                .unwrap(),
        );
        if allowed_coloured_dices.len() != index + 1 {
            vector.push((*format!(",")).parse().unwrap());
        }
    }

    #[cfg(not(debug_assertions))]
    println!("{}", vector.join(""));
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
    return if input == "exit" || input == "e" {
        true
    } else if input == "help" || input == "h" {
        print_startup_information(allowed_colored_dices, allowed_dice_sites);
        false
    } else {
        let parsed = input.parse::<u8>();
        if let Err(_err) = parsed {
            #[cfg(not(debug_assertions))]
            println!("Es muss eine Ganzzahl sein, wie oben beschrieben");
            dbgprintln!("Es muss eine Ganzzahl sein, wie oben beschrieben");
            false
        } else if let Ok(sides) = parsed {
            if allowed_dice_sites.dices.contains(&Dice { sides }) {
                let amount = ask_for_amount(error_message, "Anzahl");

                let res = normal_dice::roll(amount, sides);
                res.print_results(old_report_style, no_summary);
            } else {
                #[cfg(not(debug_assertions))]
                println!("Die ist nicht erlaubt...");
                dbgprintln!("Die ist nicht erlaubt...")
            }
            false
        } else {
            false
        }
    };
}

fn ask_for_amount(error_message: &str, prompt: &str) -> u64 {
    let input: Result<String, Error> = Input::new()
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
        result.parse().unwrap()
    } else {
        0
    };
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Würfeln")
		.version("1.0.0")
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
			.about("Unterdrückt die Start Nachricht")
		)
		.arg(Arg::new("no summary message")
			.short('s')
			.long("no-summary-message")
			.about("Unterdrückt die kurze information nachdem das Würfelergebnis ausgegeben wurde")
		)
		.arg(Arg::new("no select dice select")
			.short('d')
			.long("no-select-dice-select")
			.about("Verwendet die standart Eingabe anstatt einer Auswahl")
		)
		.arg(Arg::new("number instead")
			.short('i')
			.long("number-instead")
			.about("Verwendet eine Zahlen eingabe anstatt einer Auswahl und Anzahl von farbigen würfeln")
		)
		.get_matches();

    #[cfg(debug_assertions)]
    let start: SystemTime = SystemTime::now();
    dbgprintln!("Loading Configuration");

    let preferences = Settings::load();
    let colored_dice = ColoredDices::load();
    let normal_dices = Dices::load();

    dbgprintln!(
        "Loading Configuration finished, took {} ms",
        start.elapsed().unwrap().as_millis()
    );

    let old = matches.is_present("old_style") || preferences.old_style;
    let no_dice_select =
        matches.is_present("no select dice select") || preferences.no_select_dice_select;
    let number_instead = matches.is_present("number instead") || preferences.number_instead;
    let no_tutorial = matches.is_present("no tutorial") || preferences.no_tutorial;
    let no_summary_message =
        matches.is_present("no summary message") || preferences.no_summary_message;

    #[cfg(debug_assertions)]
    let error_message = format!(
        "{} {}: Nur Zahlen sind erlaubt! Maximal {}",
        file!(),
        line!(),
        std::u64::MAX
    );
    #[cfg(not(debug_assertions))]
    let error_message = format!("Nur Zahlen sind erlaubt! Maximal {}", std::u64::MAX);

    dbgprintln!("{:?}", preferences);
    dbgprintln!("{:?}", colored_dice);
    dbgprintln!("{:?}", normal_dices);

    let mut finished = false;
    if !no_tutorial {
        print_startup_information(&colored_dice, &normal_dices);
    }

    while !finished {
        let items = vec!["Farbiger Würfel", "Normaler Würfel", "Hilfe", "Verlassen"];
        let selection = Select::new()
            .items(&items)
            .default(1)
            .interact_on_opt(&Term::stderr())?;

        if selection == None {
            finished = true;
            continue;
        }

        let answer = items.get(selection.unwrap());

        if answer.unwrap() == &"Farbiger Würfel" {
            let mut possibilities: Vec<&str> = vec![];
            if number_instead {
                //Input a number and auto compute values
                let amount = ask_for_amount(error_message.as_str(), "Farbiger Würfel Wert");
                //Tuple of value, amount and result
                let mut dices: Vec<(u8, u64, u64, String)> = Vec::new();
                let mut remaining = amount;

                let mut copy: Vec<ColoredDice> = colored_dice.dices.to_vec();

                copy.sort_by(|a, b| a.value.cmp(&b.value).reverse());
                for dice in copy {
                    let mut result: u64 = 0;
                    for _ in 0..remaining / dice.value as u64 {
                        result += *dice.get_random() as u64;
                    }
                    let value = (dice.value, remaining / dice.value as u64, result, dice.long);
                    dices.push(value);
                    remaining %= dice.value as u64;
                }

                let mut accumulated_result: u64 = 0;
                for result in dices {
                    dbgprintln!("{}: {}", result.3, result.2);

                    #[cfg(not(debug_assertions))]
                    println!("{}: {}", result.3, result.2);

                    accumulated_result += result.2;
                }

                dbgprintln!(
                    "Insgesamt: {} ({})",
                    accumulated_result,
                    accumulated_result * 10
                );

                #[cfg(not(debug_assertions))]
                println!(
                    "Insgesamt: {} ({})",
                    accumulated_result,
                    accumulated_result * 10
                );
            } else {
                // Use multiselect...
                for (_index, dice) in colored_dice.dices.iter().enumerate() {
                    possibilities.push(&*dice.long);
                }

                let selection = MultiSelect::new()
					.items(&possibilities)
					.with_prompt("Wähle deine farbigen Würfel (Mit der Leertaste auswählen und Enter bestätigen)")
					.interact_on(&Term::stderr())?;
                if selection.len() == 0 {
                    dbgprintln!("Nichts gewählt.");
                    #[cfg(not(debug_assertions))]
                    println!("Nichts gewählt.");

                    continue;
                }

                let mut result: Vec<(&String, u64)> = Vec::new();
                let mut accumulated_amount: u64 = 0;
                for select in selection {
                    let sel = colored_dice.dices.get(select).unwrap();
                    let amount =
                        ask_for_amount(error_message.as_str(), &*format!("Anzahl {}", sel.long));
                    // Amount but shorter
                    let mut am: u64 = 0;
                    for _ in 0..amount {
                        am += *(sel.get_random()) as u64;
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

                    #[cfg(not(debug_assertions))]
                    println!("{}: {}", res.0, res.1);
                    #[cfg(not(debug_assertions))]
                    println!(
                        "Insgesamt: {} ({})",
                        accumulated_amount,
                        accumulated_amount * 10
                    )
                }
            }
        } else if answer.unwrap() == &"Hilfe" {
            finished = handle_input(
                "h".parse().unwrap(),
                old,
                &colored_dice,
                &normal_dices,
                error_message.as_str(),
                no_summary_message,
            );
        } else if answer.unwrap() == &"Verlassen" {
            finished = true;
        } else {
            if no_dice_select {
                dbgprint!("Seitenanzahl: ");
                #[cfg(not(debug_assertions))]
                print!("Seitenanzahl: ");
            } else {
                dbgprint!("Seitenanzahl: ");
                #[cfg(not(debug_assertions))]
                print!("Seitenanzahl: ");
            }
            let res = io::stdout().flush();
            if let Err(_e) = res {
                exit(-1)
            }

            if no_dice_select {
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
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
                    dice_items.push(allowed_dice_site.sides.to_string())
                }

                let selection = Select::new()
                    .items(&dice_items)
                    .default(3)
                    .interact_on_opt(&Term::stderr())?;
                if selection == None {
                    continue;
                }

                // Type annotation needed because generic borrow does not have runtime information at this point
                let input: &str = dice_items.get(selection.unwrap()).unwrap().borrow();

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
