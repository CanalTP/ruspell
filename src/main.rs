extern crate rustc_serialize;
extern crate csv;
extern crate structopt;
extern crate encoding;
extern crate regex;
extern crate ispell;
extern crate unicode_normalization;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate lazy_static;

mod utils;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i",
                help = "Path to input CSV file to be processed \
                        (typically a GTFS stops.txt file).")]
    input: String,

    #[structopt(long = "output", short = "o",
                help = "Path to output CSV file after processing \
                        (same as input, name column processed).")]
    output: Option<String>,

    #[structopt(long = "rules", short = "r", default_value = "rules.csv",
                help = "Path to output rules.csv file \
                        (modifications description).")]
    rules: String,

    #[structopt(long = "id", short = "I", default_value = "stop_id",
                help = "The heading name of the column that is the unique id of the record.")]
    heading_id: String,

    #[structopt(long = "name", short = "N", default_value = "stop_name",
                help = "The heading name of the column that needs a spell_check.")]
    heading_name: String,
}


#[derive(Debug)]
struct Record {
    id: String,
    name: String,
    raw: Vec<String>,
}

struct RecordIter<'a, R: std::io::Read + 'a> {
    iter: csv::StringRecords<'a, R>,
    id_pos: usize,
    name_pos: usize,
}
impl<'a, R: std::io::Read + 'a> RecordIter<'a, R> {
    fn new(r: &'a mut csv::Reader<R>, heading_id: &str, heading_name: &str) -> csv::Result<Self> {
        let headers = try!(r.headers());

        let get_optional_pos = |name| headers.iter().position(|s| s == name);
        let get_pos = |field| {
            get_optional_pos(field).ok_or_else(|| {
                csv::Error::Decode(format!("Invalid file, cannot find column '{}'", field))
            })
        };

        Ok(RecordIter {
               iter: r.records(),
               id_pos: try!(get_pos(heading_id)),
               name_pos: try!(get_pos(heading_name)),
           })
    }
}
impl<'a, R: std::io::Read + 'a> Iterator for RecordIter<'a, R> {
    type Item = csv::Result<Record>;
    fn next(&mut self) -> Option<Self::Item> {
        fn get(record: &[String], pos: usize) -> csv::Result<&str> {
            match record.get(pos) {
                Some(s) => Ok(s),
                None => Err(csv::Error::Decode(format!("Failed accessing record '{}'.", pos))),
            }
        }

        self.iter.next().map(|r| {
            r.and_then(|r| {
                let id = try!(get(&r, self.id_pos)).to_string();
                let name = try!(get(&r, self.name_pos)).to_string();
                Ok(Record {
                       id: id,
                       name: name,
                       raw: r,
                   })
            })
        })
    }
}


fn new_record_iter<'a, R: std::io::Read + 'a>(r: &'a mut csv::Reader<R>,
                                              heading_id: &str,
                                              heading_name: &str)
                                              -> (std::iter::FilterMap<RecordIter<'a, R>,
                                                                       fn(csv::Result<Record>)
                                                                          -> Option<Record>>,
                                                  Vec<String>,
                                                  usize) {
    fn reader_handler(rc: csv::Result<Record>) -> Option<Record> {
        rc.map_err(|e| println!("error at csv line decoding : {}", e)).ok()
    }
    let headers = r.headers().unwrap();
    let rec_iter = RecordIter::new(r, heading_id, heading_name)
        .expect("Can't find needed fields in the header.");
    let pos = rec_iter.name_pos;

    (rec_iter.filter_map(reader_handler), headers, pos)
}


#[derive(Debug, RustcEncodable)]
struct RecordRule {
    id: String,
    old_name: String,
    new_name: String,
}


use regex::Regex;
use regex::RegexSet;
fn must_be_lower(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?i)^(en|sur|et|sous|de|du|des|le|la|les|au|aux|un|une|à)$").unwrap();
    }
    RE.is_match(text)
}

fn must_be_upper(text: &str) -> bool {
    lazy_static! {
        static ref RE: RegexSet =
            RegexSet::new(&[
                r"(?i)^(RER|CDG|CES|ASPTT|PTT|EDF|INRIA|INRA|CRC|HEC|SNCF|RATP|HLM|CHR|CHU)$",
                r"(?i)^(ZA|ZI|RPA|CFA|CEA|CC|CCI|UFR|CPAM|ANPE|RN\d*|\w*\d\w*|RD\d*)$",
                r"(?i)^(XL|X{0,3})(IX|IV|V?I{0,3})$",
                ]).unwrap();
    }
    RE.is_match(text)
}

