extern crate rustc_serialize;
extern crate csv;
extern crate structopt;
extern crate encoding;
extern crate regex;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate lazy_static;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i",
                help = "CSV file to be processed (typically a GTFS stops.txt file)")]
    input: String,

    #[structopt(long = "output", short = "o", default_value = "rules.csv",
                help = "Fusio rules.csv file")]
    output: String,

    #[structopt(long = "id", short = "d", default_value = "stop_id",
                help = "The heading name of the column that is the unique id of the record")]
    heading_id: String,

    #[structopt(long = "name", short = "s", default_value = "stop_name",
                help = "The heading name of the column that needs a spell_check.")]
    heading_name: String,
}


#[derive(Debug)]
struct Record {
    id: String,
    name: String,
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
                let id = try!(get(&r, self.id_pos));
                let name = try!(get(&r, self.name_pos));
                Ok(Record {
                       id: id.to_string(),
                       name: name.to_string(),
                   })
            })
        })
    }
}


fn new_record_iter<'a, R: std::io::Read + 'a>
    (r: &'a mut csv::Reader<R>,
     heading_id: &str,
     heading_name: &str)
     -> std::iter::FilterMap<RecordIter<'a, R>, fn(csv::Result<Record>) -> Option<Record>> {
    fn reader_handler(rc: csv::Result<Record>) -> Option<Record> {
        rc.map_err(|e| println!("error at csv line decoding : {}", e)).ok()
    }
    RecordIter::new(r, heading_id, heading_name)
        .expect("Can't find needed fields in the header.")
        .filter_map(reader_handler)
}


#[derive(Debug, RustcEncodable)]
struct RecordRule {
    id: String,
    old_name: String,
    new_name: String,
}


use encoding::Encoding;
use encoding::all::{ISO_8859_15, WINDOWS_1252};
use encoding::EncoderTrap;
fn decode(name: String) -> String {
    let latin9 = ISO_8859_15;
    if let Ok(Ok(res)) = latin9.encode(&name, EncoderTrap::Strict).map(String::from_utf8) {
        return res;
    }
    let latin1 = WINDOWS_1252;
    if let Ok(Ok(res)) = latin1.encode(&name, EncoderTrap::Strict).map(String::from_utf8) {
        return res;
    }
    name
}


fn get_words(name: &String) -> Vec<&str> {
    let mut words = Vec::<&str>::new();
    let mut index_start_word = 0;
    let mut is_current_alpha = name.chars()
        .next()
        .map(char::is_alphanumeric)
        .unwrap_or(true);
    for c in name.char_indices() {
        if c.1.is_alphanumeric() != is_current_alpha {
            words.push(&name[index_start_word..c.0]);
            is_current_alpha = c.1.is_alphanumeric();
            index_start_word = c.0;
        }
    }
    words.push(&name[index_start_word..]);
    words
}


fn first_upper(name: String) -> String {
    let mut chars = name.chars();
    let mut new_name = String::new();
    new_name.extend(chars.next().map(|c| c.to_uppercase().collect::<String>()));
    new_name.extend(chars);
    new_name
}


/// MUSEE dE La GARE sncf > Musee de la gare de lyon
fn first_upper_all_lower(name: String) -> String {
    let mut chars = name.chars();
    let mut new_name = String::new();
    new_name.extend(chars.next().map(|c| c.to_uppercase().collect::<String>()));
    new_name.extend(chars.flat_map(char::to_lowercase));
    new_name
}


/// every word becomes Mmmmmm
fn snake_case(name: String) -> String {
    let mut new_name = String::new();
    for word in get_words(&name) {
        new_name.push_str(&first_upper_all_lower(word.to_string()));
    }
    new_name
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
                r"(?i)^(ZA|ZI|RPA|CFA|CC|CCI|UFR|CPAM|ANPE|RN\d*|\w*\d\w*|RD\d*)$",
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


/// management of all names
fn process_record(rec: &Record) -> Option<RecordRule> {
    let mut new_name = decode(rec.name.clone());
    new_name = snake_case(new_name);

    let mut tmp = String::new();
    for word in get_words(&new_name) {
        tmp.push_str(&sed_all(word.to_string()));
    }

    new_name = regex_all_name(tmp);
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


fn main() {
    let args = Args::from_args();

    let mut rdr = csv::Reader::from_file(args.input).unwrap().double_quote(true);

    let records = new_record_iter(&mut rdr, &args.heading_id, &args.heading_name);

    let mut wtr = csv::Writer::from_file(args.output).unwrap();
    wtr.encode(("id", "old_name", "new_name")).unwrap();

    for rule in records.filter_map(|rec| process_record(&rec)) {
        wtr.encode(&rule).unwrap();
    }
}
