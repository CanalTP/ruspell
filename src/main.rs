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
mod regex_wrapper;
mod ispell_wrapper;
mod bano_reader;

use structopt::StructOpt;
use ispell_wrapper::SpellCheck;

#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i",
                help = "Path to input CSV file to be processed \
                        (typically a GTFS stops.txt file).")]
    input: String,

    #[structopt(long = "bano", short = "b",
                help = "Path to input BANO file to be read \
                        (street and city names for dictionnary).")]
    bano: Option<String>,

    #[structopt(long = "output", short = "o",
                help = "Path to output CSV file after processing \
                        (same as input, <name> column processed).")]
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
        let headers = r.headers()?;

        let get_optional_pos = |name| headers.iter().position(|s| s == name);
        let get_pos = |field| {
            get_optional_pos(field).ok_or_else(|| {
                csv::Error::Decode(format!("Invalid file, cannot find column '{}'", field))
            })
        };

        Ok(RecordIter {
               iter: r.records(),
               id_pos: get_pos(heading_id)?,
               name_pos: get_pos(heading_name)?,
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
                let id = get(&r, self.id_pos)?.to_string();
                let name = get(&r, self.name_pos)?.to_string();
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


/// management of all processing applied to names
fn process_record(rec: &Record, ispell: &mut SpellCheck) -> Option<RecordRule> {
    use utils;
    use regex_wrapper;

    let mut new_name = utils::decode(rec.name.clone());
    new_name = utils::snake_case(new_name);
    new_name = regex_wrapper::fixed_case_word(new_name);
    new_name = regex_wrapper::sed_whole_name(new_name);
    new_name = ispell.check(new_name);
    new_name = utils::first_upper(new_name);

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
    use bano_reader;

    let args = Args::from_args();

    let mut rdr_stops = csv::Reader::from_file(args.input).unwrap().double_quote(true);
    let (records, headers, name_pos) =
        new_record_iter(&mut rdr_stops, &args.heading_id, &args.heading_name);

    // producing rules to be applied to re-spell names
    let mut wtr_rules = csv::Writer::from_file(args.rules).unwrap();
    wtr_rules.encode(("id", "old_name", "new_name")).unwrap();

    // producing output and replacing names only if requested (wtr_stops is an Option)
    let mut wtr_stops = args.output.as_ref().map(|f| csv::Writer::from_file(f).unwrap());
    wtr_stops.as_mut().map(|w| w.encode(headers).unwrap());

    // creating ispell manager (and populate dictionnary if requested)
    let mut ispell = SpellCheck::new().unwrap();
    if let Some(bano_file) = args.bano {
        bano_reader::populate_dict_from_file(&bano_file, &mut ispell);
    }

    for mut rec in records {
        if let Some(rule) = process_record(&rec, &mut ispell) {
            rec.raw[name_pos] = rule.new_name.clone();
            wtr_rules.encode(&rule).unwrap();
        }
        wtr_stops.as_mut().map(|w| w.encode(&rec.raw).unwrap());
    }

    println!("Ispell replaced {} words and produced {} error",
             ispell.nb_replace(),
             ispell.nb_error());
}