fn sed_all(name: String) -> String {
    if must_be_lower(&name) {
        return name.to_lowercase();
    }

    if must_be_upper(&name) {
        return name.to_uppercase();
    }

    let lower_case = name.to_lowercase();
    if lower_case == "gal" {
        return "Général".to_string();
    } else if lower_case == "mal" {
        return "Maréchal".to_string();
    }

    name
}


use regex::Captures;
fn regex_all_name(name: String) -> String {
    lazy_static! {
        static ref RE_SAINT: Regex =
            Regex::new(r"(?i)(^|\W)s(?:ain)?t(e?)\W").unwrap();
    }
    let res = RE_SAINT.replace_all(&name, "${1}Saint${2}-");

    lazy_static! {
        static ref RE_ND: Regex =
            Regex::new(r"(?i)(^|\W)n(?:otre)?[ -]*d(?:ame)?(\W|$)").unwrap();
    }
    let res = RE_ND.replace_all(&res, "${1}Notre-Dame${2}");

    lazy_static! {
        static ref RE_PLACE: Regex =
            Regex::new(r"(?i)(^|\W)pl\.?(\W|$)").unwrap();
    }
    let res = RE_PLACE.replace_all(&res, "${1}Place${2}");

    lazy_static! {
        static ref RE_BOULEVARD: Regex =
            Regex::new(r"(?i)(^|\W)bl?v?d\.?(\W|$)").unwrap();
    }
    let res = RE_BOULEVARD.replace_all(&res, "${1}Boulevard${2}");

    lazy_static! {
        static ref RE_AVENUE: Regex =
            Regex::new(r"(?i)(^|\W)ave?\.?(\W|$)").unwrap();
    }
    let res = RE_AVENUE.replace_all(&res, "${1}Avenue${2}");

    lazy_static! {
        static ref RE_LIEU_DIT: Regex =
            Regex::new(r"(?i)(^|\W)rte(\W|$)").unwrap();
    }
    let res = RE_LIEU_DIT.replace_all(&res, "${1}Route${2}");

    lazy_static! {
        static ref RE_SAINT_LOUIS: Regex =
            Regex::new(r"(?i)(^|\W)Saint-Louis(\W|$)").unwrap();
    }
    let res = RE_SAINT_LOUIS.replace_all(&res, "${1}Saint Louis${2}");

    lazy_static! {
        static ref RE_A: Regex =
            Regex::new(r"(?i) a ").unwrap();
    }
    let res = RE_A.replace_all(&res, " à ");

    lazy_static! {
        static ref RE_ROND_POINT: Regex =
            Regex::new(r"(?i)(^|\W)ro?n?d[ \.-]?po?i?n?t ").unwrap();
    }
    let res = RE_ROND_POINT.replace_all(&res, "${1}Rond-Point ");

    lazy_static! {
        static ref RE_SPACES: Regex =
            Regex::new(r"(?i)  +").unwrap();
    }
    let res = RE_SPACES.replace_all(&res, " ");

    lazy_static! {
        static ref RE_QUOTE: Regex =
            Regex::new(r"(?i)(^|\W)([ld])[ '](h[aiouye]|[aiouy]|et[^ ]|e[^t].)").unwrap();
    }
    let res = RE_QUOTE.replace_all(&res, |caps: &Captures| {
        format!("{}{}'{}", &caps[1], &caps[2].to_lowercase(), &caps[3])
    });

    res.into_owned()
}


use std::thread::sleep;
use ispell::SpellLauncher;
fn ispell(name: String) -> String {
    let mut checker = SpellLauncher::new()
        .aspell()
        .dictionary("fr_FR")
        .launch()
        .unwrap();
    let errors = checker.check(&name).unwrap();
    for e in errors {
        println!("'{}' (pos: {}) is misspelled!", &e.misspelled, e.position);
        if !e.suggestions.is_empty() {
            println!("Maybe you meant '{}'?", &e.suggestions[0]);
        }
    }
    sleep(std::time::Duration::new(1, 0));
    name
}

/// management of all names
use utils::*;
fn process_record(rec: &Record) -> Option<RecordRule> {
    let mut new_name = decode(rec.name.clone());
    new_name = snake_case(new_name);

    let mut tmp = String::new();
    for word in get_words(&new_name) {
        tmp.push_str(&sed_all(word.to_string()));
    }

    new_name = regex_all_name(tmp);

    //new_name = ispell(new_name);

    new_name = first_upper(new_name);

    if rec.name == new_name {
        None
    } else {
        Some(RecordRule {
                 id: rec.id.clone(),
                 old_name: rec.name.clone(),
                 new_name: new_name,
             })
    }
}



