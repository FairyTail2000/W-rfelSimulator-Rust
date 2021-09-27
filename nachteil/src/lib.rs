use common::{settings_path, Loadable};
use random_integer::random_usize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, Default, Hash, Debug, Clone)]
pub struct Disadvantage {
	name: String,
	description: String,
}

impl Loadable<Vec<Disadvantage>> for Disadvantage {
	fn load(file: Option<&str>) -> Vec<Disadvantage> {
		let alt = settings_path("nachteile.yaml");
		let file_name = file.unwrap_or(alt.to_str().unwrap());
		if Path::new(file_name).exists() {
			let file = File::open(file_name).unwrap();
			let buf_reader = BufReader::new(file);
			match serde_yaml::from_reader::<BufReader<File>, Vec<Disadvantage>>(buf_reader) {
				Ok(disadvantages) => disadvantages,
				Err(err) => {
					eprintln!("{}", err);
					let file = OpenOptions::new()
						.write(true)
						.truncate(true)
						.open(file_name)
						.unwrap();
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer(writer, &Disadvantage::defaults()) {
						Ok(_) => {}
						Err(err) => {
							eprintln!("Couldn't write default values to file!");
							eprintln!("{}", err);
						}
					}
					Disadvantage::defaults()
				}
			}
		} else {
			match File::create(file_name) {
				Ok(file) => {
					let writer = BufWriter::new(file);
					match serde_yaml::to_writer::<BufWriter<File>, Vec<Disadvantage>>(
						writer,
						&Disadvantage::defaults(),
					) {
						Ok(_) => {
							println!("Neue Nachteile wurden erzeugt");
						}
						Err(err) => {
							eprintln!("{}", err)
						}
					}
				}
				Err(err) => {
					eprintln!("{}", err);
				}
			}
			Disadvantage::defaults()
		}
	}
}

