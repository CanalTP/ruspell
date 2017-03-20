extern crate rustc_serialize;
extern crate csv;
extern crate structopt;
extern crate encoding;
extern crate regex;
extern crate ispell;
extern crate unicode_normalization;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate lazy_static;

mod utils;
mod regex_processor;
mod ispell_wrapper;
mod bano_reader;
mod records_reader;
mod errors;

use structopt::StructOpt;
use ispell_wrapper::SpellCheck;
use records_reader::Record;
use regex_processor::RegexProcessor;
use std::io;
use std::fs::File;
use errors::{Result, ResultExt};


#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i",
                help = "Path to input CSV file to be processed \
                        (typically a GTFS stops.txt file).")]
    input: String,

    #[structopt(long = "param", short = "p",
                help = "Path to param file to be read.")]
    param: String,

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

// define params file structure
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ProcessSequence {
    processes: Vec<NameProcessor>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum NameProcessor {
    Decode(Decode),
    FirstLetterUppercase,
    SnakeCase,
    LowercaseWord(FixedcaseWord),
    UppercaseWord(FixedcaseWord),
    RegexReplace(RegexReplace),
    IspellCheck(IspellCheck),
    LogSuspicious,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Decode {
    from_encoding: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FixedcaseWord {
    words: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct RegexReplace {
    from: String,
    to: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct IspellCheck {
    bano_files: Vec<String>,
}


/// management of all processing applied to names
fn process_record(rec: &Record,
                  ispell: &mut SpellCheck,
                  regex: &RegexProcessor)
                  -> Result<Option<RecordRule>> {

    let mut new_name = utils::decode(&rec.name);
    new_name = regex_processor::sed_whole_name_before(&new_name);
    new_name = ispell.check(&new_name)?;
    new_name = utils::snake_case(&new_name);
    new_name = regex_processor::fixed_case_word(&new_name, regex);
    new_name = regex_processor::sed_whole_name_after(&new_name);
    new_name = utils::first_upper(&new_name);

    regex_processor::log_suspicious(&new_name);

    if rec.name == new_name {
        Ok(None)
    } else {
        Ok(Some(RecordRule {
                    id: rec.id.clone(),
                    old_name: rec.name.clone(),
                    new_name: new_name,
                }))
    }
}


#[derive(Debug, RustcEncodable)]
struct RecordRule {
    id: String,
    old_name: String,
    new_name: String,
}


fn run() -> Result<()> {
    use bano_reader;

    let args = Args::from_args();

    let mut rdr_stops = csv::Reader::from_file(args.input)
        .chain_err(|| "Could not open input file")?
        .double_quote(true);
    let (records, headers, name_pos) =
        records_reader::new_record_iter(&mut rdr_stops, &args.heading_id, &args.heading_name)?;

    // producing rules to be applied to re-spell names
    let mut wtr_rules =
        csv::Writer::from_file(args.rules).chain_err(|| "Could not open rules file")?;
    wtr_rules.encode(("id", "old_name", "new_name"))
        .chain_err(|| "Could not write header of rules file")?;

    // producing output and replacing names only if requested (wtr_stops is an Option)
    let mut wtr_stops = match args.output {
        Some(ref f) => Some(csv::Writer::from_file(f).chain_err(|| "Could not open output file")?),
        None => None,
    };
    wtr_stops.as_mut()
        .map_or(Ok(()), |w| w.encode(headers))
        .chain_err(|| "Could not write header of output file")?;

    //creating regex wrapper from params
    let mut regex = RegexProcessor::new();
    let param_rdr = File::open(args.param).chain_err(|| "Could not open param file")?;
    let sequence: ProcessSequence = serde_yaml::from_reader(param_rdr).chain_err(|| "Problem while reading param file")?;
    let mut bano_file = None;
    for a in sequence.processes {
        match a {
            NameProcessor::LowercaseWord(fcw) => {
                for w in fcw.words {
                    regex.add_fixed_case(&w)?;
                }
            }
            NameProcessor::UppercaseWord(fcw) => {
                for w in fcw.words {
                    regex.add_fixed_case(&w)?;
                }
            }
            NameProcessor::IspellCheck(ispell) => {
                bano_file = Some(ispell.bano_files[0].clone());
            }
            _ => (),
        }
    }

    // creating ispell manager (and populate dictionnary if requested)
    let mut ispell = SpellCheck::new().chain_err(|| "Could not create ispell manager")?;
    if bano_file.is_some() {
        bano_reader::populate_dict_from_file(&bano_file.unwrap(), &mut ispell)?;
    }

    for mut rec in records {
        if let Some(rule) = process_record(&rec, &mut ispell, &regex)? {
            rec.raw[name_pos] = rule.new_name.clone();
            wtr_rules.encode(&rule).chain_err(|| "Could not write into rules file")?;
        }
        wtr_stops.as_mut()
            .map_or(Ok(()), |w| w.encode(&rec.raw))
            .chain_err(|| "Could not write into output file")?;
    }
    Ok(())
}


fn main() {
    if let Err(ref e) = run() {
        use io::Write;
        let stderr = &mut io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}