use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
struct SpellChecker {
    aspell: ispell::SpellChecker,
    nb_replace: u32,
    nb_error: u32,
}
impl SpellChecker {
    fn new() -> std::result::Result<Self, String> {
        if let Ok(aspell_checker) =
            SpellLauncher::new()
                .timeout(6)
                .aspell()
                .dictionary("fr")
                .launch() {
            Ok(SpellChecker {
                   aspell: aspell_checker,
                   nb_replace: 0,
                   nb_error: 0,
               })
        } else {
            Err("Impossible to launch aspell".to_string())
        }
    }

    fn add_word(&mut self, new_word: &str) -> ispell::Result<()> {
        for c in new_word.chars() {
            if !c.is_alphabetic() {
                println!("{}", c);
            }
        }

        self.aspell.add_word(new_word)
    }

    fn check(&mut self, name: String) -> String {

        let errors_res = self.aspell.check(&name);
        if let Err(e) = errors_res {
            print!("{:?}", e);
            println!(" ({})", name);
            self.nb_error += 1;
            return name;
            //errors_res = self.aspell.check(&name);
        }

        let misspelt_errors = errors_res.unwrap();
        /*if misspelt_errors.is_empty() {
            println!("all is ok ({})", name);
        }*/

        for e in misspelt_errors {
            //println!("'{}' (pos: {}) is misspelled!", &e.misspelled, e.position);
            if !e.suggestions.is_empty() {
                //println!("Maybe you meant : '{:?}'?", &e.suggestions[0]);
            } else {
                //println!("Nothing better to offer...");
            }

            if !&e.suggestions.is_empty() {
                let normed_miss: String = e.misspelled
                    .nfkd()
                    .filter(|c| !is_combining_mark(*c))
                    .flat_map(char::to_lowercase)
                    .collect();
                let normed_sugg: String = e.suggestions[0]
                    .nfkd()
                    .filter(|c| !is_combining_mark(*c))
                    .flat_map(char::to_lowercase)
                    .collect();
                if normed_miss == normed_sugg && e.misspelled.len() < e.suggestions[0].len() {
                    println!("REPLACE VALID  : {} > {} ({})",
                             &e.misspelled,
                             &e.suggestions[0],
                             name);
                    self.nb_replace += 1;
                }
                /*let mut m_c = e.misspelled.chars();
                for s_c in e.suggestions[0].chars() {
                    if s_c >= 128 as char
                }*/
                /*if &e.misspelled.chars().count() == &e.suggestions[0].chars().count() {
                    for (m, s) in &e.misspelled.chars().iter.zip(&e.suggestions[0]
                                                                      .chars()
                                                                      .iter) {}

                }*/
            }

            /*if !&e.suggestions.is_empty() && &e.misspelled.len() == &e.suggestions[0].len() &&
               &e.misspelled.chars().zip(&e.suggestions[0].chars()).all(|(m, s)| {
                                                                            s >= 128 as char ||
                                                                            m == s
                                                                        }) {
                println!("REPLACE VALID: {} > {}", &e.misspelled, &e.suggestions[0]);
            }*/
        }
        name
    }
}


fn main() {
    let args = Args::from_args();

    let mut rdr = csv::Reader::from_file(args.input).unwrap().double_quote(true);

    let (records, headers, name_pos) =
        new_record_iter(&mut rdr, &args.heading_id, &args.heading_name);

    let mut wtr_rules = csv::Writer::from_file(args.rules).unwrap();
    wtr_rules.encode(("id", "old_name", "new_name")).unwrap();

    let mut wtr_stops = args.output.as_ref().map(|f| csv::Writer::from_file(f).unwrap());
    wtr_stops.as_mut().map(|w| w.encode(headers).unwrap());

    let mut aspell = SpellChecker::new().unwrap();
    aspell.add_word("L'Haÿ-les-Roses").unwrap();

    for mut rec in records {
        if let Some(rule) = process_record(&rec) {
            rec.raw[name_pos] = rule.new_name.clone();

            wtr_rules.encode(&rule).unwrap();
        }

        let new_name = aspell.check(rec.raw[name_pos].clone());
        //println!("FINAL: {}", new_name);

        wtr_stops.as_mut().map(|w| w.encode(&rec.raw).unwrap());
    }

    println!("replace: {} error: {}", aspell.nb_replace, aspell.nb_error);
}

// TODO :
// "12eme"" minuscules,
// "prés hauts"
//remplacer que si pas d'accent au départ,
// essayer de vider le dictionnaire
// De Gaulle