impl Disadvantage {
	pub fn defaults() -> Vec<Disadvantage> {
		return vec![
			Disadvantage {
				name: "Abergläubisch".parse().unwrap(),
				description: "Charakter glaubt an verschiedenen Aberglauben. Wann immer eine Theorie des Aberglaubens geschieht, wird auch in absehbarer Zeit etwas Schlechtes passieren. Das ist nicht dramatisch, aber schon nicht ganz so nett.".parse().unwrap()
			},
			Disadvantage {
				name: "Abgezählt".parse().unwrap(),
				description: "Der Charakter hat nur noch drei \"Stories\" zu leben. Danach stirbt er entgültig. Der Charakter erhält dreifache EP nach jeder Story.".parse().unwrap()
			},
			Disadvantage {
				name: "Adrenalinjunkie".parse().unwrap(),
				description: "Charakter liebt die Gefahr, was ihn dazu treibt manchmal dumme oder riskante Dinge zu tun.".parse().unwrap()
			},
			Disadvantage {
				name: "Alkoholiker".parse().unwrap(),
				description: "Charakter muss regelmäßig Alkohol konsumieren. Er erleidet sonst Entzugserscheinungen. Der Charakter ist äußerst impulsiv bei Streitereien, neigt dort zu Gewalt und erhält bei UM-Proben gegenüber Nüchternen immer -4.".parse().unwrap()
			},
			Disadvantage {
				name: "Allergie".parse().unwrap(),
				description: "Schaden durch gewisse Stoffe + Malus von -4".parse().unwrap()
			},
			Disadvantage {
				name: "Amnesie".parse().unwrap(),
				description: "Vergesslichkeit. IN = 1 (fix) Nur wählbar, wenn IN auch mindestens 1/1 ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Analphabet".parse().unwrap(),
				description: "Charakter kann nicht lesen und schreiben.".parse().unwrap()
			},
			Disadvantage {
				name: "Angst".parse().unwrap(),
				description: "Charakter hat Angst vor etwas. +1 bis +5 IP Während der Angst muss der Charakter auf Sanität würfeln. Sinkt diese auf 0, wird der Sanitäts-Würfel gewürfelt.".parse().unwrap()
			},
			Disadvantage {
				name: "Asozial".parse().unwrap(),
				description: "Person mag andere Leute nicht. UM = 1 (fix) Nur wählbar, wenn UM auch mindestens 1/1 ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Astrales Leuchtfeuer".parse().unwrap(),
				description: "Charakter wird durch seine Aura meilenweit gesehen. Er kann diese auch nicht verbergen. Geister meiden den Charakter oder greifen ihn aus Panik wild an.".parse().unwrap()
			},
			Disadvantage {
				name: "Berüchtigt".parse().unwrap(),
				description: "5 von 10 Leuten kennen den Charakter in negativem Zusammenhang, da er mal etwas Schlimmes getan hat oder für etwas Schlimmes verantwortlich ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Besessen".parse().unwrap(),
				description: "Person wird von einem Geist oder Dämon bewohnt. Dieser kann in gewissen Situationen in Geschehnisse eingreifen.".parse().unwrap()
			},
			Disadvantage {
				name: "Blind".parse().unwrap(),
				description: "Charakter kann nicht sehen und bekommt einen Würfelpool von -20 auf Wahrnehmung \"Sehen\". Auch alle Aktionen, die Sicht benötigen werden erschwert.".parse().unwrap()
			},
			Disadvantage {
				name: "Blutrausch".parse().unwrap(),
				description: "Regelmäßige Entschlossenheits-Probe umd sich zu beherrschen (unter 4 Erfolge → Spieler greift mit Waffenloser Kampf an)".parse().unwrap()
			},
			Disadvantage {
				name: "Chronische Kopfschmerzen".parse().unwrap(),
				description: "Dauerhaft auf alle Proben mit den Attributen IN, UM und WK einen Malus von -3.".parse().unwrap()
			},
			Disadvantage {
				name: "Codex".parse().unwrap(),
				description: "Charakter kann eine bestimmte Sorte Personen nicht töten.".parse().unwrap()
			},
			Disadvantage {
				name: "Cyberunverträglichkeit".parse().unwrap(),
				description: "Manifestverlust durch Cybereinbauten wird verdoppelt. Nur wählbar, wenn es realistisch erscheint, dass der Charakter irgendwann mal Cyberware einbauen will.".parse().unwrap()
			},
			Disadvantage {
				name: "Defizit".parse().unwrap(),
				description: "Ein Attribut wird auf 0 gesenkt. Die Fertigkeiten darin sind dadurch nicht mehr möglich. Nur wählbar, wenn das entsprechende Attribut auch mindestens 1/1 ist. Verschiede Attribute geben unterschiedlich viele IP: SCH, IN, ST oder WK geben 20 IP. VER oder MA geben 8 IP. UM, GES, N oder F geben 2 IP.".parse().unwrap()
			},
			Disadvantage {
				name: "Depressiv".parse().unwrap(),
				description: "Ein Erfolg wird immer zu einer 1.".parse().unwrap()
			},
			Disadvantage {
				name: "Diskalkulie".parse().unwrap(),
				description: "Charakter hat Probleme bei Rechenaufgaben. Mal bekommt er zufällige Mali und manchmal Würfelboni auf Proben, die aber logisch begründet sind. Nur halt etwas weit hergeholt.".parse().unwrap()
			},
			Disadvantage {
				name: "Drogenabhängig".parse().unwrap(),
				description: "Eine Droge muss 2x täglich konsumiert werden. Die Intervalle werden mit der Zeit kürzer. Bei Entzug sinkt WK auf 1. Irgendwann kann der Konsum der Drogen böse Folgen haben.".parse().unwrap()
			},
			Disadvantage {
				name: "Egoist".parse().unwrap(),
				description: "Charakter wird immer nur an sein Wohlergehen denken und nie an das anderer. Positive Affektivitätswürfe werden halbiert.".parse().unwrap()
			},
			Disadvantage {
				name: "Eifersüchtig".parse().unwrap(),
				description: "Charakter muss verliebt oder fanatisch in/nach irgendetwas/irgendjemanden sein. Passiert etwas damit, wird der Charakter aggressiv oder wütend und neigt dazu handgreiflich zu werden.".parse().unwrap()
			},
			Disadvantage {
				name: "Empfindlich".parse().unwrap(),
				description: "Schadensmodifikatoren werden verdoppelt.".parse().unwrap()
			},
			Disadvantage {
				name: "Fanatisch".parse().unwrap(),
				description: "Charakter ist in irgendetwas extrem vernarrt und denkt bei jeder Gelegenheit an diese Sache und handelt danach.".parse().unwrap()
			},
			Disadvantage {
				name: "Feigling".parse().unwrap(),
				description: "Charakter hat Angst vor Kämpfen und versucht diese zu meiden. Kommt er doch in einen, so kann er sich die erste Kampfrunde nicht bewegen.".parse().unwrap()
			},
			Disadvantage {
				name: "Feind".parse().unwrap(),
				description: "Person hat einen natürlichen Feind, der regelmäßig auftaucht. Wird er besiegt, kommen Nachfolger, je nach Schwere 1 bis 5 IP.".parse().unwrap()
			},
			Disadvantage {
				name: "Fetischist".parse().unwrap(),
				description: "Person besitzt einen Fetisch (Beispiele: BD/SM, Lack/Leder/Latex, bestimmte Körperteile, Objekte, AB/DL, WAM, Gummi, Nekrophilie, Tiere, Amputationen,... und noch exotischere Dinge...) . Sobald diese mit dem entsprechenden Reiz konfrontiert wird, erhält die Person den Zustand \"Erregt\", bis der Reiz verschwunden ist. Bleibt die Person dem Reiz lange ausgesetzt, kann das Ganze seltsam enden. Mehrere Fetische können den Effekt auch mehrfach hervorrufen.".parse().unwrap()
			},
			Disadvantage {
				name: "Galaktisches Ziel".parse().unwrap(),
				description: "Charakter wird von einem Jäger eines anderen Planeten als Tötungsziel ausgemacht. Der Jäger wird dauerhaft bei dem Spieler sein und versuchen ihn zu töten.".parse().unwrap()
			},
			Disadvantage {
				name: "Geisterwahn".parse().unwrap(),
				description: "Charakter hat Angst vor Geistern und denkt bei paranormalen Aktivitäten, dass ein Geist in der Nähe ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Geistig eingeschränkt".parse().unwrap(),
				description: "Charakter kann auch bei geistigen Proben Patzer erzielen.".parse().unwrap()
			},
			Disadvantage {
				name: "Gemeistert".parse().unwrap(),
				description: "Charakter hat einen Meister. Er ist somit eine Art Sklave. Er muss diesem dienen und kann sich nicht gegen diesen auflehnen. Tut er doch, verliert er jedes Mal dafür 100 EP. Den Meister kann er sich selbst überlegen/selbst wählen.".parse().unwrap()
			},
			Disadvantage {
				name: "Gespaltene Persönlichkeit".parse().unwrap(),
				description: "Charakter ändert in regelmäßigen Abständen seine Persönlichkeit. Es können mehrere Persönlichkeiten sein. (2 IP für die erste Persönlichkeit, für jede Weitere +1 IP)".parse().unwrap()
			},
			Disadvantage {
				name: "Glasknochen".parse().unwrap(),
				description: "-200HP bei HP+ (fix). Charakter kann keine Möglichkeit durch etwas anderes als ST, die Stufe und Ränge HP zu erhalten.".parse().unwrap()
			},
			Disadvantage {
				name: "Glücksspielsüchtig".parse().unwrap(),
				description: "Charakter macht aus Vielem ein Glücksspiel. Bei Entscheidungen oder körperlich relevanten Proben wirft er eine Münze, um sich die Entscheidung abzunehmen.".parse().unwrap()
			},
			Disadvantage {
				name: "Gottes Wille".parse().unwrap(),
				description: "Spieler wirft nach einer Story eine Münze. Bei Kopf erhält er nur 50% der EP, bei Zahl bekommt er 1000 weitere EP. Außerdem wirft er bei jedem Skillpunkt eine Münze. Bei Kopf erhält er zwei Nachteile, bei Zahl einen weiteren SP. Zusätzlich wirft er beim Ausgeben eines Skillpunktes eine Münze. Bei Kopf verliert er alle Skillpunkte, bei Zahl muss er diesen nicht zahlen.".parse().unwrap()
			},
			Disadvantage {
				name: "Herzlos".parse().unwrap(),
				description: "Charakter kann kaum Freunde erhalten und will am liebsten, dass alle einfach nur sterben. Affektivitätswürfe sind bei ihm meistens negativ.".parse().unwrap()
			},
			Disadvantage {
				name: "Hipster".parse().unwrap(),
				description: "Charakter hat das dringende Bedürfnis technisch gesehen immer auf dem neusten Stand zu sein.".parse().unwrap()
			},
			Disadvantage {
				name: "Humorlos".parse().unwrap(),
				description: "Charakter versteht Witze nie.".parse().unwrap()
			},
			Disadvantage {
				name: "Insomnie".parse().unwrap(),
				description: "Schwäche durch Schlafmangel. Der Charakter hat daher keinerlei Konzentration.".parse().unwrap()
			},
			Disadvantage {
				name: "Instabil".parse().unwrap(),
				description: "Charakter ändert sein Lebewesen alle drei Stunden zufällig. Charakter muss Magie besitzen.Seine Skilltrees und Stufenpläne bleiben gleich.".parse().unwrap()
			},
			Disadvantage {
				name: "Kurzsichtig".parse().unwrap(),
				description: "Charakter bekommt doppelte Entfernungsmali. F = 1 (fix). Nur wählbar, wenn F auch mindestens 1/1 ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Körperlich eingeschränkt".parse().unwrap(),
				description: "Charakter besitzt ein körperliches Handicap. Je nach Schweregrad zwischen 1 und 4 IP.".parse().unwrap()
			},
			Disadvantage {
				name: "Limitierte Auflage".parse().unwrap(),
				description: "Alle Limits des Charakters sind fest auf 5.".parse().unwrap()
			},
			Disadvantage {
				name: "Naiv".parse().unwrap(),
				description: "Charakter glaubt fast alles was andere ihm erzählen.".parse().unwrap()
			},
			Disadvantage {
				name: "Orientierungslos".parse().unwrap(),
				description: "Charakter kann sich nicht orientieren. Navigation wird um 20 erschwert und der Charakter kann links und rechts nicht unterscheiden.".parse().unwrap()
			},
			Disadvantage {
				name: "Paniker".parse().unwrap(),
				description: "Charakter hat vor etwas panische Angst. Beim kleinsten Verdacht, verfällt er in Hysterie und Panik. Er wird unvorsichtig und rücksichtslos. Er neigt dazu ohnmächtig zu werden. Sanität sinkt im Panikzustand auf 0.".parse().unwrap()
			},
			Disadvantage {
				name: "Pazifist".parse().unwrap(),
				description: "Unschuldig getötete Opfer → -100 EP".parse().unwrap()
			},
			Disadvantage {
				name: "Rassistisch".parse().unwrap(),
				description: "Charakter hasst eine bestimmte Personengruppe/Gruppe von Lebewesen. Trifft er auf diese lässt er unangebrachte Kommentare ab und neigt mehr zu Gewalt. Wird wie Voreingenommen behandelt. Also 1x: 2 IP 3x: 4 IP 6x: 6 IP 10x: 8 IP 15x: 10 IP".parse().unwrap()
			},
			Disadvantage {
				name: "Raucher".parse().unwrap(),
				description: "Charakter muss stündlich rauchen. Er erleidet sonst Entzugserscheinungen. Bei Entzug verliert er die Hälfte der Konzentration. Außerdem werden Immunsystem-Proben um 30 erschwert.".parse().unwrap()
			},
			Disadvantage {
				name: "Schlechtes Immunsystem".parse().unwrap(),
				description: "Charakter ist anfällig gegenüber Krankheiten. Sobald er in kontakt mit Erregern kommt, würfelt er einen W6. Bei einem Erfolg infiziert er sich nicht.".parse().unwrap()
			},
			Disadvantage {
				name: "Schwere Allergie".parse().unwrap(),
				description: "Großen Schaden + Malus von -8 durch gewisse Stoffe".parse().unwrap()
			},
			Disadvantage {
				name: "Seelenlos".parse().unwrap(),
				description: "Charakter verliert tatsächlich seine Seele.Zahlreiche magische Aktionen sind nun unmöglich, wie das Erfinden von Zaubern, Ritualen und Runen, schmieden von Zauberformeln oder das Herstellen magischer Gegenstände. Astralschaden kann nicht mehr abgewehrt werden.".parse().unwrap()
			},
			Disadvantage {
				name: "Selbstverliebt".parse().unwrap(),
				description: "Der Charakter ist in sich verliebt und lobt sich bei jeder Aktion. Dafür kritisiert er andere immer.".parse().unwrap()
			},
			Disadvantage {
				name: "Sozialstress".parse().unwrap(),
				description: "Charakter kommt nicht gut mit vielen Leuten klar. Sobald er unter vielen Personen ist erhält er -3 auf WK und UM.".parse().unwrap()
			},
			Disadvantage {
				name: "Spielleiterhass".parse().unwrap(),
				description: "Der Spielleiter behandelt den Charakter zu Ungunsten, wenn er mit einer Münze Zahl wirft.".parse().unwrap()
			},
			Disadvantage {
				name: "Suizidgefährdet".parse().unwrap(),
				description: "Charakter hat das Ziel sich das Leben zu nehmen. Wann immer sich die Gelegenheit bieten könnte, wirft er einen W6. Bei einer 1 wird er die Gelegenheit irgendwie ausnutzen.".parse().unwrap()
			},
			Disadvantage {
				name: "Technik-Freak".parse().unwrap(),
				description: "Charakter erstellt als Hobby ständig technische Spielzeuge, die jedoch nie ihren Zweck komplett richtig erfüllen und nebenbei immer kleine Nebenwirkungen haben.".parse().unwrap()
			},
			Disadvantage {
				name: "Technik-Niete".parse().unwrap(),
				description: "Charakter versteht nichts von Technik. Wenn er damit arbeitet, funktioniert nichts und alles geht schief.".parse().unwrap()
			},
			Disadvantage {
				name: "Tollpatsch".parse().unwrap(),
				description: "Zweien werden bei Fertigkeiten zu Einsen. Sind diese improvisiert sind sogar Dreien Einsen.".parse().unwrap()
			},
			Disadvantage {
				name: "Trottel".parse().unwrap(),
				description: "Improvisierte Fertigkeiten → -3".parse().unwrap()
			},
			Disadvantage {
				name: "Unentschlossen".parse().unwrap(),
				description: "Charakter wird schnell von anderen zu etwas überredet.".parse().unwrap()
			},
			Disadvantage {
				name: "Ungebildet".parse().unwrap(),
				description: "Charakter kann egal mit welcher Stufe nur 10 FP und FG in Fertigkeiten investieren.".parse().unwrap()
			},
			Disadvantage {
				name: "Ungeschickt".parse().unwrap(),
				description: "Spieler muss bei jeder körperlichen Probe einen W6 würfeln. Bei einer 1 entsteht ein Patzer.".parse().unwrap()
			},
			Disadvantage {
				name: "Unkontrolliert".parse().unwrap(),
				description: "Charakter besitzt eine Kraft, die er nicht kontrollieren kann. Opfer widersteht ggf. mit Magie gegen Magie.".parse().unwrap()
			},
			Disadvantage {
				name: "Verbissen".parse().unwrap(),
				description: "Charakter beharrt strickt auf seiner Meinung und lässt sich nicht überzeugen, auch wenn seine Meinung falsch ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Verbittert".parse().unwrap(),
				description: "Charakter erlebte eine Tragödie in der Vergangenheit und traut deshalb Fremden nicht. Affektivitätswürfe werden halbiert. Fremde werden zunächst ignoriert.".parse().unwrap()
			},
			Disadvantage {
				name: "Verflucht".parse().unwrap(),
				description: "Der Charakter wird von einer Art Fluch geplagt. A: Finsternis -> Dem Charakter wird heimlich im Dunkeln ein Organ geklaut, erhält es irgendwann aber wieder. (5 IP) B: Sobald der Charakter etwas isst oder trinkt, friert ein zufälliger Gegenstand in der Nähe ein oder fängt an zu brennen. (2 IP) C: Sobald sich der Charakter jemandem auf 2m angenähert hat, teleportiert sich ein Gegenstand in der Nähe 1m über den Charakter. (3 IP) D: Berührt der Charakter Wasser, kann er nicht mehr sehen, bis er wieder trocken ist.(4 IP) E: Sobald der Charakter etwas mit Rhythmus hört, kann er sich nur noch passend zum Rhythmus bewegen. (1 IP) F: Der Charakter fliegt alle 10 Minuten von seinem Standpunkt auf den Boden. (4 IP) G: Sobald der Charakter eine UM-Aktionsfertigkeit würfelt, kann er ab da nur noch in Reimen sprechen, bis er erneut würfelt. (2 IP) H: Charakter erhält nach jeder Story einen zufälligen Nachteil. (2 IP) I: Charakter spricht alle 24h einen anderen Akzent/Dialekt (aus Liste). (2 IP)".parse().unwrap()
			},
			Disadvantage {
				name: "Verfolgungswahn".parse().unwrap(),
				description: "Charakter glaubt dauerhaft verfolgt zu werden.".parse().unwrap()
			},
			Disadvantage {
				name: "Verfressen".parse().unwrap(),
				description: "Übermäßiger Konsum von Nahrungsmitteln.".parse().unwrap()
			},
			Disadvantage {
				name: "Vergesslich".parse().unwrap(),
				description: "Nach IN-Proben wird ein W6 geworfen. Bei einem Erfolg gilt die Probe, bei einem Nichterfolg nicht und bei einer 1 glaubt er an ein falsches Ergebnis. Manchmal erinnert er sich auch an Dinge, die nie passiert sind.".parse().unwrap()
			},
			Disadvantage {
				name: "Verhext".parse().unwrap(),
				description: "Sobald der Spieler bei einer Handlung ist, funktioniert gar nichts mehr nach Plan. Proben werden im Extremfall wie Patzer behandelt, auch wenn es keine sind.".parse().unwrap()
			},
			Disadvantage {
				name: "Verliebt".parse().unwrap(),
				description: "Charakter kann nicht aus eigenem Willen handeln, sobald eine Person vor ihm steht, die er \"mag\".".parse().unwrap()
			},
			Disadvantage {
				name: "Verschuldet".parse().unwrap(),
				description: "Charakter ist fast immer pleite und startet mit 0 Drachmen nach der Charaktererstellung. Außerdem bekommt er 2 Millionen Drachmen Schulden, die er abbezahlen muss.".parse().unwrap()
			},
			Disadvantage {
				name: "Voreilig".parse().unwrap(),
				description: "Charakter muss in gewissen Situationen eine Entscheidung vorab fällen. Was immer nachgehend passiert oder dagegen spricht, ignoriert der Charakter und folgt seiner voreiligen Entscheidung.".parse().unwrap()
			},
			Disadvantage {
				name: "Voreingenommen".parse().unwrap(),
				description: "Charakter mag eine Personengruppe oder eine Art von Lebewesen nicht. Diesen gegenüber verhält er sich zurückhaltend oder denkt abfällig. Wird dieser Nachteil mehrfach gewählt, gibt es nur alle n+1 mal einen weiteren IP. Also 1x: 1 IP 3x: 2 IP 6x: 3 IP 10x: 4 IP 15x: 5 IP".parse().unwrap()
			},
			Disadvantage {
				name: "Wahnsinnig".parse().unwrap(),
				description: "Sanität ist fix auf 0. IN auf 10, aber UM auf 0, alle UM Proben, Technik, Entschlossenheit und Fingerfertigkeit mit Malus von -5, wenn der Nachteil aktiv ist.".parse().unwrap()
			},
			Disadvantage {
				name: "Zauberniete".parse().unwrap(),
				description: "Charakter beherrscht alle Zaubersprüche, weiß aber nie, welchen er wie anwendet. Wenigstens kann er unterscheiden, ob der Zauber, den er wirken will, Schaden macht oder nicht.".parse().unwrap()
			},
			Disadvantage {
				name: "Zerfallen".parse().unwrap(),
				description: "Charakter ist physisch so instabil, dass er in regelmäßigen Abständen unwillkürlich zwischen den ersten 8 verschiedenen Dimensionsebenen hin- und herwechselt.".parse().unwrap()
			},
			Disadvantage {
				name: "Zittrig".parse().unwrap(),
				description: "Charakter zittert schubweise. In dem Fall sinkt GES um 5 und Fünfen gelten in der Zeit nicht mehr als Erfolge.".parse().unwrap()
			}
		];
	}
}

impl Display for Disadvantage {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}\n{}", self.name, self.description)
	}
}

pub fn get_random(disadvantages: &Vec<Disadvantage>) -> Disadvantage {
	disadvantages[random_usize(0, disadvantages.len() - 1)].clone()
}
